fn main() -> Result<(), std::io::Error> {
    prost_build::Config::new()
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(clippy::derive_partial_eq_without_eq)]")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]")
        .type_attribute("mkk_basis_service.v1.RequestRegisterConfirmQuery", "#[derive(utoipa::IntoParams)]\n#[into_params(parameter_in = Query)]")
        .compile_protos(&["api/basis_service.proto"], &["api"])?;
    Ok(())
}
