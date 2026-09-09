//! Independent semantic reconstruction for the shared PER-60 vector corpus.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::{StableElementResolverV1, StableIdentityError};
use babylon_graph::substrate::GraphSubstrate;
use babylon_kernel::sha256_of;
use serde_json::Value;

use super::{
    hex_bytes, integer, row_by_id, text, unsigned, verify_fixed_tags, TagRefusal, VectorRow,
    MAX_ROWS,
};

#[path = "tick_content_hash_v1_bound_operations.rs"]
mod bound_operations;

pub(super) fn canonical(row: &VectorRow, rows: &[VectorRow]) -> Option<Vec<u8>> {
    let data = &row.data;
    match row.kind.as_str() {
        "replay_session" => Some(str16(text(data, "session"))),
        "replay_seed" => Some(integer(data, "seed").to_be_bytes().to_vec()),
        "stable_element" => Some(stable_element(data)),
        "carrier_segment" => Some(carrier_segment(data)),
        "stable_carrier_key" => Some(stable_carrier_key(data, rows)),
        "resolver_manifest" => Some(resolver_manifest(data)),
        "stable_graph" => Some(stable_graph(data)),
        "action_id" => Some(action_id(data)),
        "ordered_action_batch" => Some(ordered_action_batch(data)),
        "bsl_discriminant" if data.get("governance_utf8").is_some() => {
            let mut output = Vec::new();
            push_str32(&mut output, text(data, "governance_utf8"));
            Some(output)
        }
        "bsl_discriminant" if data.get("vocabulary").is_some() => {
            Some(vocabulary(&data["vocabulary"]))
        }
        "bsl_discriminant" if data.get("bsl_type").is_some() => Some(bsl_type(&data["bsl_type"])),
        "bsl_discriminant" if data.get("bsl_value").is_some() => {
            Some(bsl_value(&data["bsl_value"]))
        }
        "bsl_discriminant" if data.get("effect").is_some() => Some(effect(&data["effect"])),
        "bsl_discriminant" if data.get("shape_verb").is_some() => Some(vec![tag(
            text(data, "shape_verb"),
            &[
                "add_node",
                "remove_node",
                "add_edge",
                "remove_edge",
                "add_hyperedge",
                "remove_hyperedge",
            ],
        )]),
        "bsl_discriminant" if data.get("field_kind").is_some() => Some(vec![tag(
            text(data, "field_kind"),
            &["intensive", "extensive", "not_applicable"],
        )]),
        "bsl_discriminant" if data.get("rule_role").is_some() => Some(vec![tag(
            text(data, "rule_role"),
            &["mechanic", "recognizer", "external_event", "intent"],
        )]),
        "bsl_discriminant" if data.get("evidence_class").is_some() => Some(vec![tag(
            text(data, "evidence_class"),
            &["observed", "derived", "calibrated", "designed"],
        )]),
        "prepared_environment" => Some(prepared_environment(data, rows)),
        "register_manifest" => Some(register_manifest(data)),
        "register_set" => Some(register_set(data, rows)),
        "stable_world" => Some(stable_world(data, rows)),
        "tick_payload" => Some(tick_payload(data)),
        "tick_content_hash" => Some(tick_content(data, rows)),
        _ => None,
    }
}

fn str16(value: &str) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(
        &u16::try_from(value.len())
            .expect("bounded str16")
            .to_be_bytes(),
    );
    output.extend_from_slice(value.as_bytes());
    output
}

fn push_str32(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(
        &u32::try_from(value.len())
            .expect("bounded str32")
            .to_be_bytes(),
    );
    output.extend_from_slice(value.as_bytes());
}

fn stable_element(data: &Value) -> Vec<u8> {
    let mut output = b"babylon.stable-element\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    match text(data, "element_kind") {
        "node" => {
            output.push(1);
            push_str32(&mut output, text(data, "scenario"));
            push_str32(&mut output, text(data, "local_name"));
        }
        "edge" => {
            output.push(2);
            for name in [
                "scenario",
                "edge_type",
                "source_local_name",
                "target_local_name",
            ] {
                push_str32(&mut output, text(data, name));
            }
        }
        "hyperedge" => {
            output.push(3);
            push_str32(&mut output, text(data, "scenario"));
            push_str32(&mut output, text(data, "local_name"));
        }
        _ => panic!("closed stable element kind"),
    }
    output
}

fn carrier_segment(data: &Value) -> Vec<u8> {
    let mut output = Vec::new();
    for (index, segment) in data["segments"]
        .as_array()
        .expect("carrier segments")
        .iter()
        .take(258)
        .enumerate()
    {
        if index > 0 {
            output.push(b'|');
        }
        let segment = segment.as_str().expect("carrier segment text");
        output.extend_from_slice(segment.len().to_string().as_bytes());
        output.push(b':');
        output.extend_from_slice(segment.as_bytes());
    }
    output
}

fn stable_carrier_key(data: &Value, rows: &[VectorRow]) -> Vec<u8> {
    let active = data["active_segment_ids"]
        .as_array()
        .expect("active stable carrier segments");
    assert!(
        active.len() <= 256,
        "bounded active stable carrier segments"
    );
    let mut segment_ids = Vec::with_capacity(active.len() + 1);
    segment_ids.push(text(data, "subject_segment_id"));
    for id in active.iter().take(256) {
        segment_ids.push(id.as_str().expect("active carrier segment id"));
    }
    assert_eq!(
        validate_carrier_provenance(data, &segment_ids, rows),
        Ok(())
    );
    let mut segments = Vec::with_capacity(segment_ids.len() + 1);
    for id in segment_ids.iter().take(257) {
        let linked = row_by_id(rows, id);
        assert_eq!(
            linked.kind, "carrier_segment",
            "graph-owned carrier segment"
        );
        segments.push(carrier_segment(&linked.data));
    }
    segments.push(integer(data, "draw_slot").to_string().into_bytes());
    let mut output = Vec::new();
    for (index, segment) in segments.iter().take(258).enumerate() {
        if index > 0 {
            output.push(b'|');
        }
        output.extend_from_slice(segment.len().to_string().as_bytes());
        output.push(b':');
        output.extend_from_slice(segment);
    }
    assert!(output.len() <= 105_962, "bounded final stable carrier");
    output
}

