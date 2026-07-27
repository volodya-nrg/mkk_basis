mod adapter;
mod transport;
mod usecase;

use adapter::{Config, Postgres, logger};
use transport::http_server::HTTPServer;
use usecase::UseCase;

fn main() {
    let config_filepath: String = String::from("/");
    if let Err(e) = run(&config_filepath) {
        println!("{e}");
        std::process::exit(1);
    }
}

fn run(config_filepath: &String) -> Result<(), String> {
    let cfg: Config = Config::new(config_filepath);
    logger::init(&cfg.service_name, &cfg.version, &cfg.level);
    let postgres: Postgres = Postgres::new(&String::from("dsn"));
    
    let use_case: UseCase = UseCase::new(postgres);
    let http_server: HTTPServer = HTTPServer::new(use_case);

    Ok(())
}
