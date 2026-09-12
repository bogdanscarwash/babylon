//! Fresh statewide campaigns capture qualified files beside the canonical TOML.
//! Reopening uses the captured catalog and never enters this module.

use std::{io::Read, path::Path};

use super::{
    MichiganCapacityOverride, MichiganDeliveryPreset, MichiganIntervention,
    MichiganMaterialCatalog, MichiganMaterialError, MichiganOpeningStockOverride,
    MichiganPhysicalNetwork, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES,
};
use crate::michigan_defines::{MichiganDefines, MichiganDefinesError, MAX_MICHIGAN_DEFINES_BYTES};
use babylon_kernel::content_digest::sha256_of;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatewideSources {
    schema: String,
    defines_sha256: String,
    qualification_sha256: String,
    physical_network_sha256: String,
}

fn bounded_bytes(path: &Path, bound: usize) -> Result<Vec<u8>, MichiganDefinesError> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(MichiganDefinesError::Read)?
        .take(
            u64::try_from(bound + 1)
                .map_err(|_| MichiganDefinesError::Material(MichiganMaterialError::Bound))?,
        )
        .read_to_end(&mut bytes)
        .map_err(MichiganDefinesError::Read)?;
    if bytes.len() > bound {
        return Err(MichiganDefinesError::Material(MichiganMaterialError::Bound));
    }
    Ok(bytes)
}

fn pinned_gzip(path: &Path, expected: &str) -> Result<Vec<u8>, MichiganDefinesError> {
    let compressed = bounded_bytes(path, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES)?;
    if crate::michigan_economy::digest_hex(&sha256_of(&compressed)) != expected {
        return Err(MichiganDefinesError::Material(
            MichiganMaterialError::ArtifactDigest,
        ));
    }
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(compressed.as_slice())
        .take((MAX_MICHIGAN_CAPTURED_CONTENT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(MichiganDefinesError::Read)?;
    if bytes.len() > MAX_MICHIGAN_CAPTURED_CONTENT_BYTES {
        return Err(MichiganDefinesError::Material(MichiganMaterialError::Bound));
    }
    Ok(bytes)
}

pub(super) fn load_statewide(path: &Path) -> Result<MichiganMaterialCatalog, MichiganDefinesError> {
    let text = String::from_utf8(bounded_bytes(path, MAX_MICHIGAN_DEFINES_BYTES)?)
        .map_err(MichiganDefinesError::Utf8)?;
    let defines = MichiganDefines::parse(&text)?;
    let experiment = defines
        .statewide
        .experiment
        .as_ref()
        .ok_or(MichiganDefinesError::Value(
            "statewide interventions are not qualified",
        ))?;
    let directory = path
        .parent()
        .ok_or(MichiganDefinesError::Value("statewide source directory"))?;
    let manifest: StatewideSources = serde_json::from_slice(&bounded_bytes(
        &directory.join("statewide-sources.json"),
        4096,
    )?)
    .map_err(|_| MichiganDefinesError::Material(MichiganMaterialError::ArtifactDecode))?;
    let defines_hash = crate::michigan_economy::digest_hex(&sha256_of(text.as_bytes()));
    if manifest.schema != "MichiganStatewideSourcesV1" || manifest.defines_sha256 != defines_hash {
        return Err(MichiganDefinesError::Material(
            MichiganMaterialError::ArtifactDigest,
        ));
    }
    let qualification = pinned_gzip(
        &directory.join("statewide-qualification.json.gz"),
        &manifest.qualification_sha256,
    )?;
    let physical: MichiganPhysicalNetwork = serde_json::from_slice(&pinned_gzip(
        &directory.join("statewide-physical.json.gz"),
        &manifest.physical_network_sha256,
    )?)
    .map_err(|_| MichiganDefinesError::Material(MichiganMaterialError::ArtifactDecode))?;
    if physical.terminal_source_pins.get("defines_sha256") != Some(&defines_hash) {
        return Err(MichiganDefinesError::Material(
            MichiganMaterialError::ArtifactDigest,
        ));
    }
    let capacity = MichiganCapacityOverride {
        capacity_key: experiment.freight_capacity_key.clone(),
        grams_per_period: experiment.constrained_grams_per_period,
    };
    let shortage = MichiganOpeningStockOverride {
        process_key: experiment.food_process_key.clone(),
        good_key: experiment.packaging_good_key.clone(),
        quantity: experiment.shortage_opening_units,
    };
    let interventions = [
        (
            MichiganDeliveryPreset::StatewideFreightConstraint,
            vec![capacity.clone()],
            vec![],
        ),
        (
            MichiganDeliveryPreset::StatewidePackagingShortage,
            vec![],
            vec![shortage.clone()],
        ),
        (
            MichiganDeliveryPreset::StatewideBoth,
            vec![capacity],
            vec![shortage],
        ),
    ]
    .into_iter()
    .map(
        |(preset, capacities, opening_stocks)| MichiganIntervention {
            preset,
            capacities,
            opening_stocks,
            routes: vec![],
        },
    )
    .collect();
    MichiganMaterialCatalog::from_statewide_qualification(
        &text,
        &qualification,
        physical,
        interventions,
    )
}