fn validate_carrier_provenance(
    data: &Value,
    segment_ids: &[&str],
    rows: &[VectorRow],
) -> Result<(), &'static str> {
    let resolver = rows
        .iter()
        .take(MAX_ROWS)
        .find(|row| row.id == text(data, "resolver_id"))
        .filter(|row| row.kind == "resolver_manifest")
        .ok_or("carrier_provenance")?;
    let graph = rows
        .iter()
        .take(MAX_ROWS)
        .find(|row| row.id == text(data, "stable_graph_id"))
        .filter(|row| row.kind == "stable_graph")
        .ok_or("carrier_provenance")?;
    if text(&resolver.data, "scenario") != text(&graph.data, "scenario") {
        return Err("carrier_provenance");
    }
    let resolver_nodes = named_types(&resolver.data["nodes"], "node_type");
    let graph_nodes = named_types(&graph.data["nodes"], "node_type");
    let resolver_hyperedges = named_types(&resolver.data["hyperedges"], "hyperedge_type");
    let graph_hyperedges = named_types(&graph.data["hyperedges"], "hyperedge_type");
    if resolver_nodes != graph_nodes || resolver_hyperedges != graph_hyperedges {
        return Err("carrier_provenance");
    }
    let graph_edges: BTreeSet<(&str, &str, &str)> = graph.data["edges"]
        .as_array()
        .expect("stable graph edges")
        .iter()
        .take(65_536)
        .map(|row| {
            (
                text(row, "edge_type"),
                text(row, "source"),
                text(row, "target"),
            )
        })
        .collect();
    for row_id in segment_ids.iter().take(257) {
        let linked = rows
            .iter()
            .take(MAX_ROWS)
            .find(|row| row.id == **row_id)
            .filter(|row| row.kind == "carrier_segment")
            .ok_or("carrier_provenance")?;
        if !carrier_segment_is_sealed(
            &linked.data,
            text(&resolver.data, "scenario"),
            &resolver_nodes,
            &resolver_hyperedges,
            &graph_edges,
        ) {
            return Err("carrier_provenance");
        }
    }
    Ok(())
}

fn named_types<'a>(rows: &'a Value, type_field: &str) -> BTreeMap<&'a str, &'a str> {
    rows.as_array()
        .expect("named identity rows")
        .iter()
        .take(65_536)
        .map(|row| (text(row, "local_name"), text(row, type_field)))
        .collect()
}

fn carrier_segment_is_sealed(
    data: &Value,
    scenario: &str,
    nodes: &BTreeMap<&str, &str>,
    hyperedges: &BTreeMap<&str, &str>,
    edges: &BTreeSet<(&str, &str, &str)>,
) -> bool {
    let segments = data["segments"].as_array().expect("carrier segments");
    if segments.get(1).and_then(Value::as_str) != Some(scenario) {
        return false;
    }
    match segments.first().and_then(Value::as_str) {
        Some("node") => segments
            .get(2)
            .and_then(Value::as_str)
            .is_some_and(|name| nodes.contains_key(name)),
        Some("hyperedge") => segments
            .get(2)
            .and_then(Value::as_str)
            .is_some_and(|name| hyperedges.contains_key(name)),
        Some("edge") => {
            let Some(edge_type) = segments.get(2).and_then(Value::as_str) else {
                return false;
            };
            let Some(source) = segments.get(3).and_then(Value::as_str) else {
                return false;
            };
            let Some(target) = segments.get(4).and_then(Value::as_str) else {
                return false;
            };
            nodes.contains_key(source)
                && nodes.contains_key(target)
                && edges.contains(&(edge_type, source, target))
        }
        _ => false,
    }
}

fn resolver_manifest(data: &Value) -> Vec<u8> {
    let mut output = b"babylon.stable-element-resolver\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    output.push(1);
    push_str32(&mut output, text(data, "scenario"));
    let mut nodes = sorted_rows(&data["nodes"], &["local_name"]);
    output.push(2);
    output.extend_from_slice(
        &u32::try_from(nodes.len())
            .expect("node count")
            .to_be_bytes(),
    );
    for node in nodes.drain(..).take(65_536) {
        push_str32(&mut output, text(node, "local_name"));
        push_str32(&mut output, text(node, "node_type"));
    }
    let mut hyperedges = sorted_rows(&data["hyperedges"], &["local_name"]);
    output.push(3);
    output.extend_from_slice(
        &u32::try_from(hyperedges.len())
            .expect("hyperedge count")
            .to_be_bytes(),
    );
    for hyperedge in hyperedges.drain(..).take(65_536) {
        push_str32(&mut output, text(hyperedge, "local_name"));
        push_str32(&mut output, text(hyperedge, "hyperedge_type"));
    }
    output
}

fn stable_graph(data: &Value) -> Vec<u8> {
    let mut output = b"babylon.stable-graph\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    output.push(1);
    push_str32(&mut output, text(data, "scenario"));
    append_stable_nodes(&mut output, data);
    append_stable_node_f64(&mut output, data);
    append_stable_edges(&mut output, data);
    append_stable_hyperedges(&mut output, data);
    append_stable_edge_f64(&mut output, data);
    append_stable_currency(&mut output, data);
    append_stable_hyperedge_f64(&mut output, data);
    output
}

fn append_count(output: &mut Vec<u8>, tag: u8, count: usize) {
    output.push(tag);
    output.extend_from_slice(&u32::try_from(count).expect("section count").to_be_bytes());
}

fn append_stable_nodes(output: &mut Vec<u8>, data: &Value) {
    let rows = sorted_rows(&data["nodes"], &["local_name"]);
    append_count(output, 2, rows.len());
    for row in rows.into_iter().take(65_536) {
        push_str32(output, text(row, "local_name"));
        push_str32(output, text(row, "node_type"));
    }
}

fn append_stable_node_f64(output: &mut Vec<u8>, data: &Value) {
    let rows = sorted_rows(&data["node_f64"], &["local_name", "qname"]);
    append_count(output, 3, rows.len());
    for row in rows.into_iter().take(1_048_576) {
        push_str32(output, text(row, "local_name"));
        push_str32(output, text(row, "qname"));
        output.extend_from_slice(&f64_bits(text(row, "value_bits_hex")));
    }
}

fn append_stable_edges(output: &mut Vec<u8>, data: &Value) {
    let rows = sorted_rows(&data["edges"], &["edge_type", "source", "target"]);
    append_count(output, 4, rows.len());
    for row in rows.into_iter().take(65_536) {
        for name in ["edge_type", "source", "target"] {
            push_str32(output, text(row, name));
        }
        output.extend_from_slice(&f64_bits(text(row, "strength_bits_hex")));
    }
}

fn append_stable_hyperedges(output: &mut Vec<u8>, data: &Value) {
    let rows = sorted_rows(&data["hyperedges"], &["local_name"]);
    append_count(output, 5, rows.len());
    for row in rows.into_iter().take(65_536) {
        push_str32(output, text(row, "local_name"));
        push_str32(output, text(row, "hyperedge_type"));
        let mut members: Vec<&str> = row["members"]
            .as_array()
            .expect("hyperedge members")
            .iter()
            .map(|value| value.as_str().expect("member text"))
            .collect();
        members.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        output.extend_from_slice(
            &u32::try_from(members.len())
                .expect("member count")
                .to_be_bytes(),
        );
        for member in members.into_iter().take(65_536) {
            push_str32(output, member);
        }
    }
}

fn append_stable_edge_f64(output: &mut Vec<u8>, data: &Value) {
    let names = ["edge_type", "source", "target", "qname"];
    let rows = sorted_rows(&data["edge_f64"], &names);
    append_count(output, 6, rows.len());
    for row in rows.into_iter().take(1_048_576) {
        for name in names {
            push_str32(output, text(row, name));
        }
        output.extend_from_slice(&f64_bits(text(row, "value_bits_hex")));
    }
}

