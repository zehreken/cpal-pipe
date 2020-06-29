use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{Device, Host, StreamData, UnknownTypeInputBuffer, UnknownTypeOutputBuffer};
use ringbuf::RingBuffer;
use std::thread;

fn main() {
    let mut buf = String::from("");
    let input = std::io::stdin().read_line(&mut buf);

    println!("{}", buf);
    start_play_through();
}

fn start_play_through() {
    let handle = thread::spawn(|| {
        let host = cpal::default_host();
        let event_loop = host.event_loop();
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
            .build_input_stream(&input_device, &input_device.default_input_format().unwrap())
            .unwrap();

        let output_stream_id = event_loop
            .build_output_stream(
                &output_device,
                &output_device.default_output_format().unwrap(),
            )
            .unwrap();

        println!("{:?} {:?}", input_stream_id, output_stream_id);

        event_loop
            .play_stream(input_stream_id)
            .expect("Failed to play input stream");

        event_loop
            .play_stream(output_stream_id)
            .expect("Failed to play output stream");

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
                        prod.push([*elem, *elem]).unwrap();
                    }
                }
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
    });

    handle.join().unwrap();
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
        Err(error) => eprintln!("Input devices error: {}", error),
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
        Err(error) => eprintln!("Output devices error: {}", error),
    }

    output_devices
}
