use std::{thread, time::Duration};

use arboard::Clipboard;
use uinput::event::{
    keyboard::{self, Key},
    Keyboard,
};

pub fn is_shifted(c: char) -> bool {
    match c {
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O'
        | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | ')' | '!' | '@'
        | '#' | '$' | '%' | '^' | '&' | '*' | '(' | '~' | '?' | '<' | '>' | '_' | ':' | '{'
        | '}' | '+' | '|' | '"' => true,
        _ => false,
    }
}
pub fn get_keybd_key(c: char) -> Option<Key> {
    match c {
        ' ' => Some(Key::Space),
        'A' | 'a' => Some(Key::A),
        'B' | 'b' => Some(Key::B),
        'C' | 'c' => Some(Key::C),
        'D' | 'd' => Some(Key::D),
        'E' | 'e' => Some(Key::E),
        'F' | 'f' => Some(Key::F),
        'G' | 'g' => Some(Key::G),
        'H' | 'h' => Some(Key::H),
        'I' | 'i' => Some(Key::I),
        'J' | 'j' => Some(Key::J),
        'K' | 'k' => Some(Key::K),
        'L' | 'l' => Some(Key::L),
        'M' | 'm' => Some(Key::M),
        'N' | 'n' => Some(Key::N),
        'O' | 'o' => Some(Key::O),
        'P' | 'p' => Some(Key::P),
        'Q' | 'q' => Some(Key::Q),
        'R' | 'r' => Some(Key::R),
        'S' | 's' => Some(Key::S),
        'T' | 't' => Some(Key::T),
        'U' | 'u' => Some(Key::U),
        'V' | 'v' => Some(Key::V),
        'W' | 'w' => Some(Key::W),
        'X' | 'x' => Some(Key::X),
        'Y' | 'y' => Some(Key::Y),
        'Z' | 'z' => Some(Key::Z),
        '0' | ')' => Some(Key::_0),
        '1' | '!' => Some(Key::_1),
        '2' | '@' => Some(Key::_2),
        '3' | '#' => Some(Key::_3),
        '4' | '$' => Some(Key::_4),
        '5' | '%' => Some(Key::_5),
        '6' | '^' => Some(Key::_6),
        '7' | '&' => Some(Key::_7),
        '8' | '*' => Some(Key::_8),
        '9' | '(' => Some(Key::_9),
        '`' | '~' => Some(Key::Grave),
        '/' | '?' => Some(Key::Slash),
        ',' | '<' => Some(Key::Comma),
        '.' | '>' => Some(Key::Dot),
        '-' | '_' => Some(Key::Minus),
        ';' | ':' => Some(Key::SemiColon),
        '[' | '{' => Some(Key::LeftBrace),
        ']' | '}' => Some(Key::RightBrace),
        '=' | '+' => Some(Key::Equal),
        '\\' | '|' => Some(Key::BackSlash),
        '\'' | '"' => Some(Key::Apostrophe),
        '\n' => Some(Key::Enter),
        _ => None,
    }
}

pub fn write_text(text: &str) {
    let text = text.trim();
    if !text.contains("\n") {
        return keyboard(text);
    }
    let mut clipboard = Clipboard::new().unwrap();
    let contents = clipboard.get_text().unwrap_or_default();
    // println!("Clipboard contents: {contents}");
    clipboard.set_text(text).unwrap();
    send_paste();
    clipboard.set_text(contents).unwrap();
}

fn send_paste() {
    let mut device = uinput::default()
        .unwrap()
        .name("SearchCuts")
        .unwrap()
        .event(uinput::event::Keyboard::All)
        .unwrap()
        .create()
        .unwrap();
    thread::sleep(Duration::from_millis(50));
    device.press(&Key::LeftControl);
    device.click(&Key::V);
    device.release(&Key::LeftControl);
    device.synchronize().unwrap();
}
fn keyboard(text: &str) {
    let mut device = uinput::default()
        .unwrap()
        .name("SearchCuts")
        .unwrap()
        .event(uinput::event::Keyboard::All)
        .unwrap()
        .create()
        .unwrap();

    thread::sleep(Duration::from_millis(50));
    for c in text.chars() {
        // println!("'{c}': {}", c as u8);
        let key = get_keybd_key(c).expect("No key found");
        let shift = is_shifted(c);
        if shift {
            let _ = device.press(&Key::LeftShift);
        }

        let r = device.click(&key);
        match r {
            Ok(_) => (),
            Err(e) => eprintln!("Error sending key {e}"),
        }

        if shift {
            let _ = device.release(&Key::LeftShift);
        }
        // thread::sleep(Duration::from_millis(50));
        // let r = device.release(key);
        // match r {
        //     Ok(_) => (),
        //     Err(e) => eprintln!("Error releasing key {e}"),
        // }
    }
    device.synchronize().unwrap();
}