fn append_stable_currency(output: &mut Vec<u8>, data: &Value) {
    let rows = sorted_rows(&data["node_currency"], &["local_name", "qname"]);
    append_count(output, 7, rows.len());
    for row in rows.into_iter().take(1_048_576) {
        push_str32(output, text(row, "local_name"));
        push_str32(output, text(row, "qname"));
        let value: i128 = text(row, "micro_units").parse().expect("i128 micro-units");
        output.extend_from_slice(&value.to_be_bytes());
    }
}

fn append_stable_hyperedge_f64(output: &mut Vec<u8>, data: &Value) {
    let rows = sorted_rows(&data["hyperedge_f64"], &["local_name", "qname"]);
    append_count(output, 8, rows.len());
    for row in rows.into_iter().take(1_048_576) {
        push_str32(output, text(row, "local_name"));
        push_str32(output, text(row, "qname"));
        output.extend_from_slice(&f64_bits(text(row, "value_bits_hex")));
    }
}

fn f64_bits(value: &str) -> [u8; 8] {
    let raw: [u8; 8] = hex_bytes(value).try_into().expect("binary64 bits");
    let bits = u64::from_be_bytes(raw);
    if bits << 1 == 0 {
        0u64.to_be_bytes()
    } else {
        bits.to_be_bytes()
    }
}

fn sorted_rows<'a>(value: &'a Value, fields: &[&str]) -> Vec<&'a Value> {
    let mut rows: Vec<_> = value.as_array().expect("semantic rows").iter().collect();
    rows.sort_unstable_by(|left, right| compare_fields(left, right, fields));
    rows
}

fn compare_fields(left: &Value, right: &Value, fields: &[&str]) -> Ordering {
    for field in fields.iter().take(4) {
        let order = text(left, field)
            .as_bytes()
            .cmp(text(right, field).as_bytes());
        if order != Ordering::Equal {
            return order;
        }
    }
    Ordering::Equal
}

fn action_id(data: &Value) -> Vec<u8> {
    let intent = hex_bytes(text(data, "intent_bytes_hex"));
    let intent_digest = sha256_of(&intent);
    assert_eq!(intent_digest, super::hex32(text(data, "intent_digest_hex")));
    action_preimage(text(data, "session"), &intent_digest)
}

fn action_preimage(session: &str, intent_digest: &[u8; 32]) -> Vec<u8> {
    let mut output = b"babylon.practice-action-id.v1\0".to_vec();
    output.extend_from_slice(&1u16.to_be_bytes());
    output.extend_from_slice(&str16(session));
    output.extend_from_slice(&2u16.to_be_bytes());
    output.extend_from_slice(intent_digest);
    output
}

fn ordered_action_batch(data: &Value) -> Vec<u8> {
    let mut output = b"babylon.ordered-practice-action-batch.v1\0".to_vec();
    output.extend_from_slice(&1u16.to_be_bytes());
    output.extend_from_slice(&str16(text(data, "session")));
    output.extend_from_slice(&unsigned(data, "resolve_tick").to_be_bytes());
    let items = data["items"].as_array().expect("ordered action items");
    output.extend_from_slice(
        &u16::try_from(items.len())
            .expect("action count")
            .to_be_bytes(),
    );
    for (index, item) in items.iter().take(4_096).enumerate() {
        assert_eq!(unsigned(item, "ordinal"), index as u64);
        let intent = hex_bytes(text(item, "intent_bytes_hex"));
        let action = sha256_of(&action_preimage(text(data, "session"), &sha256_of(&intent)));
        output.extend_from_slice(&u16::try_from(index).expect("action ordinal").to_be_bytes());
        output.extend_from_slice(&action);
        output.extend_from_slice(
            &u16::try_from(intent.len())
                .expect("intent length")
                .to_be_bytes(),
        );
        output.extend_from_slice(&intent);
    }
    output
}

fn vocabulary(data: &Value) -> Vec<u8> {
    if !data["present"].as_bool().expect("vocabulary presence") {
        return vec![0];
    }
    let mut output = vec![1];
    for (index, name) in ["node_type", "edge_type", "hyperedge_type", "event_type"]
        .iter()
        .enumerate()
    {
        let row = &data["kinds"][name];
        let present = row["present"].as_bool().expect("kind presence");
        output.extend_from_slice(&[
            u8::try_from(index + 1).expect("kind tag"),
            u8::from(present),
        ]);
        if present {
            let mut members: Vec<&str> = row["members"]
                .as_array()
                .expect("vocabulary members")
                .iter()
                .map(|value| value.as_str().expect("member text"))
                .collect();
            members.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            output.extend_from_slice(
                &u32::try_from(members.len())
                    .expect("member count")
                    .to_be_bytes(),
            );
            for member in members.into_iter().take(1_048_576) {
                push_str32(&mut output, member);
            }
        }
    }
    output
}

fn prepared_environment(data: &Value, rows: &[VectorRow]) -> Vec<u8> {
    let mut output = b"babylon.prepared-environment\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    output.push(1);
    output.extend_from_slice(&hex_bytes(text(data, "rules_hash_hex")));
    output.push(2);
    output.extend_from_slice(
        &u32::try_from(unsigned(data, "phase_schedule_layout"))
            .expect("layout")
            .to_be_bytes(),
    );
    output.extend_from_slice(&hex_bytes(text(data, "phase_schedule_digest_hex")));
    append_prepared_rules(&mut output, data);
    append_prepared_fields(&mut output, data);
    append_prepared_intrinsics(&mut output, data);
    append_prepared_constants(&mut output, data);
    append_prepared_enums(&mut output, data);
    output.push(8);
    output.extend_from_slice(&vocabulary(&data["vocabulary"]));
    output.push(9);
    output.extend_from_slice(&1u32.to_be_bytes());
    output.extend_from_slice(&digest_by_id(rows, text(data, "resolver_manifest_id")));
    output.push(10);
    output.extend_from_slice(&1u32.to_be_bytes());
    output.extend_from_slice(&digest_by_id(rows, text(data, "register_manifest_id")));
    output
}

fn append_prepared_rules(output: &mut Vec<u8>, data: &Value) {
    let rules = data["rule_order"].as_array().expect("prepared rules");
    append_count(output, 3, rules.len());
    for rule in rules.iter().take(65_536) {
        push_str32(output, rule.as_str().expect("rule text"));
    }
}

fn append_prepared_fields(output: &mut Vec<u8>, data: &Value) {
    output.push(4);
    let rows = sorted_rows(&data["fields"], &["qname"]);
    output.extend_from_slice(
        &u32::try_from(rows.len())
            .expect("field count")
            .to_be_bytes(),
    );
    for row in rows.into_iter().take(65_536) {
        push_str32(output, text(row, "qname"));
        output.extend_from_slice(&bsl_type(&row["type"]));
        output.push(match text(row, "field_kind") {
            "intensive" => 1,
            "extensive" => 2,
            "not_applicable" => 3,
            _ => panic!("closed field kind"),
        });
    }
    let names = ["field", "reason", "owner", "date"];
    let exemptions = sorted_rows(&data["exemptions"], &names);
    output.extend_from_slice(
        &u32::try_from(exemptions.len())
            .expect("exemption count")
            .to_be_bytes(),
    );
    for row in exemptions.into_iter().take(64) {
        for name in names {
            push_str32(output, text(row, name));
        }
    }
}

