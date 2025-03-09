use std::{os::linux::raw, sync::Mutex, thread, time::Duration};

use crate::{
    ai_stuff::{get_assistant_model, get_translation_model},
    bus_stuff::MetaResult,
    keyboard::write_text,
};
use enigo::{Enigo, Key, Keyboard, Mouse, Settings};
use rig::{completion::Prompt, providers::cohere::Meta};
use sc_macros::create_grammar;
use tokio::runtime::Handle;

use once_cell::sync::Lazy;
use uinput::event::Controller;

pub fn translate(input: &str) {
    let input = input.to_string();
    let handle = Handle::current();
    thread::spawn(move || {
        handle.spawn(async move {
            let model = get_translation_model();
            let res = model.prompt(input.as_ref()).await;
            match res {
                Ok(output) => {
                    let output = output.trim();
                    println!("Successful translation: {input} -> {output}");
                    write_text(&output);
                }
                Err(err) => eprintln!("Error prompting the model {err}\n{err:?}"),
            }
        });
    });
}

pub fn assistant(input: &str, enable_thinking: bool) {
    let input = input.to_string();
    let handle = Handle::current();
    thread::spawn(move || {
        handle.spawn(async move {
            let model = get_assistant_model(enable_thinking);
            let res = model.prompt(input.as_ref()).await;
            match res {
                Ok(output) => {
                    let output = output.trim();
                    println!("Successful assistance: \n'{input}' -> '{output}'");
                    write_text(&output);
                }
                Err(err) => eprintln!("Error prompting the model {err}\n{err:?}"),
            }
        });
    });
}

create_grammar!(
    root["cal"] = calendar;
    calendar["show"] = {
        vec![MetaResult{
            name: format!("Calendar"),
            description: format!("Showing calendar"),
            path: format!("cal show"),
            icon: None
        }]
    };
    calendar["remove"] = {
        vec![MetaResult{
            name: format!("Remove"),
            description: format!("Remove calendar"),
            path: format!("cal remove"),
            icon: None
        }]
    };
    root["t"] = translate;
    translate[text] = {
        if !activate {
            return vec![MetaResult{
                name: format!("Translate \"{input}\""),
                description: format!(""),
                path: format!("translate"),
                icon: None
            }]
        }
        translate(input);
        vec![]
    };

    root["a"] = assistant;
    assistant[text] = {
        if !activate {
            return vec![MetaResult{
                name: format!("AI Assistant \"{input}\""),
                description: format!(""),
                path: format!("assistant"),
                icon: None
            }]
        }
        assistant(input, false);
        vec![]
    };
    root["smart"] = solve;
    solve[text] = {
        if !activate {
            return vec![MetaResult{
                name: format!("Smart AI Assistant \"{input}\""),
                description: format!(""),
                path: format!("smart assistant"),
                icon: None
            }]
        }
        assistant(input, true);
        vec![]
    };
);

const PREFIX: &str = "0";
impl Handler {
    pub fn new() -> Self {
        Handler {
            parser: ParseLayer { rules: vec![] },
        }
    }

    pub fn handle(&self, raw_input: String) -> Vec<MetaResult> {
        if !raw_input.starts_with(PREFIX) {
            return vec![];
        }
        let input = &raw_input[PREFIX.len()..];
        if input.trim().is_empty() {
            return vec![];
        }
        layer_root(String::new(), input, false)
    }

    pub fn activate(&self, raw_input: String) {
        if !raw_input.starts_with(PREFIX) {
            dbg!(raw_input);
            panic!("Activated input that does not start with prefix.")
        }

        let input = &raw_input[PREFIX.len()..];
        if input.trim().is_empty() {
            eprintln!("Activated empty input.");
            return;
        }
        let r = layer_root(String::new(), input, true);
        println!("Done activating. {r:?}")
    }
}

struct ParseLayer {
    rules: Vec<(String, Option<ParseLayer>)>,
}

pub struct Handler {
    parser: ParseLayer,
}

fn handle_cal(input: &str) -> Option<MetaResult> {
    let args = ensure_args("cal", input)?;
    Some(res("Calendar", "Woah!", "calender"))
}

fn ensure_args(name: &str, input: &str) -> Option<String> {
    if !input.starts_with(name) {
        None
    } else {
        Some(input[name.len()..].trim().to_owned())
    }
}
fn res(n: &str, d: &str, p: &str) -> MetaResult {
    MetaResult {
        name: format!("{n}"),
        description: format!("{d}"),
        path: format!("{p}"),
        icon: None,
    }
}
