//! Applet state, commands, and the BlueZ session.
//!
//! The parent module re-exports the public types. The view lives here so it
//! can read this module's private fields.

mod view;

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::time::Duration;

use cosmic::app::ApplicationExt;
use cosmic::iced::window;
use cosmic::iced::{self, Subscription};
use cosmic::widget::{self, container};
use cosmic::{Application, Element};

use crate::app::bluetooth;
use crate::app::device::{DeviceList, Reconnect};

type Task = cosmic::app::Task<Message>;

/// How often the device list is reloaded while the popup is open.
const REFRESH_INTERVAL: Duration = Duration::from_secs(10);
const RECONNECT_TICK: Duration = Duration::from_secs(1);

const POPUP_SIZE: (u32, u32) = (440, 560);
pub(crate) const WINDOW_SIZE: iced::Size = iced::Size::new(480.0, 680.0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Devices,
    About,
}

/// A device request that is running; its row shows a spinner-like state.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Rename {
    addr: String,
    value: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    Close(window::Id),
    Refresh,
    DevicesLoaded(Result<DeviceList, String>),
    Tick,
    Disconnect(String),
    Reconnect(String),
    Remove(String),
    RenameStart(String),
    RenameInput(String),
    RenameSubmit,
    RenameCancel,
    /// A request on `addr` finished, and the device list was reloaded.
    RequestFinished {
        addr: String,
        result: Result<DeviceList, String>,
    },
    DismissError,
    ShowAbout,
    ShowHome,
    OpenLink(&'static str),
    LinkOpened,
}

pub struct Applet {
    core: cosmic::Core,
    standalone: bool,
    popup: Option<window::Id>,
    page: Page,
    /// Set while a device-list reload is in flight.
    loading: bool,
    devices: DeviceList,
    /// Addresses with a request in flight: disconnect, remove, or rename.
    pending: HashSet<String>,
    /// Controllers being re-paired, by address.
    reconnecting: HashMap<String, Reconnect>,
    /// The paired device whose name is being edited.
    rename: Option<Rename>,
    error: Option<String>,
}