fn append_prepared_intrinsics(output: &mut Vec<u8>, data: &Value) {
    output.push(5);
    let rows = sorted_rows(&data["intrinsics"], &["name"]);
    output.extend_from_slice(
        &u32::try_from(rows.len())
            .expect("intrinsic count")
            .to_be_bytes(),
    );
    for row in rows.into_iter().take(64) {
        push_str32(output, text(row, "name"));
        output.extend_from_slice(&unsigned(row, "cost").to_be_bytes());
    }
}

fn append_prepared_constants(output: &mut Vec<u8>, data: &Value) {
    output.push(6);
    let rows = sorted_rows(&data["constants"], &["qname"]);
    output.extend_from_slice(
        &u32::try_from(rows.len())
            .expect("constant count")
            .to_be_bytes(),
    );
    for row in rows.into_iter().take(65_536) {
        push_str32(output, text(row, "qname"));
        output.extend_from_slice(&bsl_value(&row["value"]));
    }
}

fn append_prepared_enums(output: &mut Vec<u8>, data: &Value) {
    output.push(7);
    let rows = sorted_rows(&data["enum_types"], &["name"]);
    output.extend_from_slice(&u32::try_from(rows.len()).expect("enum count").to_be_bytes());
    for row in rows.into_iter().take(65_536) {
        push_str32(output, text(row, "name"));
        let members = row["members"].as_array().expect("enum members");
        output.extend_from_slice(
            &u32::try_from(members.len())
                .expect("member count")
                .to_be_bytes(),
        );
        for member in members.iter().take(1_048_576) {
            push_str32(output, member.as_str().expect("member text"));
        }
    }
}

fn bsl_type(data: &Value) -> Vec<u8> {
    let kind = text(data, "kind");
    let tag = match kind {
        "probability" => 1,
        "intensity" => 2,
        "coefficient" => 3,
        "currency" => 4,
        "real" => 5,
        "int" => 6,
        "bool" => 7,
        "enum" => 8,
        "node_set" => 9,
        "edge_set" => 10,
        _ => panic!("closed BSL type"),
    };
    let mut output = vec![tag];
    if matches!(kind, "enum" | "node_set" | "edge_set") {
        push_str32(&mut output, text(data, "name"));
    }
    output
}

fn bsl_value(data: &Value) -> Vec<u8> {
    let mut output = Vec::new();
    match text(data, "kind") {
        "int" => {
            output.push(1);
            output.extend_from_slice(&integer(data, "value").to_be_bytes());
        }
        "currency" => {
            output.push(2);
            let value: i128 = text(data, "micro_units").parse().expect("Currency value");
            output.extend_from_slice(&value.to_be_bytes());
        }
        "real" => {
            output.push(3);
            output.extend_from_slice(&f64_bits(text(data, "value_bits_hex")));
        }
        "ratio" => append_ratio(&mut output, data),
        "bool" => output.extend_from_slice(&[5, u8::from(data["value"].as_bool().expect("bool"))]),
        "enum" => {
            output.push(6);
            push_str32(&mut output, text(data, "enum_type"));
            push_str32(&mut output, text(data, "member"));
        }
        "node_ref" | "hyperedge_ref" | "edge_ref" => append_reference(&mut output, data),
        _ => panic!("closed ValueV1 kind"),
    }
    output
}

fn append_ratio(output: &mut Vec<u8>, data: &Value) {
    output.push(4);
    output.extend_from_slice(&f64_bits(text(data, "value_bits_hex")));
    for name in ["floor_bits_hex", "cap_bits_hex"] {
        if data[name].is_null() {
            output.push(0);
        } else {
            output.push(1);
            output.extend_from_slice(&f64_bits(text(data, name)));
        }
    }
}

fn append_reference(output: &mut Vec<u8>, data: &Value) {
    let (tag, kind) = match text(data, "kind") {
        "node_ref" => (7, "node"),
        "hyperedge_ref" => (8, "hyperedge"),
        "edge_ref" => (9, "edge"),
        _ => unreachable!(),
    };
    let mut element = data["element"].clone();
    element["element_kind"] = Value::String(kind.to_owned());
    output.push(tag);
    output.extend_from_slice(&stable_element(&element));
}

fn register_manifest(data: &Value) -> Vec<u8> {
    let mut output = b"babylon.world-register-manifest\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    let entries = data["entries"].as_array().expect("register entries");
    output.extend_from_slice(
        &u32::try_from(entries.len())
            .expect("entry count")
            .to_be_bytes(),
    );
    for entry in entries.iter().take(1_048_576) {
        push_str32(&mut output, text(entry, "name"));
        output.extend_from_slice(
            &u32::try_from(unsigned(entry, "layout"))
                .expect("layout")
                .to_be_bytes(),
        );
    }
    output
}

fn register_set(data: &Value, rows: &[VectorRow]) -> Vec<u8> {
    let mut output = b"babylon.world-register-set\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    output.push(1);
    output.extend_from_slice(&1u32.to_be_bytes());
    output.extend_from_slice(&digest_by_id(rows, text(data, "register_manifest_id")));
    output.push(2);
    let entries = data["entries"].as_array().expect("register entries");
    output.extend_from_slice(
        &u32::try_from(entries.len())
            .expect("entry count")
            .to_be_bytes(),
    );
    for entry in entries.iter().take(1_048_576) {
        push_str32(&mut output, text(entry, "name"));
        output.extend_from_slice(
            &u32::try_from(unsigned(entry, "layout"))
                .expect("layout")
                .to_be_bytes(),
        );
        output.extend_from_slice(&8u32.to_be_bytes());
        output.extend_from_slice(&integer(entry, "completed_tick").to_be_bytes());
    }
    output
}

fn stable_world(data: &Value, rows: &[VectorRow]) -> Vec<u8> {
    let mut output = b"babylon.stable-world\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    output.push(1);
    output.extend_from_slice(&1u32.to_be_bytes());
    output.extend_from_slice(&digest_by_id(rows, text(data, "stable_graph_id")));
    output.push(2);
    output.extend_from_slice(&1u32.to_be_bytes());
    output.extend_from_slice(&digest_by_id(rows, text(data, "register_set_id")));
    output
}

fn tick_payload(data: &Value) -> Vec<u8> {
    let mut output = b"babylon.tick-payload\0".to_vec();
    output.extend_from_slice(&1u32.to_be_bytes());
    append_payload_outcomes(&mut output, data);
    append_payload_events(&mut output, data);
    append_payload_receipts(&mut output, data);
    output.extend_from_slice(&[4, 0, 0]);
    output
}

fn append_payload_outcomes(output: &mut Vec<u8>, data: &Value) {
    let rows = data["rule_outcomes"].as_array().expect("rule outcomes");
    append_count(output, 1, rows.len());
    for row in rows.iter().take(65_536) {
        push_str32(output, text(row, "rule"));
        output.extend_from_slice(&unsigned(row, "fired").to_be_bytes());
    }
}

