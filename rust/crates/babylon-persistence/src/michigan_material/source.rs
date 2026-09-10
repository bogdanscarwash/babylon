//! Fresh statewide campaigns capture qualified files beside the canonical TOML.
//! Reopening uses the captured catalog and never enters this module.

use std::{io::Read, path::Path};

use super::{
    MichiganCapacityOverrideV2, MichiganDeliveryPresetV1, MichiganInterventionV2,
    MichiganMaterialCatalogV1, MichiganMaterialErrorV1, MichiganOpeningStockOverrideV2,
    MichiganPhysicalNetworkV2, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2,
};
use crate::michigan_defines::{
    MichiganDefinesErrorV1, MichiganDefinesV3, MAX_MICHIGAN_DEFINES_BYTES,
};
use babylon_kernel::sha256_of;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatewideSources {
    schema: String,
    defines_sha256: String,
    qualification_sha256: String,
    physical_network_sha256: String,
}

fn bounded_bytes(path: &Path, bound: usize) -> Result<Vec<u8>, MichiganDefinesErrorV1> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(MichiganDefinesErrorV1::Read)?
        .take(
            u64::try_from(bound + 1)
                .map_err(|_| MichiganDefinesErrorV1::Material(MichiganMaterialErrorV1::Bound))?,
        )
        .read_to_end(&mut bytes)
        .map_err(MichiganDefinesErrorV1::Read)?;
    if bytes.len() > bound {
        return Err(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::Bound,
        ));
    }
    Ok(bytes)
}

fn pinned_gzip(path: &Path, expected: &str) -> Result<Vec<u8>, MichiganDefinesErrorV1> {
    let compressed = bounded_bytes(path, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2)?;
    if crate::michigan_economy::digest_hex(&sha256_of(&compressed)) != expected {
        return Err(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::ArtifactDigest,
        ));
    }
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(compressed.as_slice())
        .take((MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2 + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(MichiganDefinesErrorV1::Read)?;
    if bytes.len() > MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2 {
        return Err(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::Bound,
        ));
    }
    Ok(bytes)
}

pub(super) fn load_statewide(
    path: &Path,
) -> Result<MichiganMaterialCatalogV1, MichiganDefinesErrorV1> {
    let text = String::from_utf8(bounded_bytes(path, MAX_MICHIGAN_DEFINES_BYTES)?)
        .map_err(MichiganDefinesErrorV1::Utf8)?;
    let defines = MichiganDefinesV3::parse(&text)?;
    let experiment = defines
        .statewide
        .experiment
        .as_ref()
        .ok_or(MichiganDefinesErrorV1::Value(
            "statewide interventions are not qualified",
        ))?;
    let directory = path
        .parent()
        .ok_or(MichiganDefinesErrorV1::Value("statewide source directory"))?;
    let manifest: StatewideSources = serde_json::from_slice(&bounded_bytes(
        &directory.join("statewide-sources.json"),
        4096,
    )?)
    .map_err(|_| MichiganDefinesErrorV1::Material(MichiganMaterialErrorV1::ArtifactDecode))?;
    let defines_hash = crate::michigan_economy::digest_hex(&sha256_of(text.as_bytes()));
    if manifest.schema != "MichiganStatewideSourcesV1" || manifest.defines_sha256 != defines_hash {
        return Err(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::ArtifactDigest,
        ));
    }
    let qualification = pinned_gzip(
        &directory.join("statewide-qualification.json.gz"),
        &manifest.qualification_sha256,
    )?;
    let physical: MichiganPhysicalNetworkV2 = serde_json::from_slice(&pinned_gzip(
        &directory.join("statewide-physical.json.gz"),
        &manifest.physical_network_sha256,
    )?)
    .map_err(|_| MichiganDefinesErrorV1::Material(MichiganMaterialErrorV1::ArtifactDecode))?;
    if physical.terminal_source_pins.get("defines_sha256") != Some(&defines_hash) {
        return Err(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::ArtifactDigest,
        ));
    }
    let capacity = MichiganCapacityOverrideV2 {
        capacity_key: experiment.freight_capacity_key.clone(),
        grams_per_period: experiment.constrained_grams_per_period,
    };
    let shortage = MichiganOpeningStockOverrideV2 {
        process_key: experiment.food_process_key.clone(),
        good_key: experiment.packaging_good_key.clone(),
        quantity: experiment.shortage_opening_units,
    };
    let interventions = [
        (
            MichiganDeliveryPresetV1::StatewideFreightConstraint,
            vec![capacity.clone()],
            vec![],
        ),
        (
            MichiganDeliveryPresetV1::StatewidePackagingShortage,
            vec![],
            vec![shortage.clone()],
        ),
        (
            MichiganDeliveryPresetV1::StatewideBoth,
            vec![capacity],
            vec![shortage],
        ),
    ]
    .into_iter()
    .map(
        |(preset, capacities, opening_stocks)| MichiganInterventionV2 {
            preset,
            capacities,
            opening_stocks,
            routes: vec![],
        },
    )
    .collect();
    MichiganMaterialCatalogV1::from_statewide_qualification(
        &text,
        &qualification,
        physical,
        interventions,
    )
}
