//! BlueZ requests made from the applet. Every request that changes a device
//! reloads the device list before returning, so the popup never shows stale state.

use zbus::Connection;

use crate::app::device::{ConnectedDevice, DeviceList, PairedDevice, RECONNECT_SEARCH_TIME};
use crate::services;

/// Lists connected and paired controllers. A controller's battery is read from
/// the kernel's power-supply class, which only some drivers expose.
pub async fn load_devices() -> Result<DeviceList, String> {
    let conn = system_connection().await?;
    load(&conn).await
}

pub async fn disconnect(addr: &str) -> Result<DeviceList, String> {
    let conn = system_connection().await?;
    services::disconnect_device(&conn, addr)
        .await
        .map_err(|err| format!("Failed to disconnect device: {err}"))?
        .then_some(())
        .ok_or_else(|| "Device not found".to_string())?;
    load(&conn).await
}

pub async fn remove(addr: &str) -> Result<DeviceList, String> {
    let conn = system_connection().await?;
    services::remove_device(&conn, addr)
        .await
        .map_err(|err| format!("Failed to remove device: {err}"))?
        .then_some(())
        .ok_or_else(|| "Device not found".to_string())?;
    load(&conn).await
}

pub async fn rename(addr: &str, new_alias: &str) -> Result<DeviceList, String> {
    let conn = system_connection().await?;
    services::rename_paired_device(&conn, addr, new_alias)
        .await
        .map_err(|err| format!("Failed to rename device: {err}"))?
        .then_some(())
        .ok_or_else(|| "Device not found".to_string())?;
    load(&conn).await
}

/// Forgets a paired controller, then waits for it to show up in discovery so it
/// can be paired, trusted, and connected again. The controller must be put in
/// pairing mode by the user while this runs.
pub async fn reconnect(addr: &str, name: &str) -> Result<DeviceList, String> {
    let conn = system_connection().await?;
    services::remove_device(&conn, addr)
        .await
        .map_err(|err| format!("Failed to remove device: {err}"))?;

    let found = services::discover_device_by_address(&conn, addr, RECONNECT_SEARCH_TIME)
        .await
        .map_err(|err| format!("Failed to discover device: {err}"))?;
    if !found {
        return Err(crate::fl!("reconnect-not-found").to_string());
    }

    if !services::pair_device(&conn, addr)
        .await
        .map_err(|err| format!("Failed to pair device: {err}"))?
    {
        return Err(crate::fl!("reconnect-pair-failed").to_string());
    }

    if !services::trust_device(&conn, addr, true)
        .await
        .map_err(|err| format!("Failed to trust device: {err}"))?
    {
        return Err(crate::fl!("reconnect-trust-failed").to_string());
    }

    if !services::connect_device(&conn, addr)
        .await
        .map_err(|err| format!("Failed to connect device: {err}"))?
    {
        return Err(crate::fl!("reconnect-connect-failed").to_string());
    }

    if !name.is_empty() {
        // The alias is cosmetic; a failure here leaves a working controller.
        let _ = services::rename_paired_device(&conn, addr, name).await;
    }
    load(&conn).await
}

async fn system_connection() -> Result<Connection, String> {
    Connection::system()
        .await
        .map_err(|err| format!("DBus error: {err}"))
}

async fn load(conn: &Connection) -> Result<DeviceList, String> {
    let connected = services::list_connected_devices(conn)
        .await
        .map_err(|err| format!("Failed to list connected devices: {err}"))?
        .into_iter()
        .map(|(addr, name)| ConnectedDevice {
            battery: services::read_ps_controller_battery(&addr.to_lowercase())
                .ok()
                .flatten()
                .map(|(_device, capacity)| capacity),
            addr,
            name,
        })
        .collect();

    let paired = services::list_paired_devices(conn)
        .await
        .map_err(|err| format!("Failed to list paired devices: {err}"))?
        .into_iter()
        .map(|(addr, name)| PairedDevice { addr, name })
        .collect();

    Ok(DeviceList { connected, paired })
}
