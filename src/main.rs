use bus_stuff::{connect, search_iface, MetaResult};
use logic::Handler;

// #![deny(warnings)]
pub mod bus_stuff;
mod logic;

fn main() {
    println!("Searchcuts started!");
    let mut handler = Handler::new();
    let conn = connect().unwrap();
    let f = dbus::tree::Factory::new_fn();
    let search_interface = search_iface(move |data| -> Option<MetaResult> { handler.handle(data) });

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
