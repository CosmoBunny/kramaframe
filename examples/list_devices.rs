use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let available_hosts = cpal::available_hosts();
    println!("Available Hosts: {:?}", available_hosts);

    for host_id in available_hosts {
        println!("\n--- Host: {:?} ---", host_id);
        let host = cpal::host_from_id(host_id).unwrap();
        
        println!("  Default Input: {:?}", host.default_input_device().map(|d| d.name().unwrap_or("?".into())));
        println!("  Default Output: {:?}", host.default_output_device().map(|d| d.name().unwrap_or("?".into())));

        println!("  Input Devices:");
        if let Ok(devices) = host.input_devices() {
            for (i, device) in devices.enumerate() {
                println!("    {}: {}", i, device.name().unwrap_or("Unknown".into()));
            }
        } else {
            println!("    (Failed to list input devices)");
        }
    }
}

