//! Shared language-neutral vectors for the pinned Archive worker behavior.

use babylon_persistence::{
    archive_batch_matches_receipt_v1, archive_contiguous_watermark_v1, classify_archive_receipt_v1,
    classify_archive_sweep_v1, ArchiveDirtyBatchV1, ArchivePageInputV1, ArchiveProducerOutcomeV1,
    ArchiveReceiptDispositionV1, ArchiveReceiptPlanV1, ArchiveSubjectKindV1, ArchiveSubjectV1,
    PendingArchiveReceiptV1, SemanticArchiveErrorV1, ARCHIVE_PENDING_RECEIPTS_SQL_V1,
    ARCHIVE_SWEEP_MAX_RECEIPTS_V1, ARCHIVE_SWEEP_MAX_SCAN_V1, ARCHIVE_SWEEP_WATERMARK_SQL_V1,
};
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

const VECTORS: &str = include_str!("../../../../contracts/archive_worker_v1_vectors.jsonl");
const MAX_ROWS: usize = 32;
const MAX_LINE_BYTES: usize = 16_384;
const TICK_CONTENT_HASH_HEX: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const ALT_CONTENT_HASH_HEX: &str =
    "2222222222222222222222222222222222222222222222222222222222222222";
const SOURCE_PATH: &str = "rust/crates/babylon-persistence/src/archive_worker.rs";
const ERROR_VARIANTS_USED: [&str; 3] = [
    "InvalidVerifiedTick",
    "ReceiptMismatch",
    "StoredPageMismatch",
];

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn hex_decode(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "even hex length");
    text.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let byte = std::str::from_utf8(chunk).expect("ASCII hex");
            u8::from_str_radix(byte, 16).expect("hex byte")
        })
        .collect()
}

fn tick_hash(hex: &str) -> [u8; 32] {
    hex_decode(hex).try_into().expect("exact 32-byte hash")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

fn batch_ref_json(resolve_tick: u64, hash_hex: &str, page_count: usize) -> Value {
    json!({
        "resolve_tick": resolve_tick,
        "tick_content_hash_hex": hash_hex,
        "page_count": page_count,
    })
}

/// One paged batch reference: the batch identity plus its undrained tail.
fn paged_batch_ref_json(
    resolve_tick: u64,
    hash_hex: &str,
    page_count: usize,
    remaining: usize,
) -> Value {
    json!({
        "resolve_tick": resolve_tick,
        "tick_content_hash_hex": hash_hex,
        "page_count": page_count,
        "remaining": remaining,
    })
}

fn page_input(resolve_tick: u64, index: u64) -> ArchivePageInputV1 {
    ArchivePageInputV1::try_new(
        ArchiveSubjectV1::try_new(
            ArchiveSubjectKindV1::County,
            format!("26{index:03}"),
            "Wayne County".to_owned(),
        )
        .expect("valid subject"),
        resolve_tick,
        tick_hash(TICK_CONTENT_HASH_HEX),
        "Which neighboring place should organizers investigate next?".to_owned(),
        Vec::new(),
        Vec::new(),
    )
    .expect("valid page input")
}

fn batch_from_ref(data: &Value) -> ArchiveDirtyBatchV1 {
    let resolve_tick = data["resolve_tick"].as_u64().expect("u64 resolve tick");
    let page_count =
        usize::try_from(data["page_count"].as_u64().expect("u64 page count")).expect("page count");
    let mut pages = Vec::with_capacity(page_count);
    for index in 0..page_count {
        pages.push(page_input(resolve_tick, index as u64));
    }
    ArchiveDirtyBatchV1::try_new(
        resolve_tick,
        tick_hash(data["tick_content_hash_hex"].as_str().expect("hash hex")),
        pages,
    )
    .expect("valid dirty batch")
}

fn receipt_from_ref(data: &Value) -> PendingArchiveReceiptV1 {
    PendingArchiveReceiptV1::try_new(
        data["resolve_tick"].as_u64().expect("u64 resolve tick"),
        tick_hash(data["tick_content_hash_hex"].as_str().expect("hash hex")),
    )
    .expect("valid pending receipt")
}

fn error_name(error: &SemanticArchiveErrorV1) -> &'static str {
    match error {
        SemanticArchiveErrorV1::InvalidVerifiedTick => "InvalidVerifiedTick",
        SemanticArchiveErrorV1::ReceiptMismatch => "ReceiptMismatch",
        SemanticArchiveErrorV1::StoredPageMismatch => "StoredPageMismatch",
        other => panic!("unexpected vector error variant: {other:?}"),
    }
}

