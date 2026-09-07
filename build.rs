fn main() -> Result<(), std::io::Error> {
    prost_build::compile_protos(&["api/basis_service.proto"], &["api"])?;
    Ok(())
}
