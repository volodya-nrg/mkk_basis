fn main() -> Result<(), std::io::Error> {
    prost_build::Config::new()
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["api/basis_service.proto"], &["api"])?;
    Ok(())
}
