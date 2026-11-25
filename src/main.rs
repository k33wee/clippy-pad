mod app;
mod i18n;
mod services;

use app::Applet;

fn main() -> cosmic::iced::Result {
    let mut standalone = std::env::var_os("COSMIC_PANEL_NAME").is_none();
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--standalone" => standalone = true,
            "--help" | "-h" => {
                println!(
                    "Clippy Pad - Bluetooth controller manager for COSMIC\n\n\
                     Usage: cosmic-applet-clippy-pad [--standalone]\n\n\
                     Add the applet in COSMIC Settings -> Desktop -> Panel -> Applets.\n\
                     Without a panel host, a regular window opens instead."
                );
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {argument}. Use --help for usage.");
                std::process::exit(2);
            }
        }
    }
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    if standalone {
        cosmic::app::run::<Applet>(
            cosmic::app::Settings::default()
                .size(app::WINDOW_SIZE)
                .exit_on_close(true),
            true,
        )
    } else {
        cosmic::applet::run::<Applet>(false)
    }
}