fn plan_name(plan: ArchiveReceiptPlanV1) -> &'static str {
    match plan {
        ArchiveReceiptPlanV1::Stage => "Stage",
        ArchiveReceiptPlanV1::Consume => "Consume",
    }
}

fn sweep_step(value: &Value) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
    if let Some(error) = value.get("error") {
        assert_eq!(error.as_str(), Some("ReceiptMismatch"));
        return Err(SemanticArchiveErrorV1::ReceiptMismatch);
    }
    let batch = batch_from_ref(&value["batch"]);
    let remaining = usize::try_from(
        value["batch"]["remaining"]
            .as_u64()
            .expect("u64 remaining count"),
    )
    .expect("remaining count");
    Ok(ArchiveProducerOutcomeV1::new(batch, remaining))
}

fn rows() -> Vec<Value> {
    let input = VECTORS.strip_suffix('\n').unwrap_or(VECTORS);
    let mut rows = Vec::with_capacity(MAX_ROWS);
    for (index, line) in input.split('\n').take(MAX_ROWS + 1).enumerate() {
        assert!(index < MAX_ROWS, "bounded vector row count");
        assert!(!line.is_empty() && line.len() <= MAX_LINE_BYTES);
        let row: Value = serde_json::from_str(line).expect("valid bounded vector row");
        assert!(
            row["id"].is_string() && row["kind"].is_string() && row["data"].is_object(),
            "vector row shape"
        );
        rows.push(row);
    }
    rows
}

fn rows_of_kind<'a>(rows: &'a [Value], kind: &'a str) -> impl Iterator<Item = &'a Value> + 'a {
    rows.iter()
        .filter(move |row| row["kind"].as_str() == Some(kind))
}

fn watermark_rows() -> Vec<Value> {
    [
        ("watermark-empty-state", None, 0, 0),
        ("watermark-all-consumed", None, 5, 5),
        ("watermark-gap-pending", Some(2), 3, 1),
        ("watermark-pending-first", Some(1), 3, 0),
    ]
    .into_iter()
    .map(|(id, first_pending, max_receipt, expected)| {
        json!({
            "id": id,
            "kind": "watermark",
            "data": {
                "first_pending_tick": first_pending,
                "max_receipt_tick": max_receipt,
                "expected": expected,
            },
        })
    })
    .collect()
}

fn match_rows() -> Vec<Value> {
    let receipt_ref = json!({
        "resolve_tick": 42,
        "tick_content_hash_hex": TICK_CONTENT_HASH_HEX,
    });
    [
        (
            "match-exact-ok",
            batch_ref_json(42, TICK_CONTENT_HASH_HEX, 0),
        ),
        (
            "match-tick-mismatch",
            batch_ref_json(43, TICK_CONTENT_HASH_HEX, 0),
        ),
        (
            "match-hash-mismatch",
            batch_ref_json(42, ALT_CONTENT_HASH_HEX, 0),
        ),
    ]
    .into_iter()
    .map(|(id, batch_ref)| {
        let mut data = json!({"batch": batch_ref, "receipt": receipt_ref.clone()});
        if id == "match-exact-ok" {
            data["expected"] = "ok".into();
        } else {
            data["expected_error"] = "ReceiptMismatch".into();
        }
        json!({"id": id, "kind": "match", "data": data})
    })
    .collect()
}

fn plan_rows() -> Vec<Value> {
    [
        ("plan-empty-consumes", 0, 0),
        ("plan-nonempty-materializes", 1, 0),
        ("plan-multi-page-materializes", 2, 0),
        ("plan-paged-materializes-without-consuming", 256, 316),
        ("plan-empty-budget-exhausted-still-materializes", 0, 4),
    ]
    .into_iter()
    .map(|(id, page_count, remaining)| {
        json!({
            "id": id,
            "kind": "plan",
            "data": {
                "batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, page_count, remaining),
                "expected": plan_name(classify_archive_receipt_v1(
                    &ArchiveProducerOutcomeV1::new(
                        batch_from_ref(&batch_ref_json(42, TICK_CONTENT_HASH_HEX, page_count)),
                        remaining,
                    ),
                )),
            },
        })
    })
    .collect()
}

