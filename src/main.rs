use console::style;
use console::Key;
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{Device, Host, StreamData, UnknownTypeInputBuffer, UnknownTypeOutputBuffer};
use ringbuf::RingBuffer;

fn main() {
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    // Fetch input devices
    let input_devices = get_input_devices(&host);
    println!("Available Input Devices");
    for (i, device) in input_devices.iter().enumerate() {
        match device.name() {
            Ok(n) => println!("({}) {}", i, n),
            Err(_) => println!("({}) Unknown device", i),
        }
    }
    let input_device: &Device = &input_devices[0];

    // Fetch output devices
    let output_devices = get_output_devices(&host);
    println!("Available Output Devices");
    for (i, device) in output_devices.iter().enumerate() {
        match device.name() {
            Ok(n) => println!("({}) {}", i, n),
            Err(_) => println!("({}) Unknown device", i),
        }
    }
    let output_device: &Device = &output_devices[0];

    let input_stream_id = event_loop
        .build_input_stream(input_device, &input_device.default_input_format().unwrap())
        .unwrap();

    let output_stream_id = event_loop
        .build_output_stream(
            output_device,
            &output_device.default_output_format().unwrap(),
        )
        .unwrap();

    // event_loop
    //     .play_stream(input_stream_id)
    //     .expect("Failed to play input stream");

    let ring_buffer = RingBuffer::<[f32; 2]>::new(44100);
    let (mut prod, mut cons) = ring_buffer.split();
    for _ in 0..10 {
        prod.push([0.0, 0.0]).unwrap();
    }

    event_loop.run(move |stream_id, stream_result| {
        let stream_data = match stream_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                return;
            }
        };

        match stream_data {
            StreamData::Input {
                buffer: UnknownTypeInputBuffer::F32(buffer),
            } => {
                for elem in buffer.iter() {
                    // println!("{}", elem);
                    // out_elem = *elem;
                    prod.push([*elem, *elem]).unwrap();
                }
            }
            // StreamData::Output {
            //     buffer: UnknownTypeOutputBuffer::U16(mut buffer),
            // } => {
            //     for elem in buffer.iter_mut() {
            //         *elem = u16::max_value() / 2;
            //     }
            // }
            // StreamData::Output {
            //     buffer: UnknownTypeOutputBuffer::I16(mut buffer),
            // } => {
            //     for elem in buffer.iter_mut() {
            //         *elem = sin as i16;
            //     }
            // }
            StreamData::Output {
                buffer: UnknownTypeOutputBuffer::F32(mut buffer),
            } => {
                for elem in buffer.iter_mut() {
                    *elem = match cons.pop() {
                        Some(f) => f[0],
                        None => 0.0,
                    };
                }
            }
            _ => (),
        }
    });

    // println!("{}", host.)
    let term = console::Term::stdout();
    let mut res_key;
    'running: loop {
        res_key = term.read_key();
        match res_key.unwrap() {
            Key::Char(c) => test_key(c),
            Key::Enter => println!("enter"),
            Key::Backspace => println!("backspace"),
            Key::Escape => break 'running,
            _ => {}
        }
    } // 'running

    println!("Quit");
}

fn test_key(c: char) {
    if c == 'a' {
        println!("test a");
    } else if c == 's' {
        println!("test s");
    } else if c == 'A' {
        println!("test A");
    }
}

fn get_input_devices(host: &Host) -> Vec<Device> {
    let r = host.input_devices();
    let mut input_devices: Vec<Device> = vec![];
    match r {
        Ok(devices) => {
            for device in devices {
                input_devices.push(device.into());
            }
        }
        Err(error) => println!("Input devices error: {}", error),
    }

    input_devices
}

fn get_output_devices(host: &Host) -> Vec<Device> {
    let r = host.output_devices();
    let mut output_devices: Vec<Device> = vec![];
    match r {
        Ok(devices) => {
            for device in devices {
                output_devices.push(device.into());
            }
        }
        Err(error) => println!("Output devices error: {}", error),
    }

    output_devices
}

fn create_streams() {}

/*
//! Feeds back the input stream directly into the output stream.
//!
//! Assumes that the input and output devices can use the same stream configuration and that they
//! support the f32 sample format.
//!
//! Uses a delay of `LATENCY_MS` milliseconds in case the default input and output streams are not
//! precisely synchronised.

extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::RingBuffer;

const LATENCY_MS: f32 = 150.0;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    // Default devices.
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    let output_device = host
        .default_output_device()
        .expect("failed to get default output device");
    println!("Using default input device: \"{}\"", input_device.name()?);
    println!("Using default output device: \"{}\"", output_device.name()?);

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = input_device.default_input_config()?.into();

    // Create a delay in case the input and output devices aren't synced.
    let latency_frames = (LATENCY_MS / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;

    // The buffer to share samples
    let ring = RingBuffer::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = None;
        for sample in data {
            *sample = match consumer.pop() {
                Ok(s) => s,
                Err(err) => {
                    input_fell_behind = Some(err);
                    0.0
                }
            };
        }
        if let Some(err) = input_fell_behind {
            eprintln!(
                "input stream fell behind: {:?}: try increasing latency",
                err
            );
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn)?;
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        LATENCY_MS
    );
    input_stream.play()?;
    output_stream.play()?;

    // Run for 3 seconds before closing.
    println!("Playing for 3 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(input_stream);
    drop(output_stream);
    println!("Done!");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
*/