fn append_payload_events(output: &mut Vec<u8>, data: &Value) {
    let rows = data["events"].as_array().expect("events");
    append_count(output, 2, rows.len());
    for row in rows.iter().take(1_048_576) {
        push_str32(output, text(row, "event"));
        let payload = row["payload"].as_array().expect("payload items");
        output.extend_from_slice(
            &u32::try_from(payload.len())
                .expect("payload count")
                .to_be_bytes(),
        );
        for item in payload.iter().take(1_048_576) {
            push_str32(output, text(item, "label"));
            output.extend_from_slice(&bsl_value(&item["value"]));
        }
    }
}

fn append_payload_receipts(output: &mut Vec<u8>, data: &Value) {
    let rows = data["receipts"].as_array().expect("receipts");
    append_count(output, 3, rows.len());
    for row in rows.iter().take(1_048_576) {
        push_str32(output, text(row, "rule"));
        output.push(tag(
            text(row, "role"),
            &["mechanic", "recognizer", "external_event", "intent"],
        ));
        output.push(tag(
            text(row, "evidence"),
            &["observed", "derived", "calibrated", "designed"],
        ));
        output.extend_from_slice(
            &u32::try_from(unsigned(row, "ordinal"))
                .expect("ordinal")
                .to_be_bytes(),
        );
        output.extend_from_slice(&effect(&row["effect"]));
    }
}

fn effect(data: &Value) -> Vec<u8> {
    let kind = text(data, "kind");
    if let Some(index) = ["node_field", "edge_field", "hyperedge_field"]
        .iter()
        .position(|value| *value == kind)
    {
        let mut output = vec![u8::try_from(index + 1).expect("effect tag")];
        push_str32(&mut output, text(data, "qname"));
        return output;
    }
    if kind == "event" {
        let mut output = vec![4];
        push_str32(&mut output, text(data, "event"));
        return output;
    }
    vec![
        5,
        tag(
            text(data, "verb"),
            &[
                "add_node",
                "remove_node",
                "add_edge",
                "remove_edge",
                "add_hyperedge",
                "remove_hyperedge",
            ],
        ),
    ]
}

fn tag(value: &str, choices: &[&str]) -> u8 {
    u8::try_from(
        choices
            .iter()
            .position(|choice| *choice == value)
            .expect("closed tag")
            + 1,
    )
    .expect("u8 tag")
}

fn tick_content(data: &Value, rows: &[VectorRow]) -> Vec<u8> {
    validate_outer_action_link(data, rows).expect("exact-empty runtime action link");
    let mut output = b"babylon.tick-content\0".to_vec();
    output.extend_from_slice(&layout(data, "outer", 1).to_be_bytes());
    output.push(1);
    output.extend_from_slice(&layout(data, "session", 1).to_be_bytes());
    output.extend_from_slice(&str16(text(data, "session")));
    output.push(2);
    output.extend_from_slice(&unsigned(data, "resolve_tick").to_be_bytes());
    output.push(3);
    output.extend_from_slice(&layout(data, "seed", 1).to_be_bytes());
    output.extend_from_slice(&layout(data, "rng", 2).to_be_bytes());
    output.extend_from_slice(&integer(data, "seed").to_be_bytes());
    output.push(4);
    output.extend_from_slice(&layout(data, "content", 1).to_be_bytes());
    output.extend_from_slice(&hex_bytes(text(data, "defines_digest_hex")));
    output.extend_from_slice(&hex_bytes(text(data, "rules_digest_hex")));
    for (tag, name) in [
        "reference",
        "prepared",
        "prior_world",
        "actions",
        "result_world",
        "payload",
    ]
    .iter()
    .enumerate()
    {
        output.push(u8::try_from(tag + 5).expect("outer tag"));
        output.extend_from_slice(&layout(data, name, 1).to_be_bytes());
        output.extend_from_slice(&outer_digest(data, rows, name));
    }
    output
}

fn validate_outer_action_link(data: &Value, rows: &[VectorRow]) -> Result<(), &'static str> {
    let action = row_by_id(rows, text(data, "actions_id"));
    if action.kind != "ordered_action_batch" {
        return Err("runtime_actions_link");
    }
    let action_data = &action.data;
    if !action_data["items"]
        .as_array()
        .ok_or("runtime_actions_link")?
        .is_empty()
    {
        return Err("nonempty_runtime_actions");
    }
    if text(action_data, "session") != text(data, "session") {
        return Err("action_session_mismatch");
    }
    if unsigned(action_data, "resolve_tick") != unsigned(data, "resolve_tick") {
        return Err("action_tick_mismatch");
    }
    Ok(())
}

fn layout(data: &Value, name: &str, default: u32) -> u32 {
    data["layout_overrides"][name]
        .as_u64()
        .map_or(default, |value| u32::try_from(value).expect("layout u32"))
}

fn outer_digest(data: &Value, rows: &[VectorRow], name: &str) -> [u8; 32] {
    let direct = format!("{name}_digest_hex");
    if let Some(value) = data[&direct].as_str() {
        return super::hex32(value);
    }
    let linked = format!("{name}_id");
    digest_by_id(rows, text(data, &linked))
}

fn digest_by_id(rows: &[VectorRow], id: &str) -> [u8; 32] {
    let row = row_by_id(rows, id);
    let canonical = hex_bytes(text(&row.data, "canonical_hex"));
    sha256_of(&canonical)
}

pub(super) fn verify_every_canonical_row(rows: &[VectorRow]) {
    for row in rows.iter().take(MAX_ROWS) {
        if matches!(row.kind.as_str(), "mutation" | "refusal") {
            continue;
        }
        let Some(expected) = row.data["canonical_hex"].as_str() else {
            continue;
        };
        let actual =
            canonical(row, rows).unwrap_or_else(|| panic!("{} needs a semantic builder", row.id));
        assert_eq!(actual, hex_bytes(expected), "{}", row.id);
    }
}

pub(super) fn verify_every_outer_mutation(rows: &[VectorRow]) {
    for mutation in rows
        .iter()
        .take(MAX_ROWS)
        .filter(|row| row.kind == "mutation")
    {
        let data = &mutation.data;
        let base = row_by_id(rows, text(data, "base_id"));
        let mut changed = base.data.clone();
        let field = text(data, "field");
        if let Some(name) = field.strip_prefix("layout_overrides.") {
            changed["layout_overrides"][name] = data["replacement"].clone();
        } else {
            changed[field] = data["replacement"].clone();
            if let Some(name) = field.strip_suffix("_digest_hex") {
                changed
                    .as_object_mut()
                    .expect("outer data mapping")
                    .remove(&format!("{name}_id"));
            }
        }
        if let Some(actions_id) = data["derived_actions_id"].as_str() {
            changed["actions_id"] = Value::String(actions_id.to_owned());
        }
        let actual = tick_content(&changed, rows);
        assert_ne!(actual, tick_content(&base.data, rows), "{}", mutation.id);
        assert_eq!(
            sha256_of(&actual),
            super::hex32(text(data, "after_digest_hex")),
            "{}",
            mutation.id
        );
    }
}