fn sweep_rows() -> Vec<Value> {
    [
        (
            "sweep-all-quiet",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 0, 0)},
                {"batch": paged_batch_ref_json(44, TICK_CONTENT_HASH_HEX, 0, 0)},
            ]),
            json!({"expected": ["Consume", "Consume"]}),
        ),
        (
            "sweep-mixed-order",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 0, 0)},
                {"batch": paged_batch_ref_json(43, TICK_CONTENT_HASH_HEX, 1, 0)},
                {"batch": paged_batch_ref_json(44, TICK_CONTENT_HASH_HEX, 1, 0)},
            ]),
            json!({"expected": ["Consume", "Consume", "Consume"]}),
        ),
        (
            "sweep-stop-on-first-error",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 0, 0)},
                {"error": "ReceiptMismatch"},
                {"batch": paged_batch_ref_json(44, TICK_CONTENT_HASH_HEX, 0, 0)},
            ]),
            json!({"expected_error": "ReceiptMismatch"}),
        ),
        (
            "sweep-error-first",
            json!([
                {"error": "ReceiptMismatch"},
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 0, 0)},
            ]),
            json!({"expected_error": "ReceiptMismatch"}),
        ),
        // The PER-318 foundation drain: one receipt dirties 745 place pages
        // and 83 county pages. The shared 256-page budget publishes the 83
        // county pages plus a 173-place head, then 256-place pages per sweep,
        // until the final 60-page sweep leaves nothing undrained.
        (
            "sweep-foundation-drain-one",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 256, 572)},
            ]),
            json!({"expected": ["Stage"]}),
        ),
        (
            "sweep-foundation-drain-two",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 256, 316)},
            ]),
            json!({"expected": ["Stage"]}),
        ),
        (
            "sweep-foundation-drain-three",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 256, 60)},
            ]),
            json!({"expected": ["Stage"]}),
        ),
        (
            "sweep-foundation-drain-four",
            json!([
                {"batch": paged_batch_ref_json(42, TICK_CONTENT_HASH_HEX, 60, 0)},
            ]),
            json!({"expected": ["Consume"]}),
        ),
    ]
    .into_iter()
    .map(|(id, steps, expected)| {
        json!({
            "id": id,
            "kind": "sweep",
            "data": {
                "steps": steps,
                "expected": expected["expected"],
                "expected_error": expected["expected_error"],
            },
        })
    })
    .collect()
}

fn disposition_name(disposition: ArchiveReceiptDispositionV1) -> &'static str {
    match disposition {
        ArchiveReceiptDispositionV1::Applied => "Applied",
        ArchiveReceiptDispositionV1::AlreadyConsumed => "AlreadyConsumed",
        ArchiveReceiptDispositionV1::Paged => "Paged",
    }
}

fn identity_row() -> Value {
    let plan_names: Vec<&str> = [ArchiveReceiptPlanV1::Consume, ArchiveReceiptPlanV1::Stage]
        .into_iter()
        .map(plan_name)
        .collect();
    let disposition_names: Vec<&str> = [
        ArchiveReceiptDispositionV1::Applied,
        ArchiveReceiptDispositionV1::AlreadyConsumed,
        ArchiveReceiptDispositionV1::Paged,
    ]
    .into_iter()
    .map(disposition_name)
    .collect();
    json!({
        "id": "identity-sql-and-bound",
        "kind": "identity",
        "data": {
            "source_path": SOURCE_PATH,
            "pending_receipts_sql_sha256_hex": sha256_hex(ARCHIVE_PENDING_RECEIPTS_SQL_V1.as_bytes()),
            "watermark_sql_sha256_hex": sha256_hex(ARCHIVE_SWEEP_WATERMARK_SQL_V1.as_bytes()),
            "max_receipts_per_sweep": ARCHIVE_SWEEP_MAX_RECEIPTS_V1,
            "max_scan_per_sweep": ARCHIVE_SWEEP_MAX_SCAN_V1,
            "plans": plan_names,
            "dispositions": disposition_names,
            "error_variants": ERROR_VARIANTS_USED,
        },
    })
}

fn generate_vectors() -> String {
    let mut lines = watermark_rows();
    lines.extend(match_rows());
    lines.extend(plan_rows());
    lines.extend(sweep_rows());
    lines.push(identity_row());
    let mut output = String::new();
    for line in lines {
        output.push_str(&serde_json::to_string(&line).expect("vector row serializes"));
        output.push('\n');
    }
    output
}

#[test]
#[ignore = "regeneration writes contracts/archive_worker_v1_vectors.jsonl from the pinned worker"]
fn generate_shared_vectors_from_the_pinned_worker() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/archive_worker_v1_vectors.jsonl"
    );
    std::fs::write(path, generate_vectors()).expect("vector corpus written");
}

