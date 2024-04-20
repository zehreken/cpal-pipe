use super::constants;
use super::cpal_utils;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Device;
use cpal::StreamConfig;
use ringbuf::RingBuffer;
use std::sync::mpsc::Receiver;
use std::thread;

pub fn start_play_through(receiver: Receiver<usize>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let input_devices = cpal_utils::get_input_devices(&host);
        let msg = constants::PURPLE_FILL.to_owned() + "Available Input Devices" + constants::RESET;
        println!("{}", msg);
        for (i, device) in input_devices.iter().enumerate() {
            match device.name() {
                Ok(name) => println!("({}) {}", i, name),
                Err(_) => eprintln!("({}) Unknown device", i),
            }
            match device.default_input_config() {
                Ok(f) => println!(" -- {:?}", f),
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

        // Set buffer size, what we set here is the actual buffer size, on the hardware
        const BUFFER_SIZE: u32 = 512;
        const RINGBUFFER_SIZE: usize = 32 * 4 * 8;

        // Fetch output devices
        let output_devices = cpal_utils::get_output_devices(&host);
        let msg = constants::PURPLE_FILL.to_owned() + "Available Output Devices" + constants::RESET;
        println!("{}", msg);
        for (i, device) in output_devices.iter().enumerate() {
            match device.name() {
                Ok(name) => println!("({}) {}", i, name),
                Err(_) => eprintln!("({}) Unknown device", i),
            }
            match device.default_output_config() {
                Ok(f) => println!(" -- {:?}", f),
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

        println!("Running pipe...");
        let msg =
            constants::RED_FILL.to_owned() + "To quit, press 'q' and then enter" + constants::RESET;
        println!("{}", msg);

        let ring_buffer = RingBuffer::new(RINGBUFFER_SIZE);
        let (mut producer, mut consumer) = ring_buffer.split();

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for &sample in data {
                let r = producer.push(sample);
                match r {
                    Ok(_) => {}
                    Err(_) => eprintln!("Buffer overrun error, output stream is behind"),
                }
            }
        };

        // let input_config: cpal::StreamConfig = cpal::StreamConfig {
        //     channels: 2,
        //     sample_rate: cpal::SampleRate(48000),
        //     buffer_size: BufferSize::Fixed(BUFFER_SIZE),
        // };
        let input_config: StreamConfig = input_device.default_input_config().unwrap().into();
        let input_stream = input_device
            .build_input_stream(&input_config, input_data_fn, err_fn, None)
            .unwrap();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data {
                *sample = consumer.pop().unwrap_or(0.0);
            }
            consumer.discard(RINGBUFFER_SIZE);
        };

        let output_config = output_device.default_output_config().unwrap().into();
        let output_stream = output_device
            .build_output_stream(&output_config, output_data_fn, err_fn, None)
            .unwrap();

        loop {
            input_stream
                .play()
                .expect("Error while playing input stream");
            output_stream
                .play()
                .expect("Error while playing output stream");

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