pub(super) fn verify_every_refusal(rows: &[VectorRow]) {
    let refusals: Vec<_> = rows
        .iter()
        .take(MAX_ROWS)
        .filter(|row| row.kind == "refusal")
        .collect();
    assert!(!refusals.is_empty(), "shared refusal rows");
    for row in refusals.into_iter().take(MAX_ROWS) {
        verify_refusal(row, rows);
    }
}

fn verify_refusal(row: &VectorRow, rows: &[VectorRow]) {
    let data = &row.data;
    match text(data, "operation") {
        "session" => assert_eq!(
            session_refusal(text(data, "value")),
            text(data, "expected_code")
        ),
        "bsl_value" => assert_eq!(
            bsl_value_refusal(&data["value"]),
            text(data, "expected_code")
        ),
        "option_byte" => {
            assert_eq!(unsigned(data, "value"), 2);
            assert_eq!(text(data, "expected_code"), "invalid_option");
        }
        "ordered_tags" => verify_tag_refusal(data),
        "outer_action_link" => {
            let mut outer = row_by_id(rows, text(data, "outer_id")).data.clone();
            outer["actions_id"] = Value::String(text(data, "actions_id").to_owned());
            assert_eq!(
                validate_outer_action_link(&outer, rows),
                Err(text(data, "expected_code")),
                "{}",
                row.id
            );
        }
        "stable_carrier_provenance" => {
            let mut carrier = row_by_id(rows, text(data, "carrier_id")).data.clone();
            carrier["subject_segment_id"] =
                Value::String(text(data, "subject_segment_id").to_owned());
            let active = carrier["active_segment_ids"]
                .as_array()
                .expect("active carrier segments");
            let mut segment_ids = Vec::with_capacity(active.len() + 1);
            segment_ids.push(text(&carrier, "subject_segment_id"));
            for id in active.iter().take(256) {
                segment_ids.push(id.as_str().expect("active carrier segment id"));
            }
            assert_eq!(
                validate_carrier_provenance(&carrier, &segment_ids, rows),
                Err(text(data, "expected_code"))
            );
        }
        "seal_stable_resolver" => verify_resolver_refusal(data),
        "bound_case" => verify_bound_case(data),
        operation => panic!("unexecuted shared refusal operation {operation}"),
    }
}

fn verify_resolver_refusal(data: &Value) {
    let manifest = &data["manifest"];
    let mut graph = MemoryGraph::new();
    let nodes = manifest["nodes"]
        .as_array()
        .expect("resolver refusal nodes");
    assert!(nodes.len() <= 65_536, "bounded resolver refusal nodes");
    let mut handles = Vec::with_capacity(nodes.len());
    let mut node_names = HashMap::with_capacity(nodes.len());
    for node in nodes.iter().take(65_536) {
        let handle = graph
            .add_node(text(node, "node_type"))
            .expect("resolver refusal node");
        handles.push(handle);
        node_names.insert(handle, text(node, "local_name").to_owned());
    }
    let mut hyperedge_names = HashMap::new();
    let hyperedges = manifest["hyperedges"]
        .as_array()
        .expect("resolver refusal hyperedges");
    assert!(
        hyperedges.len() <= 65_536,
        "bounded resolver refusal hyperedges"
    );
    for hyperedge in hyperedges.iter().take(65_536) {
        let member_indices = hyperedge["member_node_indices"]
            .as_array()
            .expect("resolver refusal member indices");
        assert!(
            member_indices.len() <= 65_534,
            "bounded resolver refusal members"
        );
        let members: Vec<_> = member_indices
            .iter()
            .take(65_534)
            .map(|index| {
                handles[usize::try_from(index.as_u64().expect("resolver refusal member index"))
                    .expect("bounded member index")]
            })
            .collect();
        let handle = graph
            .add_hyperedge(text(hyperedge, "hyperedge_type"), &members)
            .expect("resolver refusal hyperedge");
        hyperedge_names.insert(handle, text(hyperedge, "local_name").to_owned());
    }
    let result = StableElementResolverV1::seal(
        &graph,
        text(manifest, "scenario"),
        &node_names,
        &hyperedge_names,
    );
    let actual = match result {
        Err(StableIdentityError::DuplicateNodeName { .. }) => "duplicate_node_name",
        Err(StableIdentityError::DuplicateHyperedgeName { .. }) => "duplicate_hyperedge_name",
        other => panic!("unexpected duplicate authored-name result: {other:?}"),
    };
    assert_eq!(actual, text(data, "expected_code"));
}

fn session_refusal(value: &str) -> &'static str {
    if !value.is_ascii() {
        return "non_ascii";
    }
    if value.len() > 256 {
        return "string_too_long";
    }
    if value.is_empty() || value.bytes().any(|byte| !(0x21..=0x7e).contains(&byte)) {
        return "non_graphic_ascii";
    }
    "missing_refusal"
}

fn bsl_value_refusal(value: &Value) -> &'static str {
    if text(value, "kind") == "bool" && value["value"].as_bool().is_none() {
        return "noncanonical_boolean";
    }
    let bits = u64::from_be_bytes(
        hex_bytes(text(value, "value_bits_hex"))
            .try_into()
            .expect("binary64 refusal bits"),
    );
    if bits & 0x7ff0_0000_0000_0000 == 0x7ff0_0000_0000_0000 {
        "non_finite"
    } else {
        "missing_refusal"
    }
}

fn verify_tag_refusal(data: &Value) {
    let expected: Vec<u8> = data["expected_tags"]
        .as_array()
        .expect("expected tags")
        .iter()
        .take(10)
        .map(|value| u8::try_from(value.as_u64().expect("tag")).expect("u8 tag"))
        .collect();
    let actual = verify_fixed_tags(
        &hex_bytes(text(data, "canonical_hex")),
        &expected,
        usize::try_from(unsigned(data, "payload_bytes")).expect("payload bytes"),
    );
    let expected = match text(data, "expected_code") {
        "unknown_tag" => TagRefusal::Unknown,
        "truncated" => TagRefusal::Truncated,
        "trailing_bytes" => TagRefusal::Trailing,
        "tag_order" => TagRefusal::Order,
        code => panic!("unknown tag refusal code {code}"),
    };
    assert_eq!(actual, Err(expected));
}

fn verify_bound_case(data: &Value) {
    let name = text(data, "bound");
    let (operation, maximum, expected_code) = bound_contract(name);
    assert_eq!(text(data, "expected_code"), expected_code);
    let accepted = &data["accepted_recipe"];
    let refused = &data["refused_recipe"];
    assert_eq!(text(accepted, "operation"), operation);
    assert_eq!(text(refused, "operation"), operation);
    assert_eq!(measure_bound_recipe(name, accepted), maximum);
    assert_eq!(measure_bound_recipe(name, refused), maximum + 1);
    assert_eq!(execute_bound_recipe(name, accepted), Ok(()));
    assert_eq!(execute_bound_recipe(name, refused), Err(expected_code));
}

