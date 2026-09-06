//! Explicit shipped-content fixture for native unit tests only.

pub(crate) fn catalog() -> crate::michigan_material::MichiganMaterialCatalogV1 {
    crate::michigan_material::MichiganMaterialCatalogV1::from_defines_toml(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../content/scenarios/michigan/defines.toml"
    )))
    .expect("shipped numeric material fixture")
}
