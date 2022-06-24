use cpal::traits::HostTrait;
use cpal::{Device, Devices, DevicesError, Host};
use std::iter::Filter;

type DeviceResult = Result<Filter<Devices, fn(&Device) -> bool>, DevicesError>;

pub fn get_input_devices(host: &Host) -> Vec<Device> {
    let r = host.input_devices();

    filter_devices(r)
}

pub fn get_output_devices(host: &Host) -> Vec<Device> {
    let r = host.output_devices();

    filter_devices(r)
}

fn filter_devices(devices: DeviceResult) -> Vec<Device> {
    let mut filtered_devices: Vec<Device> = vec![];
    match devices {
        Ok(ds) => {
            for d in ds {
                filtered_devices.push(d);
            }
        }
        Err(error) => eprintln!("Devices error: {}", error),
    }

    filtered_devices
}
