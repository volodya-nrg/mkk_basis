fn main() -> Result<(), std::io::Error> {
    let mut config = prost_build::Config::new();
    config.type_attribute(
        ".",
        "#[allow(dead_code)]\n
     #[allow(clippy::derive_partial_eq_without_eq)]\n
     #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    [
        "mkk_basis_service.v1.RequestRegisterConfirm",
        "mkk_basis_service.v1.RequestLimitOffset",
        "mkk_basis_service.v1.RequestTaskData",
    ]
    .into_iter()
    .map(query_param_attrs)
    .for_each(|(path, attrs)| {
        config.type_attribute(path, attrs);
    });

    config.compile_protos(&["api/basis_service.proto"], &["api"])?;

    Ok(())
}
const fn query_param_attrs(path: &str) -> (&str, &'static str) {
    (
        path,
        "#[derive(utoipa::IntoParams)]\n#[into_params(parameter_in = Query)]",
    )
}
