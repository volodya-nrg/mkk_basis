use log::LevelFilter;
// use std::io::Write;

pub fn init(service_name: String, version: String, level: String) {
    let level: LevelFilter = match level.as_str() {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };

    json_env_logger::builder()
        // .format_module_path(false)
        // .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter(None, level)
        .init();
}
