use dbus::arg::Variant;
use dbus::tree::{Interface, MTFn, MethodErr, MethodInfo, MethodResult};
use dbus::Message;

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Once};

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

pub fn get_initial_resultset(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let terms: Vec<String> = minfo.msg.read1()?;
    eprintln!("GetInitialResultSet terms={:?}", terms);

    // Just use the merged terms as result-id
    let expr = terms.join(" ");
    ctx().lock().unwrap().insert(expr.clone());
    let result_ids = vec![expr];

    let m_return = minfo.msg.method_return().append1(result_ids);
    Ok(vec![m_return])
}

pub fn get_subsearch_resultset(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let (prev, terms): (Vec<String>, Vec<String>) = minfo.msg.read2()?;
    eprintln!("GetSubsearchResultSet prev={:?} terms={:?}", prev, terms);

    // Called to refine search within initial results, nothing we wanna do here
    let expr = terms.join(" ");
    ctx().lock().unwrap().insert(expr.clone());
    let result_ids = vec![expr];

    let m_ret = minfo.msg.method_return().append1(result_ids);
    Ok(vec![m_ret])
}

pub struct MetaResult {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
}

// A map from string to variant
pub type MetasMap = HashMap<String, Variant<String>>;
fn create_get_result_metas(
    handler: impl Fn(String) -> Option<MetaResult>,
) -> impl Fn(&MethodInfo<'_, MTFn, ()>) -> Result<Vec<Message>, MethodErr> {
    let get_result_metas = move |minfo: &MethodInfo<MTFn<()>, ()>| -> MethodResult {
        let ids: Vec<String> = minfo.msg.read1()?;
        eprintln!("GetResultMetas ids={:?}", ids);

        // We're storing the
        let mut metas = Vec::new();
        for input in ids {
            let mut meta = MetasMap::new();
            let result = handler(input.clone());
            if let Some(result) = result {
                meta.insert(format!("name"), Variant(result.name));
                meta.insert(format!("id"), Variant(input));
                meta.insert(format!("description"), Variant(result.description));
                if let Some(icon) = result.icon {
                    meta.insert(format!("icon"), Variant(icon));
                }
                metas.push(meta);
            }
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

pub fn search_iface(
    handler: impl Fn(String) -> Option<MetaResult> + 'static,
) -> Interface<MTFn<()>, ()> {
    let f = dbus::tree::Factory::new_fn();
    // DOCS: org.gnome.ShellSearchProvider2.xml
    f.interface("org.gnome.Shell.SearchProvider2", ())
        .add_m(
            f.method("GetInitialResultSet", (), get_initial_resultset)
                .inarg::<Vec<String>, _>("terms")
                .outarg::<Vec<String>, _>("results"),
        )
        .add_m(
            f.method("GetSubsearchResultSet", (), get_subsearch_resultset)
                .inarg::<Vec<String>, _>("previous_results")
                .inarg::<Vec<String>, _>("terms")
                .outarg::<Vec<String>, _>("results"),
        )
        .add_m(
            f.method("GetResultMetas", (), create_get_result_metas(handler))
                .inarg::<String, _>("identifiers")
                .outarg::<Vec<MetasMap>, _>("metas"),
        )
        .add_m(
            f.method("ActivateResult", (), activate_result)
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
