use dbus::arg::Variant;
use dbus::tree::{Interface, MTFn, MethodErr, MethodInfo, MethodResult};
use dbus::Message;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Mutex, Once};

use crate::searchcuts::Handler;

pub fn connect() -> Result<dbus::Connection, dbus::Error> {
    const BUS_NAME: &str = "zip.conner.SearchCuts.SearchProvider";
    let conn = dbus::Connection::get_private(dbus::BusType::Session)?;
    conn.register_name(BUS_NAME, dbus::NameFlag::ReplaceExisting as u32)?;
    eprintln!("Connected!");
    Ok(conn)
}

pub fn ctx() -> &'static Mutex<HashSet<String>> {
    // map from result-id to context
    static INIT: Once = Once::new();
    static mut CTX: Option<Mutex<HashSet<String>>> = None;
    unsafe {
        INIT.call_once(|| {
            CTX = Some(Mutex::new(HashSet::new()));
        });
        CTX.as_ref().unwrap()
    }
}

pub fn create_get_initial_resultset(
    handler: impl Fn(String) -> Vec<MetaResult>,
) -> impl Fn(&MethodInfo<'_, MTFn, ()>) -> MethodResult {
    move |minfo: &MethodInfo<MTFn<()>, ()>| -> MethodResult {
        let terms: Vec<String> = minfo.msg.read1()?;

        eprintln!("GetInitialResultSet terms={:?}", terms);

        let expr = terms.join(" ");
        let subs_res = handler(expr.clone());
        let mut result_ids = vec![];
        for i in 0..(subs_res.len().min(5)) {
            result_ids.push(format!("{i}-{expr}"));
        }

        let m_return = minfo.msg.method_return().append1(result_ids);
        Ok(vec![m_return])
    }
}
fn create_get_subsearch_resultset(
    handler: impl Fn(String) -> Vec<MetaResult>,
) -> impl Fn(&MethodInfo<'_, MTFn, ()>) -> MethodResult {
    let get_result_metas = move |minfo: &MethodInfo<MTFn<()>, ()>| -> MethodResult {
        let (prev, terms): (Vec<String>, Vec<String>) = minfo.msg.read2()?;
        eprintln!("GetSubsearchResultSet prev={:?} terms={:?}", prev, terms);

        let expr = terms.join(" ");
        let subs_res = handler(expr.clone());
        let mut result_ids = vec![];
        for i in 0..(subs_res.len().min(5)) {
            result_ids.push(format!("{i}-{expr}"));
        }

        let m_ret = minfo.msg.method_return().append1(result_ids);
        Ok(vec![m_ret])
    };
    get_result_metas
}

#[derive(Debug)]
pub struct MetaResult {
    pub name: String,
    pub path: String,
    pub description: String,
    pub icon: Option<String>,
}

// A map from string to variant
pub type MetasMap = HashMap<String, Variant<String>>;
fn create_get_result_metas(
    handler: impl Fn(String) -> Vec<MetaResult>,
) -> impl Fn(&MethodInfo<'_, MTFn, ()>) -> Result<Vec<Message>, MethodErr> {
    let get_result_metas = move |minfo: &MethodInfo<MTFn<()>, ()>| -> MethodResult {
        let ids: Vec<String> = minfo.msg.read1()?;
        eprintln!("GetResultMetas ids={:?}", ids);
        if ids.is_empty() {
            return Ok(vec![]);
        };
        let first = ids.first().unwrap();
        let expr = first.split_once("-").unwrap().1;
        let results = handler(format!("{expr}"));
        // We're storing the
        let mut metas = Vec::new();
        for input in ids {
            let id = input
                .split_once("-")
                .unwrap()
                .0
                .parse::<usize>()
                .expect("ID not an int");
            let result = results.iter().nth(id).expect("Result id not existing!?");
            let mut meta = MetasMap::new();
            meta.insert(format!("name"), Variant(result.name.clone()));
            meta.insert(format!("id"), Variant(input.clone()));
            meta.insert(format!("description"), Variant(result.description.clone()));
            if let Some(icon) = result.icon.clone() {
                meta.insert(format!("icon"), Variant(icon));
            }
            metas.push(meta);
        }

        dbg!(&metas);
        let m_ret = minfo.msg.method_return().append1(metas);
        Ok(vec![m_ret])
    };
    get_result_metas
}
pub fn activate_result(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let (id, terms, ts): (String, Vec<String>, u32) = minfo.msg.read3()?;
    eprintln!("ActivateResult id={} terms={:?} ts={}", id, terms, ts);

    Ok(vec![])
}

pub fn launch_search(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let terms: Vec<String> = minfo.msg.read1()?;
    eprintln!("LaunchSearch terms={:?}", terms);
    Ok(vec![])
}

pub fn search_iface(handler: Rc<Handler>) -> Interface<MTFn<()>, ()> {
    let f = dbus::tree::Factory::new_fn();
    let handler1 = handler.clone();
    let handler2 = handler.clone();
    let handler3 = handler.clone();
    let handler4 = handler.clone();
    // DOCS: org.gnome.ShellSearchProvider2.xml
    f.interface("org.gnome.Shell.SearchProvider2", ())
        .add_m(
            f.method(
                "GetInitialResultSet",
                (),
                create_get_initial_resultset(move |a| handler1.handle(a)),
            )
            .inarg::<Vec<String>, _>("terms")
            .outarg::<Vec<String>, _>("results"),
        )
        .add_m(
            f.method(
                "GetSubsearchResultSet",
                (),
                create_get_subsearch_resultset(move |a| handler2.handle(a)),
            )
            .inarg::<Vec<String>, _>("previous_results")
            .inarg::<Vec<String>, _>("terms")
            .outarg::<Vec<String>, _>("results"),
        )
        .add_m(
            f.method(
                "GetResultMetas",
                (),
                create_get_result_metas(move |a| handler3.handle(a)),
            )
            .inarg::<String, _>("identifiers")
            .outarg::<Vec<MetasMap>, _>("metas"),
        )
        .add_m(
            f.method(
                "ActivateResult",
                (),
                move |minfo: &MethodInfo<MTFn<()>, ()>| -> MethodResult {
                    let (id, terms, ts): (String, Vec<String>, u32) = minfo.msg.read3()?;
                    eprintln!("ActivateResult id={} terms={:?} ts={}", id, terms, ts);
                    let expr = terms.join(" ");
                    handler4.activate(expr);
                    Ok(vec![])
                },
            )
            .inarg::<String, _>("identifier")
            .inarg::<Vec<String>, _>("terms")
            .inarg::<u32, _>("timestamp"),
        )
        .add_m(
            f.method("LaunchSearch", (), launch_search)
                .inarg::<Vec<String>, _>("terms")
                .inarg::<u32, _>("timestamp"),
        )
}
