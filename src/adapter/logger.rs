use env_logger::{Builder, Target};
use log::{LevelFilter, Record};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize)]
struct LogEntry {
    level: String,
    message: String,
    target: String,
    service_name: String,
    version: String,
}

pub fn init(service_name: &str, version: &str, level: &str, filepath: &str, is_test: bool) -> Result<(), String> {
    let level: LevelFilter = match level.to_lowercase().as_str() {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };
    let service_name_loc = service_name.to_string();
    let version_loc = version.to_string();
    let mut builder = Builder::new();

    if filepath != "" {
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filepath)
            .map_err(|e| format!("failed to init file: {e}"))?;

        builder.target(Target::Pipe(Box::new(log_file)));
    }

    builder
        .filter(None, level)
        .format(move |buf, record: &Record| {
            let log_entry = LogEntry {
                level: record.level().to_string(),
                service_name: service_name_loc.to_string(),
                version: version_loc.to_string(),
                message: record.args().to_string(),
                target: record.target().to_string(),
            };

            if let Ok(json_str) = serde_json::to_string(&log_entry) {
                Ok(writeln!(buf, "{}", json_str).unwrap_or_default())
            } else {
                Ok(writeln!(buf, "{}", record.args()).unwrap_or_default())
            }
        })
        .is_test(is_test)
        .init();

    Ok(())
}
