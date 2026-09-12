//! Shared test-only synthetic scale fixture; no real infrastructure claim.

use babylon_persistence::michigan_material::{MichiganMaterialCatalog, MichiganPhysicalNetwork};
use serde::Deserialize;
use std::io::Read;

const CANONICAL_DEFINES: &str =
    include_str!("../../../../../content/scenarios/michigan/defines.toml");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticFixture {
    pub scope: String,
    pub qualification: serde_json::Value,
    pub physical: MichiganPhysicalNetwork,
}

pub fn load() -> SyntheticFixture {
    let encoded = include_bytes!("statewide_synthetic.json.gz");
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(encoded.as_slice())
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(bytes.len() <= 1_048_576);
    let fixture: SyntheticFixture = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        fixture.scope,
        "synthetic-scale-only; not real infrastructure qualification"
    );
    assert!(fixture.physical.source.pbf_url.starts_with("synthetic://"));
    assert_eq!(fixture.physical.terminals.len(), 83);
    let defines_hash = hex_digest(CANONICAL_DEFINES.as_bytes());
    assert_eq!(
        fixture.qualification["defines_sha256"].as_str(),
        Some(defines_hash.as_str()),
        "canonical defines changed: regenerate the test-only synthetic fixture",
    );
    assert_eq!(
        fixture.physical.terminal_source_pins.get("defines_sha256"),
        Some(&defines_hash)
    );
    fixture
}

pub fn catalog() -> MichiganMaterialCatalog {
    let fixture = load();
    let qualification = serde_json::to_vec(&fixture.qualification).unwrap();
    MichiganMaterialCatalog::from_statewide_qualification(
        CANONICAL_DEFINES,
        &qualification,
        fixture.physical,
        Vec::new(),
    )
    .expect("all real roster owners must compile on the explicitly synthetic network")
}

/// Owned sibling files for the real fresh-campaign loader, with synthetic interventions only.
pub struct SyntheticSources {
    directory: std::path::PathBuf,
}

impl SyntheticSources {
    pub fn create() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
        let directory = loop {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let candidate = std::env::temp_dir().join(format!(
                "babylon-statewide-synthetic-{}-{sequence}",
                std::process::id()
            ));
            match std::fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create exact synthetic fixture directory: {error}"),
            }
        };
        let sources = Self { directory };
        sources.write_files();
        sources
    }

    pub fn path(&self, name: &str) -> std::path::PathBuf {
        self.directory.join(name)
    }

    fn write_files(&self) {
        let mut fixture = load();
        let food = fixture.qualification["processes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["family"] == "prepared_food")
            .unwrap();
        let process = format!(
            "{}-{}-prepared_food",
            food["county_geoid"].as_str().unwrap(),
            food["sector_code"].as_str().unwrap()
        );
        let defines = synthetic_defines(CANONICAL_DEFINES, &process);
        let defines_hash = hex_digest(defines.as_bytes());
        // The qualifier does not read interventions: native recipes, finite
        // orders and all relationships remain exactly its original output.
        fixture.qualification["defines_sha256"] = defines_hash.clone().into();
        fixture
            .physical
            .terminal_source_pins
            .insert("defines_sha256".to_owned(), defines_hash.clone());
        let qualification = gzip_json(&fixture.qualification);
        let physical = gzip_json(&fixture.physical);
        let manifest = serde_json::json!({
            "schema": "MichiganStatewideSourcesV1",
            "defines_sha256": defines_hash,
            "qualification_sha256": hex_digest(&qualification),
            "physical_network_sha256": hex_digest(&physical),
        });
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        for (name, bytes) in [
            ("defines.toml", defines.as_bytes()),
            ("statewide-qualification.json.gz", qualification.as_slice()),
            ("statewide-physical.json.gz", physical.as_slice()),
            ("statewide-sources.json", manifest_bytes.as_slice()),
        ] {
            std::fs::write(self.path(name), bytes).unwrap();
        }
    }
}

impl Drop for SyntheticSources {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.directory) {
            eprintln!(
                "synthetic fixture cleanup failed at {}: {error}",
                self.directory.display()
            );
        }
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut text = String::new();
    for byte in babylon_kernel::content_digest::sha256_of(bytes) {
        write!(&mut text, "{byte:02x}").unwrap();
    }
    text
}

fn gzip_json(value: &impl serde::Serialize) -> Vec<u8> {
    use std::io::Write;
    let mut writer = flate2::GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), flate2::Compression::default());
    writer
        .write_all(&serde_json::to_vec(value).unwrap())
        .unwrap();
    writer.finish().unwrap()
}

fn synthetic_defines(canonical: &str, process: &str) -> String {
    let mut defines: toml::Value = toml::from_str(canonical).unwrap();
    let experiment = toml::Value::Table(
        [
            (
                "FREIGHT_CAPACITY_KEY".to_owned(),
                toml::Value::String("synthetic-shared-road".to_owned()),
            ),
            (
                "CONSTRAINED_GRAMS_PER_PERIOD".to_owned(),
                toml::Value::Integer(50_000_000),
            ),
            (
                "FOOD_PROCESS_KEY".to_owned(),
                toml::Value::String(process.to_owned()),
            ),
            (
                "PACKAGING_GOOD_KEY".to_owned(),
                toml::Value::String("paper_packaging".to_owned()),
            ),
            ("SHORTAGE_OPENING_UNITS".to_owned(), toml::Value::Integer(0)),
        ]
        .into_iter()
        .collect(),
    );
    // Replacing the table also works once real intervention values are frozen.
    // Other canonical values retain their parsed meaning; only formatting changes.
    defines["statewide"]
        .as_table_mut()
        .unwrap()
        .insert("EXPERIMENT".to_owned(), experiment);
    toml::to_string(&defines).unwrap()
}

#[test]
fn synthetic_experiment_replaces_an_existing_table_and_preserves_other_values() {
    let mut before: toml::Value = toml::from_str(CANONICAL_DEFINES).unwrap();
    before["statewide"].as_table_mut().unwrap().insert(
        "EXPERIMENT".to_owned(),
        toml::Value::Table(
            [(
                "FREIGHT_CAPACITY_KEY".to_owned(),
                toml::Value::String("preexisting-real-network".to_owned()),
            )]
            .into_iter()
            .collect(),
        ),
    );
    let encoded = synthetic_defines(&toml::to_string(&before).unwrap(), "synthetic-food-process");
    let mut after: toml::Value = toml::from_str(&encoded).unwrap();
    let actual = after["statewide"]
        .as_table_mut()
        .unwrap()
        .remove("EXPERIMENT")
        .unwrap();
    before["statewide"]
        .as_table_mut()
        .unwrap()
        .remove("EXPERIMENT");
    assert_eq!(after, before, "all non-experiment canonical values survive");
    assert_eq!(
        actual["FREIGHT_CAPACITY_KEY"].as_str(),
        Some("synthetic-shared-road")
    );
    assert_eq!(
        actual["FOOD_PROCESS_KEY"].as_str(),
        Some("synthetic-food-process")
    );
    assert_eq!(
        synthetic_defines(&encoded, "synthetic-food-process"),
        encoded
    );
}