impl Application for Applet {
    type Executor = cosmic::executor::Default;
    /// `standalone` is the application flag: a normal window, rather than a panel applet.
    type Flags = bool;
    type Message = Message;
    const APP_ID: &'static str = "io.github.k33wee.clippy-pad";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(mut core: cosmic::Core, standalone: bool) -> (Self, Task) {
        core.window.show_headerbar = standalone;
        core.window.sharp_corners = true;
        core.window.show_close = standalone;
        // COSMIC Panel drops a new layer surface when the applet pushes a
        // corner radius before the first configure.
        core.set_auto_corner_radius(cosmic::core::Auto::Window | cosmic::core::Auto::Popup);
        let mut app = Self {
            core,
            standalone,
            popup: None,
            page: Page::Devices,
            loading: false,
            devices: DeviceList::default(),
            pending: HashSet::new(),
            reconnecting: HashMap::new(),
            rename: None,
            error: None,
        };
        let main = app.core.main_window_id().unwrap_or(window::Id::RESERVED);
        let title = app.set_window_title(crate::fl!("app-title").to_string(), main);
        let refresh = app.refresh();
        (app, Task::batch([title, refresh]))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::Close(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        // The list is only worth polling while somebody can see it.
        let mut subscriptions = Vec::new();
        if self.standalone || self.popup.is_some() {
            subscriptions.push(iced::time::every(REFRESH_INTERVAL).map(|_| Message::Refresh));
        }
        if !self.reconnecting.is_empty() {
            // The countdown labels are drawn from the clock; they need a repaint.
            subscriptions.push(iced::time::every(RECONNECT_TICK).map(|_| Message::Tick));
        }
        let on_escape = match (self.page, self.rename.is_some()) {
            (Page::About, _) => Some(escape_shows_home as EscapeHandler),
            (Page::Devices, true) => Some(escape_cancels_rename as EscapeHandler),
            (Page::Devices, false) => None,
        };
        if let Some(handler) = on_escape {
            subscriptions.push(iced::event::listen_with(handler));
        }
        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task {
        match message {
            Message::TogglePopup if !self.standalone => {
                return if let Some(id) = self.popup.take() {
                    destroy_popup(id)
                } else {
                    self.page = Page::Devices;
                    self.open_popup().chain(self.refresh())
                };
            }
            Message::Close(id) if self.popup == Some(id) => {
                self.popup = None;
                return destroy_popup(id);
            }
            Message::Close(id) if self.core.main_window_id() == Some(id) => {
                return iced::exit();
            }
            Message::Refresh => return self.refresh(),
            Message::Tick => (),
            Message::DevicesLoaded(result) => self.devices_loaded(result),
            Message::Disconnect(addr) => {
                return self.request(
                    addr,
                    |addr| async move { bluetooth::disconnect(&addr).await },
                );
            }
            Message::Remove(addr) => {
                return self.request(addr, |addr| async move { bluetooth::remove(&addr).await });
            }
            Message::Reconnect(addr) => return self.reconnect(addr),
            Message::RequestFinished { addr, result } => {
                self.pending.remove(&addr);
                self.reconnecting.remove(&addr);
                self.devices_loaded(result);
            }
            Message::RenameStart(addr) => {
                let name = self
                    .devices
                    .paired
                    .iter()
                    .find(|device| device.addr == addr)
                    .map(|device| device.name.clone())
                    .unwrap_or_default();
                self.rename = Some(Rename { addr, value: name });
            }
            Message::RenameInput(value) => {
                if let Some(rename) = &mut self.rename {
                    rename.value = value;
                }
            }
            Message::RenameCancel => self.rename = None,
            Message::RenameSubmit => {
                let Some(rename) = self.rename.take() else {
                    return Task::none();
                };
                let name = rename.value.trim().to_string();
                if name.is_empty() {
                    self.error = Some(crate::fl!("rename-empty").to_string());
                    return Task::none();
                }
                return self.request(rename.addr, move |addr| {
                    let name = name.clone();
                    async move { bluetooth::rename(&addr, &name).await }
                });
            }
            Message::DismissError => self.error = None,
            Message::ShowAbout => self.page = Page::About,
            Message::ShowHome => {
                self.page = Page::Devices;
                self.rename = None;
            }
            Message::OpenLink(url) => {
                return cosmic::task::future(async move {
                    match tokio::process::Command::new("xdg-open")
                        .arg(url)
                        .status()
                        .await
                    {
                        Ok(status) if status.success() => {}
                        result => eprintln!("Could not open link: {result:?}"),
                    }
                    Message::LinkOpened
                });
            }
            Message::LinkOpened | Message::Close(_) | Message::TogglePopup => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.standalone {
            return container(view::popup(self))
                .class(cosmic::theme::Container::WindowBackground)
                .into();
        }
        let bytes: &'static [u8] = include_bytes!("../../resources/icon.svg");
        let handle = widget::icon::from_svg_bytes(bytes).symbolic(true);
        self.core
            .applet
            .icon_button_from_handle(handle)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        if self.standalone {
            container(view::popup(self))
                .class(cosmic::theme::Container::WindowBackground)
                .into()
        } else {
            self.core.applet.popup_container(view::popup(self)).into()
        }
    }

    fn style(&self) -> Option<iced::theme::Style> {
        if self.standalone {
            None
        } else {
            Some(cosmic::applet::style())
        }
    }
}

impl Applet {
    fn open_popup(&mut self) -> Task {
        let id = window::Id::unique();
        self.popup = Some(id);
        cosmic::surface::surface_task(cosmic::surface::action::app_popup::<Self>(
            |_| Default::default(),
            move |app| {
                let parent = app.core.main_window_id().unwrap_or(window::Id::RESERVED);
                app.core
                    .applet
                    .get_popup_settings(parent, id, Some(POPUP_SIZE), None, None)
            },
            None,
        ))
    }

    /// Forgets and re-pairs the controller. Its row shows a countdown while
    /// the user puts the controller in pairing mode.
    fn reconnect(&mut self, addr: String) -> Task {
        let Some(name) = self
            .devices
            .paired
            .iter()
            .find(|device| device.addr == addr)
            .map(|device| device.name.clone())
        else {
            return Task::none();
        };
        self.error = None;
        self.reconnecting
            .insert(addr.clone(), Reconnect::new(name.clone()));
        cosmic::task::future(async move {
            let result = bluetooth::reconnect(&addr, &name).await;
            Message::RequestFinished { addr, result }
        })
    }

    fn devices_loaded(&mut self, result: Result<DeviceList, String>) {
        self.loading = false;
        match result {
            Ok(devices) => {
                self.devices = devices;
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    /// Reloads the device list, unless a request that reloads it is already running.
    fn refresh(&mut self) -> Task {
        if self.loading || !self.pending.is_empty() {
            return Task::none();
        }
        self.loading = true;
        cosmic::task::future(async { Message::DevicesLoaded(bluetooth::load_devices().await) })
    }

    /// Runs `request` on `addr`, then reloads the device list. The row is
    /// marked pending so its buttons stay quiet until the result arrives.
    fn request<F, Fut>(&mut self, addr: String, request: F) -> Task
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: Future<Output = Result<DeviceList, String>> + Send + 'static,
    {
        self.error = None;
        self.pending.insert(addr.clone());
        cosmic::task::future(async move {
            let result = request(addr.clone()).await;
            Message::RequestFinished { addr, result }
        })
    }
}

type EscapeHandler = fn(iced::Event, iced::event::Status, window::Id) -> Option<Message>;

fn is_escape(event: &iced::Event) -> bool {
    matches!(
        event,
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            ..
        })
    )
}

fn escape_shows_home(
    event: iced::Event,
    _status: iced::event::Status,
    _id: window::Id,
) -> Option<Message> {
    is_escape(&event).then_some(Message::ShowHome)
}

fn escape_cancels_rename(
    event: iced::Event,
    _status: iced::event::Status,
    _id: window::Id,
) -> Option<Message> {
    is_escape(&event).then_some(Message::RenameCancel)
}

fn destroy_popup(id: window::Id) -> Task {
    cosmic::task::message(cosmic::Action::Surface(
        cosmic::surface::Action::DestroyPopup(id),
    ))
}
