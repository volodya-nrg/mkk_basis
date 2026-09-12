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

pub fn init(
    service_name: String,
    version: String,
    level: &str,
    filepath: Option<String>,
    is_test: bool,
) -> Result<(), String> {
    let level: LevelFilter = match level.to_lowercase().as_str() {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };
    let mut builder = Builder::new();

    if let Some(v) = filepath {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(v.clone())
            .map_err(|e| format!("failed to open filepath({v}): {e}"))?;

        builder.target(Target::Pipe(Box::new(log_file)));
    }

    builder
        .filter(None, level)
        .format(move |buf, record: &Record| {
            let json_string = serde_json::to_string(&LogEntry {
                level: record.level().to_string(),
                service_name: service_name.clone(),
                version: version.clone(),
                message: record.args().to_string(),
                target: record.target().to_string(),
            })
            .unwrap_or_else(|_| record.args().to_string());
            writeln!(buf, "{}", json_string).unwrap_or_default();
            Ok(())
        })
        .is_test(is_test)
        .init();

    Ok(())
}
