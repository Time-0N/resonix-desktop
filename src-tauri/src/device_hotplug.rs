use tauri::{AppHandle, Emitter};
use std::time::Duration;
use cpal::traits::DeviceTrait;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct DeviceEvent {
    name: String,
}


pub fn start_device_hotplug_monitor(app: AppHandle) {
    use cpal::traits::HostTrait;
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let mut last_name = host.default_output_device().and_then(|d| d.name().ok());
        loop {
            let cur = host.default_output_device().and_then(|d| d.name().ok());
            if cur != last_name {
                last_name = cur.clone();
                if let Some(name) = cur.clone() {
                    let _ = app.emit("audio:device", DeviceEvent { name });
                }
                // A robust approach is to tell the UI to recreate the engine or call a command that rebuilds the stream.
                // You can expose a command like `reinit_output()` that stops+rebuilds the stream and restarts playback.
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}