fn recipe_count(recipe: &Value, field: &str) -> u64 {
    recipe.get(field).and_then(Value::as_u64).unwrap_or(0)
}

fn recipe_counts(recipe: &Value, field: &str, maximum: usize) -> Vec<u64> {
    let rows = recipe[field].as_array().expect("bound recipe count rows");
    assert!(rows.len() <= maximum, "bounded recipe count rows");
    rows.iter()
        .take(maximum)
        .map(|value| value.as_u64().expect("bound recipe count"))
        .collect()
}

fn sum_fields(recipe: &Value, fields: &[&str]) -> u64 {
    assert!(fields.len() <= 8, "bounded recipe fields");
    fields
        .iter()
        .take(8)
        .map(|field| recipe_count(recipe, field))
        .sum()
}

fn framed_length(length: u64) -> u64 {
    let digits = if length == 0 {
        1
    } else {
        u64::from(length.ilog10()) + 1
    };
    digits + 1 + length
}

fn measure_bound_recipe(name: &str, recipe: &Value) -> u64 {
    match name {
        "vector_rows" => recipe_count(recipe, "row_count"),
        "vector_line_bytes" => recipe_count(recipe, "line_bytes"),
        "replay_session_bytes" => recipe_count(recipe, "session_bytes"),
        "rng_domain_bytes" | "qname_bytes" => {
            let segments = recipe_counts(recipe, "segment_bytes", 5);
            segments.iter().take(5).sum::<u64>() + segments.len().saturating_sub(1) as u64
        }
        "rng_domain_segments" | "qname_segments" => {
            recipe_counts(recipe, "segment_bytes", 5).len() as u64
        }
        "symbol_bytes" => recipe_count(recipe, "symbol_bytes"),
        "structural_type_bytes" | "enum_type_bytes" => recipe_count(recipe, "type_bytes"),
        "intrinsic_identity_bytes" => recipe_count(recipe, "identity_bytes"),
        "enum_member_bytes" => recipe_count(recipe, "member_bytes"),
        "governance_utf8_bytes" => recipe_count(recipe, "utf8_bytes"),
        "stable_carrier_active_elements" => recipe_count(recipe, "active_element_count"),
        "stable_carrier_bytes" => measure_carrier_bytes(recipe),
        "resolver_rows" => measure_resolver_rows(recipe),
        "resolver_edges" => recipe_count(recipe, "edge_rows"),
        "resolver_hyperedge_members" => recipe_count(recipe, "hyperedge_member_rows"),
        "resolver_fact_units" => measure_resolver_facts(recipe),
        "resolver_manifest_bytes" => measure_resolver_manifest_bytes(recipe),
        "stable_graph_elements" => recipe_count(recipe, "state_node_rows"),
        "stable_graph_attribute_rows" => recipe_count(recipe, "node_f64_rows"),
        "stable_graph_hyperedge_members" => recipe_count(recipe, "state_member_rows"),
        "stable_graph_fact_units" => measure_graph_facts(recipe),
        "stable_graph_bytes" => measure_stable_graph_bytes(recipe),
        "ordered_action_items" => recipe_count(recipe, "item_count"),
        "ordered_action_batch_bytes" => measure_ordered_action_batch_bytes(recipe),
        "prepared_rows" => recipe_count(recipe, "constant_rows"),
        "prepared_small_rows" => recipe_count(recipe, "intrinsic_rows"),
        "prepared_enum_members" => recipe_count(recipe, "enum_member_rows"),
        "prepared_vocabulary_members" => recipe_count(recipe, "vocabulary_member_rows"),
        "prepared_aggregate_rows" => measure_prepared_aggregate(recipe),
        "prepared_combined_bytes" => measure_prepared_bytes(recipe),
        "tick_rule_outcomes" => recipe_count(recipe, "rule_outcome_rows"),
        "tick_rows" => recipe_count(recipe, "event_rows"),
        "tick_aggregate_rows" => {
            recipe_count(recipe, "event_rows") + recipe_count(recipe, "payload_rows")
        }
        "tick_combined_bytes" => measure_tick_bytes(recipe),
        _ => panic!("unknown bound recipe {name}"),
    }
}

fn measure_carrier_bytes(recipe: &Value) -> u64 {
    let scenario = recipe_count(recipe, "scenario_bytes");
    let subject_name = recipe_count(recipe, "subject_local_name_bytes");
    let edge_type = recipe_count(recipe, "active_edge_type_bytes");
    let endpoint = recipe_count(recipe, "active_endpoint_name_bytes");
    let subject_segment = [4, scenario, subject_name]
        .iter()
        .map(|length| framed_length(*length))
        .sum::<u64>()
        + 2;
    let edge_segment = [4, scenario, edge_type, endpoint, endpoint]
        .iter()
        .map(|length| framed_length(*length))
        .sum::<u64>()
        + 4;
    let active = recipe_count(recipe, "active_element_count");
    let slot = integer(recipe, "draw_slot").to_string().len() as u64;
    framed_length(subject_segment)
        + active * (1 + framed_length(edge_segment))
        + 1
        + framed_length(slot)
}

fn measure_resolver_rows(recipe: &Value) -> u64 {
    let hyperedges = match text(recipe, "fixture") {
        "resolver_single_hyperedge" => 1,
        "resolver_fact_units" => recipe_counts(recipe, "hyperedge_member_rows", 32).len() as u64,
        _ => 0,
    };
    recipe_count(recipe, "node_rows") + hyperedges
}

fn measure_resolver_facts(recipe: &Value) -> u64 {
    let members = recipe_counts(recipe, "hyperedge_member_rows", 32);
    recipe_count(recipe, "node_rows") + members.len() as u64 + members.iter().sum::<u64>()
}

fn measure_resolver_manifest_bytes(recipe: &Value) -> u64 {
    let full_row = 8
        + recipe_count(recipe, "full_node_name_bytes")
        + recipe_count(recipe, "full_node_type_bytes");
    let final_row = 8
        + recipe_count(recipe, "final_node_name_bytes")
        + recipe_count(recipe, "final_node_type_bytes");
    31 + 20
        + text(recipe, "scenario").len() as u64
        + recipe_count(recipe, "full_node_rows") * full_row
        + final_row
}

fn measure_graph_facts(recipe: &Value) -> u64 {
    sum_fields(
        recipe,
        &["node_rows", "node_f64_rows", "node_currency_rows"],
    )
}

fn measure_stable_graph_bytes(recipe: &Value) -> u64 {
    let fixed = 20 + 1 + 4 + 1 + 4 + text(recipe, "scenario").len() as u64 + 7 * 5;
    let node = 8 + text(recipe, "node_name").len() as u64 + text(recipe, "node_type").len() as u64;
    let full = 17 + recipe_count(recipe, "full_qname_bytes");
    let final_row = 17 + recipe_count(recipe, "final_qname_bytes");
    fixed + node + recipe_count(recipe, "full_node_f64_rows") * full + final_row
}

