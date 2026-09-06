use std::collections::BTreeSet;
use std::io::Write as _;

use super::*;

fn first_fields() -> Vec<String> {
    let decoded = decode_gzip(ARTIFACT).unwrap();
    csv_record(decoded.lines().nth(1).unwrap()).unwrap()
}

#[test]
fn pinned_cells_preserve_exact_coverage_disclosure_composites_and_provenance() {
    let sectors = michigan_county_sectors_v1().unwrap();
    assert!(std::ptr::eq(sectors, michigan_county_sectors_v1().unwrap()));
    let rows = sectors.rows();
    assert_eq!(rows.len(), 1603);
    assert_eq!(
        rows.iter()
            .map(MichiganCountySectorV1::county_geoid)
            .collect::<BTreeSet<_>>()
            .len(),
        83
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.disclosure() == MichiganSectorDisclosureV1::Suppressed)
            .count(),
        416
    );
    assert_eq!(
        rows.iter()
            .filter(
                |row| row.sector_code().disposition() == MichiganSectorDispositionV1::Unclassified
            )
            .count(),
        81
    );
    assert_eq!(MAX_ROWS - rows.len(), 57);
    assert_eq!(
        rows.iter()
            .map(MichiganCountySectorV1::annual_avg_estabs_count)
            .sum::<u64>(),
        235_170
    );
    assert_eq!(
        rows.iter()
            .filter_map(MichiganCountySectorV1::total_annual_wages)
            .max(),
        Some(10_954_108_416)
    );
    let codes = rows
        .iter()
        .map(|row| row.sector_code().as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(codes, SECTOR_CODES.into_iter().collect());
    let sources = source_pins(SOURCE_MANIFEST).unwrap();
    for row in rows {
        let source = &sources[county_index(row.county_geoid()).unwrap()];
        assert_eq!(row.source_file(), source.file);
        assert_eq!(row.source_sha256(), source.sha256);
        assert!(row.annual_avg_estabs_count() > 0);
        let measures = [
            row.annual_avg_emplvl(),
            row.total_annual_wages(),
            row.annual_avg_wkly_wage(),
        ];
        if row.disclosure() == MichiganSectorDisclosureV1::Suppressed {
            assert_eq!(measures, [None; 3]);
        } else {
            assert!(measures.iter().all(Option::is_some));
        }
    }
    assert_eq!(
        semantic_digest(rows).unwrap(),
        QCEW_SECTORS_SEMANTIC_SHA256_V1
    );
    let contract = include_str!("../../../../../contracts/qcew_county_sectors_v1.yaml");
    assert!(contract.contains(&format!("sha256: {QCEW_SECTORS_ARTIFACT_SHA256_V1}")));
    assert!(contract.contains(&format!(
        "semantic_sha256: {QCEW_SECTORS_SEMANTIC_SHA256_V1}"
    )));
}

#[test]
fn quoted_commas_and_doubled_quotes_preserve_exact_field_content() {
    let line = "26001,31-33,\"Manufacturing, \"\"quoted\"\"\",classified,N,7,,,,\"2024.annual 26001 Alcona County, Michigan.csv\",digest";
    let fields = csv_record(line).unwrap();
    assert_eq!(fields.len(), 11);
    assert_eq!(fields[2], "Manufacturing, \"quoted\"");
    assert_eq!(&fields[6..9], ["", "", ""]);
    assert_eq!(fields[9], "2024.annual 26001 Alcona County, Michigan.csv");
    for malformed in [
        "a,b,c,d,e,f,g,h,i,j,\"unclosed",
        "a,b,c,d,e,f,g,h,i,j,\"closed\"junk",
        "a,b,c,d,e,f,g,h,i,j,unquoted\"quote",
        "a,b,c,d,e,f,g,h,i,j,\"hidden\nline\"",
        "a,b,c,d,e,f,g,h,i,j,k,extra",
        "a,b,c,d,e,f,g,h,i,j",
    ] {
        assert_eq!(csv_record(malformed), Err(MichiganSectorsErrorV1::CsvShape));
    }
}

#[test]
fn suppressed_metrics_are_absent_while_disclosed_zero_stays_exact() {
    let sources = source_pins(SOURCE_MANIFEST).unwrap();
    let mut fields = first_fields();
    assert_eq!(fields[4], "N");
    let suppressed = parse_row(&fields, &sources).unwrap();
    assert_eq!(suppressed.annual_avg_estabs_count(), 9);
    assert_eq!(suppressed.annual_avg_emplvl(), None);
    fields[6] = "0".into();
    assert_eq!(
        parse_row(&fields, &sources),
        Err(MichiganSectorsErrorV1::Disclosure)
    );
    fields[4].clear();
    assert_eq!(
        parse_row(&fields, &sources),
        Err(MichiganSectorsErrorV1::Value)
    );
    fields[7] = "0".into();
    fields[8] = "0".into();
    let observed_zero = parse_row(&fields, &sources).unwrap();
    assert_eq!(
        observed_zero.disclosure(),
        MichiganSectorDisclosureV1::Disclosed
    );
    assert_eq!(observed_zero.annual_avg_emplvl(), Some(0));
    assert_eq!(observed_zero.total_annual_wages(), Some(0));
    assert_eq!(observed_zero.annual_avg_wkly_wage(), Some(0));
    fields[5] = "0".into();
    assert_eq!(
        parse_row(&fields, &sources),
        Err(MichiganSectorsErrorV1::Value)
    );
}

