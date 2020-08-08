use std::sync::mpsc;
mod constants;
mod cpal_utils;
mod pipe;

fn main() {
    let (sender, receiver) = mpsc::channel::<usize>();
    pipe::start_play_through(receiver); // return the JoinHandle

    // This is the blocking thread and also the main
    let mut buf = String::new();
    'key: loop {
        if buf == "quit" {
            break 'key;
        } else if buf == "0" {
            sender.send(0).unwrap();
            buf.clear();
        } else if buf == "1" {
            sender.send(1).unwrap();
            buf.clear();
        } else if buf == "2" {
            sender.send(2).unwrap();
            buf.clear();
        } else {
            buf.clear();
            std::io::stdin().read_line(&mut buf).unwrap();
            buf.remove(buf.len() - 1);
        }
    }
}
