use std::os::linux::raw;

use sc_macros::create_grammar;

use crate::bus_stuff::MetaResult;

// Syntax: <required> [optional] multiple*
// <prefix><app> [args]*
create_grammar!(
    root["cal"] = calendar;
    calendar["show"] = {
        println!("SHOWING CALENDAR");
        Some(MetaResult{
            name: format!("Calendar"),
            description: format!("Showing calendar"),
            icon: None
        })
    };
);

const PREFIX: &str = "0";
impl Handler {
    pub fn new() -> Self {
        Handler {
            parser: ParseLayer { rules: vec![] },
        }
    }

    pub fn handle(&self, raw_input: String) -> Option<MetaResult> {
        dbg!(&raw_input);
        if !raw_input.starts_with(PREFIX) {
            return None;
        }
        let input = &raw_input[PREFIX.len()..];
        if input.trim().is_empty() {
            return None;
        }
        layer_root(input)
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
    Some(res("Calendar", "Woah!"))
}

fn ensure_args(name: &str, input: &str) -> Option<String> {
    if !input.starts_with(name) {
        None
    } else {
        Some(input[name.len()..].trim().to_owned())
    }
}
fn res(n: &str, d: &str) -> MetaResult {
    MetaResult {
        name: format!("{n}"),
        description: format!("{d}"),
        icon: None,
    }
}
