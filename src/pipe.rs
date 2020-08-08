use super::constants;
use super::cpal_utils;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Device;
use ringbuf::RingBuffer;
use std::sync::mpsc::Receiver;
use std::thread;

pub fn start_play_through(receiver: Receiver<usize>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let input_devices = cpal_utils::get_input_devices(&host);
        println!("Available Input Devices ====");
        for (i, device) in input_devices.iter().enumerate() {
            match device.name() {
                Ok(n) => println!("({}) {}", i, n),
                Err(_) => eprintln!("({}) Unknown device", i),
            }
            match device.default_input_config() {
                Ok(f) => println!("--- {:?}", f),
                Err(_) => eprintln!("Couldn't fetch format"),
            }
        }

        let mut index = receiver.recv().unwrap();
        while index >= input_devices.len() {
            let mut options_str = String::new();
            for i in 0..input_devices.len() {
                if i == input_devices.len() - 1 {
                    options_str += &format!("{}", i)[..];
                } else {
                    options_str += &format!("{}, ", i)[..];
                }
            }
            println!("Available options: {}", options_str);
            index = receiver.recv().unwrap();
        }
        let input_device: &Device = &input_devices[index];
        let input_channel_count = match input_device.default_input_config() {
            Ok(f) => f.channels(),
            Err(_) => 0,
        };

        // Fetch output devices
        let output_devices = cpal_utils::get_output_devices(&host);
        println!("Available Output Devices");
        for (i, device) in output_devices.iter().enumerate() {
            match device.name() {
                Ok(n) => println!("({}) {}", i, n),
                Err(_) => eprintln!("({}) Unknown device", i),
            }
            match device.default_output_config() {
                Ok(f) => println!("--- {:?}", f),
                Err(_) => eprintln!("Couldn't fetch format"),
            }
        }

        index = receiver.recv().unwrap();
        while index >= output_devices.len() {
            let mut options_str = String::new();
            for i in 0..output_devices.len() {
                if i == input_devices.len() - 1 {
                    options_str += &format!("{}", i)[..];
                } else {
                    options_str += &format!("{}, ", i)[..];
                }
            }
            println!("Available options: {}", options_str);
            index = receiver.recv().unwrap();
        }
        let output_device: &Device = &output_devices[index];
        let output_channel_count = match output_device.default_output_config() {
            Ok(f) => f.channels(),
            Err(_) => 0,
        };

        let (prod_factor, cons_factor) =
            get_channel_factor(input_channel_count, output_channel_count);

        // println!("{}, {}", prod_factor, cons_factor);
        let ring_buffer = RingBuffer::new(constants::BUFFER_CAPACITY);
        let (mut prod, mut cons) = ring_buffer.split();
        for _ in 0..10 {
            prod.push(0.0).unwrap();
        }

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut output_fell_behind = false;
            for &sample in data {
                for _ in 0..prod_factor {
                    if prod.push(sample).is_err() {
                        output_fell_behind = true;
                    }
                }
            }
            // if output_fell_behind {
            //     eprintln!("Output stream fell behind")
            // }
        };

        let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
        let input_stream = input_device
            .build_input_stream(&config, input_data_fn, err_fn)
            .unwrap();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut input_fell_behind = false;
            for sample in data {
                for _ in 0..cons_factor {
                    *sample = match cons.pop() {
                        Some(s) => s,
                        None => {
                            input_fell_behind = true;
                            0.0
                        }
                    };
                }
            }
            // if input_fell_behind {
            //     eprintln!("Input stream fell behind");
            // }
        };

        let output_stream = output_device
            .build_output_stream(&config, output_data_fn, err_fn)
            .unwrap();

        loop {
            input_stream
                .play()
                .expect("Error while playing input stream");
            output_stream
                .play()
                .expect("Error while playing output stream");
        }
    });
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn get_channel_factor(input_channel_count: u16, output_channel_count: u16) -> (u16, u16) {
    let mut prod_factor = 0;
    let mut cons_factor = 0;
    if input_channel_count == output_channel_count
        || (input_channel_count == 0 || output_channel_count == 0)
    {
        prod_factor = 1;
        cons_factor = 1;
    } else if input_channel_count != output_channel_count {
        if input_channel_count > output_channel_count {
            prod_factor = 1;
            cons_factor = input_channel_count / output_channel_count;
        } else {
            prod_factor = output_channel_count / input_channel_count;
            cons_factor = 1;
        }
    }
    (prod_factor, cons_factor)
}
