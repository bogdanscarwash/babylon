use super::{hex, refused, report::Candidate, Result};
use babylon_kernel::content_digest::sha256_of;
use babylon_persistence::michigan_material::{
    MichiganPhysicalNetwork, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES,
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_REPORT_BYTES: usize = 64 * 1024 * 1024;
const USAGE: &str = "statewide_experiment --defines PATH --qualification DECODED_JSON --physical JSON --capacity-key KEY --food-process KEY --constrained-grams POSITIVE --shortage-opening NONNEGATIVE --output NEW_ABSOLUTE_JSON";
pub struct Arguments {
    pub defines: PathBuf,
    pub qualification: PathBuf,
    pub physical: PathBuf,
    pub candidate: Candidate,
    pub output: PathBuf,
}
pub struct Inputs {
    pub defines: String,
    pub qualification: Vec<u8>,
    pub physical: MichiganPhysicalNetwork,
    pub hashes: BTreeMap<&'static str, String>,
}
impl Arguments {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let values: Vec<_> = args.collect();
        if values == ["--help"] {
            println!("{USAGE}");
            return Ok(None);
        }
        let mut fields = BTreeMap::new();
        for pair in values.chunks(2) {
            if pair.len() != 2
                || !matches!(
                    pair[0].as_str(),
                    "--defines"
                        | "--qualification"
                        | "--physical"
                        | "--capacity-key"
                        | "--food-process"
                        | "--constrained-grams"
                        | "--shortage-opening"
                        | "--output"
                )
            {
                return Err(refused(USAGE));
            }
            if fields.insert(pair[0].clone(), pair[1].clone()).is_some() {
                return Err(refused(format!("duplicate argument {}", pair[0])));
            }
        }
        let mut take = |key: &str| {
            fields
                .remove(key)
                .ok_or_else(|| refused(format!("missing {key}; {USAGE}")))
        };
        let result = Self {
            defines: take("--defines")?.into(),
            qualification: take("--qualification")?.into(),
            physical: take("--physical")?.into(),
            candidate: Candidate {
                capacity_key: take("--capacity-key")?,
                food_process: take("--food-process")?,
                constrained_grams: take("--constrained-grams")?.parse()?,
                shortage_opening: take("--shortage-opening")?.parse()?,
            },
            output: take("--output")?.into(),
        };
        if !result.output.is_absolute() {
            return Err(refused("output must be an absolute new file path"));
        }
        match std::fs::symlink_metadata(&result.output) {
            Ok(_) => return Err(refused("refusing to overwrite existing output")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(Some(result))
    }
}
fn bounded_read(path: &Path) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?
        .take((MAX_MICHIGAN_CAPTURED_CONTENT_BYTES + 1) as u64)
        .read_to_end(&mut data)?;
    if data.len() > MAX_MICHIGAN_CAPTURED_CONTENT_BYTES {
        return Err(refused(format!(
            "input exceeds captured-content bound: {}",
            path.display()
        )));
    }
    Ok(data)
}
pub fn read_inputs(args: &Arguments) -> Result<Inputs> {
    let defines = bounded_read(&args.defines)?;
    let qualification = bounded_read(&args.qualification)?;
    let physical_bytes = bounded_read(&args.physical)?;
    let physical: MichiganPhysicalNetwork = serde_json::from_slice(&physical_bytes)?;
    if physical.source.pbf_url.starts_with("synthetic://") {
        return Err(refused(
            "synthetic networks are restricted to the focused example tests",
        ));
    }
    Ok(Inputs {
        hashes: BTreeMap::from([
            ("defines", hex(&sha256_of(&defines))),
            ("qualification", hex(&sha256_of(&qualification))),
            ("physical", hex(&sha256_of(&physical_bytes))),
        ]),
        defines: String::from_utf8(defines)?,
        qualification,
        physical,
    })
}
struct BoundedOutput(Vec<u8>);
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self
            .0
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_REPORT_BYTES)
        {
            return Err(std::io::Error::other("operator report exceeds 64 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub fn write_report(path: &Path, report: &impl Serialize) -> Result<()> {
    let mut bytes = BoundedOutput(Vec::new());
    serde_json::to_writer(&mut bytes, report)?;
    bytes.write_all(b"\n")?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes.0)?;
    file.sync_all()?;
    Ok(())
}
