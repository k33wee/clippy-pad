//! Device lists as BlueZ reports them, and the rows the popup draws.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How long the applet searches for a controller before giving up on it.
pub const RECONNECT_SEARCH_TIME: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectedDevice {
    pub addr: String,
    pub name: String,
    /// Battery charge in percent, when the kernel exposes it for the controller.
    pub battery: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairedDevice {
    pub addr: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceList {
    pub connected: Vec<ConnectedDevice>,
    pub paired: Vec<PairedDevice>,
}

/// A paired device while the applet re-pairs it. BlueZ has forgotten the
/// device by then, so the applet keeps its name to show it in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reconnect {
    pub name: String,
    pub deadline: Instant,
}

impl Reconnect {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            deadline: Instant::now() + RECONNECT_SEARCH_TIME,
        }
    }

    /// Time left before the search is given up.
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
}

/// One row of the paired list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairedRow {
    pub addr: String,
    pub name: String,
    /// Set when the controller is also connected right now.
    pub connected: bool,
    /// Set while the applet is searching for the controller.
    pub reconnecting: Option<Duration>,
}

impl DeviceList {
    /// The paired list as the popup shows it: BlueZ's view, with devices that
    /// are being re-paired shown from the applet's own record instead.
    #[must_use]
    pub fn paired_rows(&self, reconnecting: &HashMap<String, Reconnect>) -> Vec<PairedRow> {
        let mut rows: Vec<PairedRow> = self
            .paired
            .iter()
            .filter(|device| !reconnecting.contains_key(&device.addr))
            .map(|device| PairedRow {
                connected: self.is_connected(&device.addr),
                addr: device.addr.clone(),
                name: device.name.clone(),
                reconnecting: None,
            })
            .collect();
        rows.extend(reconnecting.iter().map(|(addr, reconnect)| PairedRow {
            addr: addr.clone(),
            name: reconnect.name.clone(),
            connected: false,
            reconnecting: Some(reconnect.remaining()),
        }));
        rows.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.addr.cmp(&b.addr)));
        rows
    }

    #[must_use]
    pub fn is_connected(&self, addr: &str) -> bool {
        self.connected.iter().any(|device| device.addr == addr)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{ConnectedDevice, DeviceList, PairedDevice, PairedRow, Reconnect};

    fn list() -> DeviceList {
        DeviceList {
            connected: vec![ConnectedDevice {
                addr: "AA:BB".into(),
                name: "DualSense".into(),
                battery: Some(80),
            }],
            paired: vec![
                PairedDevice {
                    addr: "CC:DD".into(),
                    name: "Xbox".into(),
                },
                PairedDevice {
                    addr: "AA:BB".into(),
                    name: "DualSense".into(),
                },
            ],
        }
    }

    #[test]
    fn paired_rows_are_sorted_and_mark_connected_devices() {
        let rows = list().paired_rows(&Default::default());
        assert_eq!(
            rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
            ["DualSense", "Xbox"]
        );
        assert!(rows[0].connected);
        assert!(!rows[1].connected);
    }

    #[test]
    fn paired_rows_replace_a_reconnecting_device_with_the_applet_record() {
        let reconnecting = [(
            "AA:BB".to_string(),
            Reconnect::new("DualSense (Edge)".into()),
        )]
        .into_iter()
        .collect();

        let rows = list().paired_rows(&reconnecting);

        assert_eq!(rows.len(), 2);
        let edge = &rows[0];
        assert_eq!(edge.addr, "AA:BB");
        assert_eq!(edge.name, "DualSense (Edge)");
        assert!(!edge.connected);
        let remaining = edge.reconnecting.expect("the search is still running");
        assert!(remaining <= super::RECONNECT_SEARCH_TIME);
        assert!(remaining > Duration::ZERO);
        assert_eq!(
            rows[1],
            PairedRow {
                addr: "CC:DD".into(),
                name: "Xbox".into(),
                connected: false,
                reconnecting: None,
            }
        );
    }

    #[test]
    fn reconnect_countdown_runs_towards_zero_and_stops_there() {
        let reconnect = Reconnect {
            name: "Pad".into(),
            deadline: Instant::now(),
        };
        assert_eq!(reconnect.remaining(), Duration::ZERO);
    }
}
