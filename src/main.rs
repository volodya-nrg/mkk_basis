fn main() {
    if let Err(e) = run("".to_string()) {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn run(config_filepath: String) -> Result<(), std::io::Error> {
    // let cfg = Config::new("");

    Ok(())
}