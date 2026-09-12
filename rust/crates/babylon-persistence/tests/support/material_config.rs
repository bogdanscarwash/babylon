//! Explicit shipped-content input; the production runtime has no implicit defaults.

pub fn catalog() -> babylon_persistence::michigan_material::MichiganMaterialCatalog {
    babylon_persistence::michigan_material::MichiganMaterialCatalog::from_defines_toml(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/scenarios/michigan/defines.toml"
        )),
    )
    .expect("shipped numeric material fixture")
}
