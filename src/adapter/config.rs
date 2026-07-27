pub struct Config {
    pub service_name: String,
    pub version: String,
    pub level: String,
}

impl Config {
    pub fn new(filepath: &String) -> Self {
        Self {
            service_name: String::from("default service_name"),
            version: String::from("v0.0.1"),
            level: String::from("debug"),
        }
    }
}