#[test]
fn identities_provenance_and_exact_integer_domain_refuse_lossy_admission() {
    let sources = source_pins(SOURCE_MANIFEST).unwrap();
    for (column, replacement, error) in [
        (0, "26002", MichiganSectorsErrorV1::RowIdentity),
        (0, "26167", MichiganSectorsErrorV1::RowIdentity),
        (1, "31", MichiganSectorsErrorV1::RowIdentity),
        (1, "331", MichiganSectorsErrorV1::RowIdentity),
        (2, "", MichiganSectorsErrorV1::Value),
        (2, "hidden\ttext", MichiganSectorsErrorV1::Value),
        (3, "unclassified", MichiganSectorsErrorV1::RowIdentity),
        (4, "X", MichiganSectorsErrorV1::Disclosure),
        (9, "other-source.csv", MichiganSectorsErrorV1::Provenance),
        (10, "0000", MichiganSectorsErrorV1::Provenance),
    ] {
        let mut fields = first_fields();
        fields[column] = replacement.into();
        assert_eq!(parse_row(&fields, &sources), Err(error));
    }
    for invalid in [
        "",
        "00",
        "+1",
        "-1",
        "1.5",
        "NaN",
        "１２",
        "9223372036854775808",
    ] {
        assert_eq!(integer(invalid), Err(MichiganSectorsErrorV1::Value));
    }
    assert_eq!(
        integer("9223372036854775807"),
        Ok(9_223_372_036_854_775_807)
    );
    for code in ["31-33", "44-45", "48-49"] {
        let code = MichiganSectorCodeV1::parse(code).unwrap();
        assert_eq!(code.disposition(), MichiganSectorDispositionV1::Classified);
    }
    assert_eq!(
        MichiganSectorCodeV1::parse("99").unwrap().disposition(),
        MichiganSectorDispositionV1::Unclassified
    );
}

#[test]
fn duplicate_reordered_missing_and_changed_cells_do_not_pass_semantic_admission() {
    let source = decode_gzip(ARTIFACT).unwrap();
    let pins = source_pins(SOURCE_MANIFEST).unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    let mut duplicate = lines.clone();
    duplicate.insert(2, lines[1]);
    assert_eq!(
        checked_csv(&(duplicate.join("\n") + "\n"), &pins),
        Err(MichiganSectorsErrorV1::Ordering)
    );
    let mut reordered = lines.clone();
    reordered.swap(1, 2);
    assert_eq!(
        checked_csv(&(reordered.join("\n") + "\n"), &pins),
        Err(MichiganSectorsErrorV1::Ordering)
    );
    let missing = lines
        .iter()
        .filter(|line| !line.starts_with("26001,"))
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(
        checked_csv(&missing, &pins),
        Err(MichiganSectorsErrorV1::Coverage)
    );
    let changed = source.replacen(",N,9,,,,", ",N,10,,,,", 1);
    assert_ne!(changed, source);
    assert_eq!(
        checked_csv(&changed, &pins),
        Err(MichiganSectorsErrorV1::SemanticDigest)
    );
    assert_eq!(
        checked_csv(source.trim_end_matches('\n'), &pins),
        Err(MichiganSectorsErrorV1::CsvShape)
    );
}

fn gzip(source: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), flate2::Compression::fast());
    encoder.write_all(source).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn gzip_limits_headers_crc_and_trailing_members_are_checked() {
    assert_eq!(decode_gzip(&gzip(b"bounded\n")), Ok("bounded\n".into()));
    assert_eq!(
        decode_gzip(&vec![0; MAX_BYTES + 1]),
        Err(MichiganSectorsErrorV1::ArtifactSize)
    );
    assert_eq!(
        decode_gzip(&gzip(&vec![b'a'; MAX_BYTES + 1])),
        Err(MichiganSectorsErrorV1::ArtifactSize)
    );
    let mut timestamp = gzip(b"bounded\n");
    timestamp[4] = 1;
    assert_eq!(
        decode_gzip(&timestamp),
        Err(MichiganSectorsErrorV1::ArtifactDecode)
    );
    let mut crc = gzip(b"bounded\n");
    let checksum = crc.len() - 8;
    crc[checksum] ^= 1;
    assert_eq!(
        decode_gzip(&crc),
        Err(MichiganSectorsErrorV1::ArtifactDecode)
    );
    let mut trailing = gzip(b"bounded\n");
    trailing.extend(gzip(b"extra\n"));
    assert_eq!(
        decode_gzip(&trailing),
        Err(MichiganSectorsErrorV1::ArtifactDecode)
    );
    let mut truncated = gzip(b"bounded\n");
    truncated.pop();
    assert_eq!(
        decode_gzip(&truncated),
        Err(MichiganSectorsErrorV1::ArtifactDecode)
    );
}

#[test]
fn public_admission_pins_source_manifest_before_parsing_and_exact_compressed_bytes() {
    let mut source = SOURCE_MANIFEST.to_vec();
    source.push(b' ');
    assert_eq!(
        admit(b"not gzip", &source),
        Err(MichiganSectorsErrorV1::SourceDigest)
    );
    assert_eq!(
        admit(b"not gzip", SOURCE_MANIFEST),
        Err(MichiganSectorsErrorV1::ArtifactDigest)
    );
    let mut raw = ARTIFACT.to_vec();
    raw.push(0);
    assert_eq!(
        admit(&raw, SOURCE_MANIFEST),
        Err(MichiganSectorsErrorV1::ArtifactDigest)
    );
}

#[test]
fn semantic_encoding_matches_python_ascii_json_exact_integers_and_nulls() {
    let mut encoded = String::new();
    ascii_json_line(
        &mut encoded,
        &("Café, \"𝄞\"", None::<u64>, 0, 10_954_108_416_u64),
    )
    .unwrap();
    assert_eq!(
        encoded,
        "[\"Caf\\u00e9, \\\"\\ud834\\udd1e\\\"\",null,0,10954108416]\n"
    );
}
