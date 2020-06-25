use console::style;
use console::Key;

fn main() {
    let term = console::Term::stdout();
    
    let mut res_key;
    'running: loop {
        res_key = term.read_key();
        match res_key.unwrap() {
            Key::Char(c) => test_key(c),
            Key::Enter => println!("enter"),
            Key::Backspace => println!("backspace"),
            Key::Escape => break 'running,
            _ => {},
        }
    } // 'running
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