#[test]
fn shared_watermark_vectors_match_the_contiguous_derivation() {
    let rows = rows();
    let watermark_rows: Vec<&Value> = rows_of_kind(&rows, "watermark").collect();
    assert_eq!(watermark_rows.len(), 4);
    for row in watermark_rows {
        let data = &row["data"];
        let first_pending = data["first_pending_tick"].as_u64();
        let max_receipt_tick = data["max_receipt_tick"].as_u64().expect("u64 max receipt");
        let actual = archive_contiguous_watermark_v1(first_pending, max_receipt_tick);
        assert_eq!(
            actual,
            data["expected"].as_u64().expect("u64 expected watermark"),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn shared_match_vectors_match_the_batch_identity_refusal() {
    let rows = rows();
    let match_rows: Vec<&Value> = rows_of_kind(&rows, "match").collect();
    assert_eq!(match_rows.len(), 3);
    for row in match_rows {
        let data = &row["data"];
        let batch = batch_from_ref(&data["batch"]);
        let receipt = receipt_from_ref(&data["receipt"]);
        let result = archive_batch_matches_receipt_v1(&batch, &receipt);
        if data.get("expected").and_then(Value::as_str) == Some("ok") {
            assert_eq!(result, Ok(()), "{}", row["id"]);
        } else {
            assert_eq!(
                result,
                Err(SemanticArchiveErrorV1::ReceiptMismatch),
                "{}",
                row["id"]
            );
            assert_eq!(
                data["expected_error"].as_str(),
                Some("ReceiptMismatch"),
                "{}",
                row["id"]
            );
        }
    }
}

#[test]
fn shared_plan_vectors_match_the_receipt_classification() {
    let rows = rows();
    let plan_rows: Vec<&Value> = rows_of_kind(&rows, "plan").collect();
    assert_eq!(plan_rows.len(), 5);
    for row in plan_rows {
        let data = &row["data"];
        let produced = sweep_step(&json!({"batch": data["batch"]})).expect("plan outcome");
        assert_eq!(
            plan_name(classify_archive_receipt_v1(&produced)),
            data["expected"].as_str().expect("expected plan"),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn shared_sweep_vectors_match_the_stop_on_first_error_planner() {
    let rows = rows();
    let sweep_rows: Vec<&Value> = rows_of_kind(&rows, "sweep").collect();
    assert_eq!(sweep_rows.len(), 8);
    for row in sweep_rows {
        let data = &row["data"];
        let steps: Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>> = data["steps"]
            .as_array()
            .expect("steps array")
            .iter()
            .map(sweep_step)
            .collect();
        let result = classify_archive_sweep_v1(steps);
        if let Some(expected_error) = data.get("expected_error").and_then(Value::as_str) {
            let error = result.expect_err("sweep vector must stop with an error");
            assert_eq!(error_name(&error), expected_error, "{}", row["id"]);
        } else {
            let plans = result.expect("sweep vector must produce plans");
            let expected = data["expected"].as_array().expect("expected plans");
            assert_eq!(plans.len(), expected.len(), "{}", row["id"]);
            for (plan, expected_plan) in plans.iter().zip(expected) {
                assert_eq!(
                    plan_name(*plan),
                    expected_plan.as_str().expect("expected plan"),
                    "{}",
                    row["id"]
                );
            }
        }
    }
}

#[test]
fn shared_identity_vectors_match_the_pinned_sql_and_taxonomy() {
    let rows = rows();
    let identity_rows: Vec<&Value> = rows_of_kind(&rows, "identity").collect();
    assert_eq!(identity_rows.len(), 1);
    let data = &identity_rows[0]["data"];
    assert_eq!(
        data["pending_receipts_sql_sha256_hex"].as_str(),
        Some(sha256_hex(ARCHIVE_PENDING_RECEIPTS_SQL_V1.as_bytes()).as_str())
    );
    assert_eq!(
        data["watermark_sql_sha256_hex"].as_str(),
        Some(sha256_hex(ARCHIVE_SWEEP_WATERMARK_SQL_V1.as_bytes()).as_str())
    );
    assert_eq!(
        data["max_receipts_per_sweep"].as_i64(),
        Some(ARCHIVE_SWEEP_MAX_RECEIPTS_V1)
    );
    assert_eq!(
        data["max_scan_per_sweep"].as_i64(),
        Some(ARCHIVE_SWEEP_MAX_SCAN_V1)
    );
    assert!(
        data["max_scan_per_sweep"].as_i64().expect("scan bound")
            >= data["max_receipts_per_sweep"]
                .as_i64()
                .expect("receipt bound"),
        "one page always fits inside the scan bound"
    );
    assert_eq!(
        data["plans"].as_array().expect("plans").len(),
        2,
        "plan taxonomy stays pinned"
    );
    assert_eq!(
        data["dispositions"].as_array().expect("dispositions").len(),
        3,
        "disposition taxonomy stays pinned"
    );
    assert_eq!(
        data["error_variants"]
            .as_array()
            .expect("error variants")
            .len(),
        ERROR_VARIANTS_USED.len()
    );
}
