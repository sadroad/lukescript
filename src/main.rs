use phf::phf_map;
use rdev::{listen, simulate, EventType, Key};
use std::sync::mpsc::channel;
use std::{thread, time};
mod map;
use map::KEYPRESS_MAP;

const BAD_PHRASES: phf::Map<&'static str, &'static str> = phf_map! {
    "bad word" => "good word",
};

fn main() {
    let (sender, receiver) = channel();
    let _ = thread::spawn(move || {
        listen(move |event| {
            sender
                .send(event)
                .unwrap_or_else(|e| println!("Cloud not send event {:?}", e));
        })
        .expect("Could not listen to events");
    });

    let mut shift_pressed = false;
    let mut chat_open = false;
    let mut buffer = Vec::new();
    for event in receiver.iter() {
        match event.event_type {
            EventType::KeyPress(key) => {
                match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_pressed = true;
                        continue;
                    }
                    _ => (),
                }
                if let Some(name) = match event.name {
                    Some(ref name) => match key {
                        Key::Return => {
                            if chat_open {
                                buffer.clear();
                            } else if shift_pressed {
                                thread::sleep(time::Duration::from_millis(100));
                                for _ in 0..=4 {
                                    _ = simulate(&EventType::KeyPress(Key::Backspace));
                                    _ = simulate(&EventType::KeyRelease(Key::Backspace));
                                }
                            }
                            chat_open = !chat_open;
                            None
                        }
                        Key::Escape => {
                            if chat_open {
                                chat_open = !chat_open;
                                buffer.clear();
                            }
                            None
                        }
                        Key::Space => {
                            buffer.clear();
                            None
                        }
                        Key::Backspace => {
                            buffer.pop();
                            None
                        }
                        _ => {
                            if !name.is_empty() && name.chars().all(|x| x.is_alphanumeric() || x == '/') {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }
                    },
                    None => None,
                } {
                    if chat_open {
                        buffer.push(name);
                    }
                }
            }
            EventType::KeyRelease(key) => match key {
                Key::ShiftLeft | Key::ShiftRight => {
                    shift_pressed = false;
                }
                _ => (),
            },
            _ => (),
        }
        if chat_open {
            let phrase = buffer.join("").to_lowercase();
            if let Some(replacement) = BAD_PHRASES.get(phrase.as_str()) {
                for _ in 0..phrase.len() {
                    _ = simulate(&EventType::KeyPress(Key::Backspace));
                    _ = simulate(&EventType::KeyRelease(Key::Backspace));
                }
                for c in replacement.chars() {
                    let key = *KEYPRESS_MAP.get(&c).unwrap();
                    _ = simulate(&EventType::KeyPress(key));
                    _ = simulate(&EventType::KeyRelease(key));
                }
            }
        }
    }
}
