use env_logger::{Builder, Target};
use log::{LevelFilter, Record};
use serde::Serialize;
use std::fmt;
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

// Debug - для unwrap и подобных
#[derive(Debug)]
pub enum LogError {
    Common(std::io::Error),
}

// fmt::Display - для возможности конвертации в строку ".to_string()"
impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogError::Common(s) => write!(f, "{}", s),
        }
    }
}

pub fn init(
    service_name: String,
    version: String,
    level: String,
    filepath: Option<String>,
    is_test: bool,
) -> Result<(), LogError> {
    let level: LevelFilter = match level.to_lowercase().as_str() {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };
    let mut builder = Builder::new();

    if filepath.is_some() {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath.unwrap_or_default())
            .map_err(LogError::Common)?;

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