fn measure_ordered_action_batch_bytes(recipe: &Value) -> u64 {
    let intent = 187 + 32 * recipe_count(recipe, "evidence_digests_per_intent");
    55 + recipe_count(recipe, "session_bytes") + recipe_count(recipe, "item_count") * (36 + intent)
}

fn measure_prepared_aggregate(recipe: &Value) -> u64 {
    sum_fields(
        recipe,
        &[
            "vocabulary_kind_rows",
            "node_type_members",
            "event_type_members",
        ],
    )
}

fn measure_prepared_bytes(recipe: &Value) -> u64 {
    let full = 16 + 1 + 3 * recipe_count(recipe, "full_governance_field_bytes");
    let final_row = 16
        + 1
        + sum_fields(
            recipe,
            &[
                "final_reason_bytes",
                "final_owner_bytes",
                "final_date_bytes",
            ],
        );
    21 + recipe_count(recipe, "full_exemption_rows") * full + final_row
}

fn measure_tick_bytes(recipe: &Value) -> u64 {
    let fixed = 14 + 4 + 10 + text(recipe, "event_name").len() as u64 + 4;
    let full = 13 + recipe_count(recipe, "full_payload_label_bytes");
    let final_row = 13 + recipe_count(recipe, "final_payload_label_bytes");
    fixed + recipe_count(recipe, "full_payload_rows") * full + final_row
}

fn execute_bound_recipe(name: &str, recipe: &Value) -> Result<(), &'static str> {
    if matches!(name, "vector_rows" | "vector_line_bytes") {
        return execute_vector_loader_bound_recipe(name, recipe);
    }
    if matches!(
        name,
        "replay_session_bytes"
            | "symbol_bytes"
            | "structural_type_bytes"
            | "intrinsic_identity_bytes"
            | "enum_type_bytes"
            | "enum_member_bytes"
            | "governance_utf8_bytes"
            | "qname_bytes"
            | "qname_segments"
            | "rng_domain_bytes"
            | "rng_domain_segments"
    ) {
        return bound_operations::execute(name, recipe);
    }
    if name.starts_with("resolver_")
        || name.starts_with("stable_graph_")
        || name.starts_with("prepared_")
        || name.starts_with("tick_")
        || matches!(
            name,
            "stable_carrier_active_elements"
                | "stable_carrier_bytes"
                | "ordered_action_items"
                | "ordered_action_batch_bytes"
        )
    {
        return bound_operations::execute(name, recipe);
    }
    panic!("unexecuted bound operation {name}")
}

fn execute_vector_loader_bound_recipe(name: &str, recipe: &Value) -> Result<(), &'static str> {
    let input = match name {
        "vector_rows" => {
            let count = usize::try_from(recipe_count(recipe, "row_count"))
                .expect("bounded vector row recipe");
            assert!(count <= MAX_ROWS + 1, "bounded vector row recipe");
            (0..=MAX_ROWS)
                .take(count)
                .map(|_| r#"{"id":"x","kind":"x","data":{}}"#)
                .collect::<Vec<_>>()
                .join("\n")
        }
        "vector_line_bytes" => super::valid_vector_line(
            usize::try_from(recipe_count(recipe, "line_bytes"))
                .expect("bounded vector line recipe"),
        ),
        _ => panic!("non-loader bound {name}"),
    };
    match super::parse_vector_rows(&input) {
        Ok(_) => Ok(()),
        Err(code) => Err(code),
    }
}

fn bound_contract(name: &str) -> (&'static str, u64, &'static str) {
    match name {
        "vector_rows" => ("load_vector_file", 256, "too_many_rows"),
        "vector_line_bytes" => ("load_vector_line", 262_144, "invalid_line_length"),
        "replay_session_bytes" => ("encode_replay_session", 256, "string_too_long"),
        "rng_domain_bytes" => ("parse_rng_domain", 128, "rng_domain_length"),
        "rng_domain_segments" => ("parse_rng_domain", 4, "rng_domain_segments"),
        "symbol_bytes" => ("parse_symbol", 64, "invalid_symbol"),
        "qname_bytes" => ("parse_qname", 128, "invalid_qname"),
        "qname_segments" => ("parse_qname", 4, "invalid_qname"),
        "structural_type_bytes" => ("parse_structural_type", 128, "invalid_structural_type"),
        "intrinsic_identity_bytes" => (
            "encode_intrinsic_identity",
            96,
            "invalid_intrinsic_identity",
        ),
        "enum_type_bytes" => ("encode_enum_type", 64, "invalid_enum_type"),
        "enum_member_bytes" => ("encode_enum_member", 64, "invalid_enum_member"),
        "governance_utf8_bytes" => (
            "encode_governance_text",
            4_194_304,
            "governance_string_too_long",
        ),
        "stable_carrier_active_elements" => ("compose_stable_carrier", 256, "active_element_limit"),
        "stable_carrier_bytes" => ("compose_stable_carrier", 105_962, "byte_limit"),
        "resolver_rows" => ("seal_stable_resolver", 65_536, "row_limit"),
        "resolver_edges" => ("seal_stable_resolver", 65_536, "edge_limit"),
        "resolver_hyperedge_members" => ("seal_stable_resolver", 65_534, "hyperedge_members"),
        "resolver_fact_units" => ("seal_stable_resolver", 1_048_576, "aggregate_row_limit"),
        "resolver_manifest_bytes" => ("seal_stable_resolver", 8_388_608, "byte_limit"),
        "stable_graph_elements" => ("encode_stable_graph", 65_536, "row_limit"),
        "stable_graph_attribute_rows" => ("encode_stable_graph", 524_288, "row_limit"),
        "stable_graph_hyperedge_members" => ("encode_stable_graph", 65_534, "hyperedge_members"),
        "stable_graph_fact_units" => ("encode_stable_graph", 1_048_576, "aggregate_row_limit"),
        "stable_graph_bytes" => ("encode_stable_graph", 67_108_864, "byte_limit"),
        "ordered_action_items" => ("project_ordered_action_batch", 4_096, "row_limit"),
        "ordered_action_batch_bytes" => ("project_ordered_action_batch", 9_302_326, "byte_limit"),
        "prepared_rows" => ("encode_prepared_bsl", 65_536, "row_limit"),
        "prepared_small_rows" => ("encode_prepared_bsl", 64, "row_limit"),
        "prepared_enum_members" => ("encode_prepared_bsl", 4_096, "row_limit"),
        "prepared_vocabulary_members" => ("encode_prepared_bsl", 524_288, "row_limit"),
        "prepared_aggregate_rows" => ("encode_prepared_bsl", 1_048_576, "aggregate_row_limit"),
        "prepared_combined_bytes" => ("encode_prepared_bsl", 67_108_864, "byte_limit"),
        "tick_rule_outcomes" => ("encode_tick_payload", 65_536, "row_limit"),
        "tick_rows" => ("encode_tick_payload", 1_048_576, "row_limit"),
        "tick_aggregate_rows" => ("encode_tick_payload", 1_048_576, "aggregate_row_limit"),
        "tick_combined_bytes" => ("encode_tick_payload", 67_108_864, "byte_limit"),
        _ => panic!("unknown bound refusal {name}"),
    }
}
