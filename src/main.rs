use dotenv::dotenv;
use std::{cell::RefCell, rc::Rc};

use bus_stuff::{connect, search_iface, MetaResult};
use dbus::Error;
use searchcuts::Handler;

// #![deny(warnings)]
pub mod ai_stuff;
pub mod bus_stuff;
pub mod keyboard;
mod searchcuts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    serach_cuts();
    Ok(())
}

fn serach_cuts() {
    println!("Searchcuts started!");
    let handler = Rc::new(Handler::new());
    let conn = connect().unwrap();
    let f = dbus::tree::Factory::new_fn();
    let search_interface = search_iface(handler.clone());

    const SEARCH_PATH: &str = "/zip/conner/SearchCuts/SearchProvider";
    let tree = f.tree(()).add(
        f.object_path(SEARCH_PATH, ())
            .introspectable()
            .add(search_interface),
    );
    tree.set_registered(&conn, true).unwrap();
    conn.add_handler(tree);
    loop {
        conn.incoming(1000).next();
    }
}
