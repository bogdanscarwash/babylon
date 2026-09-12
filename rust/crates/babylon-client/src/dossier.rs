//! One scoped Archive observation shared by native composition and headless JSONL.

use std::io::Write;
use std::num::NonZero;

use babylon_persistence::archive_revision::{
    ArchiveAtomChange, ArchiveDossierBounds, ArchiveDossierPage, ArchiveDossierPending,
    ArchiveDossierRead, ArchiveDossierState, ArchiveDossierUnavailable, ArchiveLinkedPageState,
    ArchiveReadScope, ArchiveSearchState,
};
use babylon_persistence::{
    identity::CampaignId, ArchiveAtom, ArchivePageRef, ArchiveSubjectKind, SemanticArchiveReader,
};
use bevy::app::AppExit;
use bevy::prelude::{MessageWriter, Res, Resource};
use serde_json::{json, Value};

use crate::cli::CliCommand;

/// The retained page is readable while pending, but that is not verification.
#[must_use]
pub fn retained_page(read: &ArchiveDossierRead) -> Option<&ArchiveDossierPage> {
    match &read.state {
        ArchiveDossierState::Ready { page, .. }
        | ArchiveDossierState::Pending {
            page: Some(page), ..
        } => Some(page),
        ArchiveDossierState::Pending { page: None, .. } | ArchiveDossierState::Unavailable(_) => {
            None
        }
    }
}

/// Only the reader's exact selected-page result certifies this observation.
#[must_use]
pub const fn verified_tick(read: &ArchiveDossierRead) -> Option<u64> {
    match &read.state {
        ArchiveDossierState::Ready {
            verified_through_tick,
            ..
        } => Some(*verified_through_tick),
        _ => None,
    }
}

/// Static availability wording shared by the card and CLI.
#[must_use]
pub const fn availability_label(read: &ArchiveDossierRead) -> &'static str {
    match &read.state {
        ArchiveDossierState::Ready { .. } => "Verified for this viewed period",
        ArchiveDossierState::Pending { reason, .. } => pending_label(*reason),
        ArchiveDossierState::Unavailable(reason) => unavailable_label(*reason),
    }
}

pub(crate) const fn pending_label(reason: ArchiveDossierPending) -> &'static str {
    match reason {
        ArchiveDossierPending::ReceiptProcessing => "Archive is still processing this observation",
        ArchiveDossierPending::KnowledgeRefresh => {
            "Newly learned information awaits Archive publication"
        }
    }
}

pub(crate) const fn unavailable_label(reason: ArchiveDossierUnavailable) -> &'static str {
    match reason {
        ArchiveDossierUnavailable::FoundationHasNoPage => {
            "The campaign foundation has no published Archive page"
        }
        ArchiveDossierUnavailable::SubjectNotDisclosed => {
            "This subject is not disclosed in this observation"
        }
        ArchiveDossierUnavailable::PageNotMaterialized => {
            "No Archive page has been published for this subject at this period"
        }
    }
}

pub(crate) const fn link_state_label(state: ArchiveLinkedPageState) -> &'static str {
    match state {
        ArchiveLinkedPageState::Unknown => "unknown",
        ArchiveLinkedPageState::KnownUnavailable => "unavailable",
        ArchiveLinkedPageState::KnownPending => "pending",
        ArchiveLinkedPageState::KnownReady => "ready",
    }
}

/// Bind the exact installed observation, never the session's newer durable hash.
///
/// # Errors
/// Refuses a missing or noncanonical positive-tick identity, or a fabricated foundation hash.
pub fn observation_scope(
    campaign: CampaignId,
    tick: u64,
    hash: Option<&str>,
) -> Result<ArchiveReadScope, String> {
    if tick == 0 {
        return if hash.is_none() {
            Ok(ArchiveReadScope::foundation(campaign))
        } else {
            Err("Foundation cannot carry a committed tick hash".into())
        };
    }
    let hash = hash.ok_or("The installed observation has no committed identity")?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("The installed observation has a noncanonical committed identity".into());
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hash[index * 2..index * 2 + 2], 16)
            .map_err(|_| "The installed observation has an invalid committed identity")?;
    }
    ArchiveReadScope::committed(campaign, tick, bytes).map_err(|error| error.to_string())
}

/// One CLI invocation; live commands pin a marker once before their scoped reads.
#[derive(Resource, Clone, Debug)]
pub struct HeadlessInvocation {
    command: CliCommand,
    campaign_id: CampaignId,
}
impl HeadlessInvocation {
    /// Capture one parsed command and campaign.
    #[must_use]
    pub const fn new(command: CliCommand, campaign_id: CampaignId) -> Self {
        Self {
            command,
            campaign_id,
        }
    }
}

/// Execute a scoped command, returning a process exit code.
#[must_use]
pub fn run_headless(invocation: &HeadlessInvocation) -> u8 {
    match execute(invocation) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("babylon-client: {message}");
            2
        }
    }
}

pub(crate) fn pinned_scope(
    reader: &SemanticArchiveReader,
    campaign: CampaignId,
) -> Result<ArchiveReadScope, String> {
    reader
        .committed_tick_status(campaign)
        .map_err(|error| error.to_string())?
        .map_or_else(
            || Ok(ArchiveReadScope::foundation(campaign)),
            |status| {
                ArchiveReadScope::committed(
                    campaign,
                    status.resolve_tick(),
                    *status.tick_content_hash(),
                )
                .map_err(|error| error.to_string())
            },
        )
}

fn county_subject(geoid: &str) -> Result<ArchivePageRef, String> {
    ArchivePageRef::try_new(ArchiveSubjectKind::County, geoid.into())
        .map_err(|error| error.to_string())
}

fn execute(invocation: &HeadlessInvocation) -> Result<(), String> {
    let reader = SemanticArchiveReader::from_env().map_err(|error| error.to_string())?;
    let mut out = std::io::stdout().lock();
    if invocation.command == CliCommand::TickStatus {
        write_jsonl(&mut out, &tick_status_row(&reader, invocation.campaign_id)?)?;
    } else {
        let scope = pinned_scope(&reader, invocation.campaign_id)?;
        match &invocation.command {
            CliCommand::DossierShow { geoid } => {
                let read = reader
                    .dossier_as_of(
                        &scope,
                        &county_subject(geoid)?,
                        &ArchiveDossierBounds::default(),
                    )
                    .map_err(|error| error.to_string())?;
                write_jsonl(&mut out, &dossier_json(&read))?;
            }
            CliCommand::DossierSearch { query } => write_search(&mut out, &reader, &scope, query)?,
            CliCommand::Changelog { geoid } => {
                write_changes(&mut out, &reader, &scope, &county_subject(geoid)?)?;
            }
            CliCommand::TickStatus => unreachable!("handled above"),
        }
    }
    out.flush().map_err(|error| error.to_string())
}

fn write_search(
    out: &mut impl Write,
    reader: &SemanticArchiveReader,
    scope: &ArchiveReadScope,
    query: &str,
) -> Result<(), String> {
    let read = reader
        .search_as_of(scope, query, 50)
        .map_err(|error| error.to_string())?;
    let (state, reason) = match read.state {
        ArchiveSearchState::Ready => ("ready", None),
        ArchiveSearchState::Pending(reason) => ("pending", Some(pending_label(reason))),
        ArchiveSearchState::Unavailable(reason) => ("unavailable", Some(unavailable_label(reason))),
    };
    write_jsonl(
        out,
        &json!({"record":"archive-search-status", "scope":scope_json(&read.scope),
        "state":state,"reason":reason,"durable_tick":read.durable_tick,"processed_tick":read.processed_tick,
        "truncated":read.truncated}),
    )?;
    for hit in read.hits {
        write_jsonl(
            out,
            &json!({"record":"search-hit","subject_kind":hit.subject.kind().as_str(),
            "geoid":hit.subject.id(),"title":hit.title,"revision_id":hex_bytes(hit.revision_id),
            "content_source":scope_json(&hit.content_source)}),
        )?;
    }
    Ok(())
}

fn write_changes(
    out: &mut impl Write,
    reader: &SemanticArchiveReader,
    scope: &ArchiveReadScope,
    subject: &ArchivePageRef,
) -> Result<(), String> {
    let mut cursor = None;
    loop {
        let bounds =
            ArchiveDossierBounds::try_new(32, cursor.clone()).map_err(|error| error.to_string())?;
        let read = reader
            .dossier_as_of(scope, subject, &bounds)
            .map_err(|error| error.to_string())?;
        write_jsonl(
            out,
            &json!({"record":"archive-changes-page","scope":scope_json(&read.scope),
            "subject":read.subject,"availability":availability_label(&read),
            "coverage_from_tick":retained_page(&read).map(|page|page.changes.coverage_from_tick),
            "has_more":retained_page(&read).is_some_and(|page|page.changes.next_cursor.is_some())}),
        )?;
        let Some(page) = retained_page(&read) else {
            return Ok(());
        };
        for change in &page.changes.changes {
            write_jsonl(out, &change_json(change))?;
        }
        let Some(next) = &page.changes.next_cursor else {
            return Ok(());
        };
        if cursor.as_ref() == Some(next) {
            return Err("Archive continuation did not advance".into());
        }
        cursor = Some(next.clone());
    }
}

/// Execute the one headless invocation and request application exit.
pub fn run_headless_command(invocation: Res<HeadlessInvocation>, mut exit: MessageWriter<AppExit>) {
    let code = run_headless(&invocation);
    exit.write(match NonZero::<u8>::new(code) {
        Some(code) => AppExit::Error(code),
        None => AppExit::Success,
    });
}

pub(crate) fn scope_json(scope: &ArchiveReadScope) -> Value {
    json!({"campaign_id":scope.campaign_id().as_uuid().to_string(),"tick":scope.tick(),
        "tick_content_hash":scope.tick_content_hash().map(hex_bytes)})
}

pub(crate) fn dossier_json(read: &ArchiveDossierRead) -> Value {
    let page = retained_page(read);
    let state = match read.state {
        ArchiveDossierState::Ready { .. } => "ready",
        ArchiveDossierState::Pending { .. } => "pending",
        ArchiveDossierState::Unavailable(_) => "unavailable",
    };
    json!({"record":"county-dossier","schema_version":2,"scope":scope_json(&read.scope),"subject":read.subject,
        "geoid":read.subject.id(),"state":state,"availability":availability_label(read),
        "durable_tick":read.durable_tick,"processed_tick":read.processed_tick,"verified_tick":verified_tick(read),
        "page":page.map(|page| json!({"title":page.title,"question":page.question,
            "revision_id":hex_bytes(page.revision_id),"content_source":scope_json(&page.content_source),
            "content_sha256":hex_bytes(page.content_sha256),"effective_tick":page.effective_tick,"markdown":page.markdown,
            "citations":page.citations.iter().map(|citation|json!({"source_id":citation.source_id(),"locator":citation.locator()})).collect::<Vec<_>>(),
            "signals":page.signals.iter().map(|signal|json!({"grant_key":signal.grant_key(),"label":signal.label(),
                "value":signal.value(),"citation":{"source_id":signal.citation().source_id(),"locator":signal.citation().locator()}})).collect::<Vec<_>>(),
            "atoms":page.atoms.iter().map(atom_json).collect::<Vec<_>>(),
            "links":page.links.iter().map(|link|json!({"target":link.target,"label":link.retained_label,
                "state":link_state_label(link.target_state)})).collect::<Vec<_>>(),
            "coverage_from_tick":page.changes.coverage_from_tick,
            "changes":page.changes.changes.iter().map(change_json).collect::<Vec<_>>(),
            "has_more_changes":page.changes.next_cursor.is_some()}))})
}

pub(crate) fn change_json(change: &ArchiveAtomChange) -> Value {
    json!({"record":"changelog-row","publication_tick":change.publication_tick,"signal_key":change.signal_key,
        "before":change.before.as_ref().map(atom_json),"after":change.after.as_ref().map(atom_json)})
}

fn atom_json(atom: &ArchiveAtom) -> Value {
    json!({"signal_key":atom.signal_key(),"grant_key":atom.grant_key(),"evidence_class":atom.evidence_class().as_str(),
        "value":atom_value_json(atom.value()),"valid_tick":atom.valid_tick(),"atom_id":hex_bytes(atom.atom_id()),
        "citation":{"source_id":atom.citation().source_id(),"locator":atom.citation().locator()}})
}

pub(crate) fn atom_value_json(value: &babylon_persistence::ArchiveAtomValue) -> Value {
    match value {
        babylon_persistence::ArchiveAtomValue::Text(text) => Value::String(text.clone()),
        babylon_persistence::ArchiveAtomValue::F64(number) => {
            serde_json::Number::from_f64(*number).map_or(Value::Null, Value::Number)
        }
        babylon_persistence::ArchiveAtomValue::U64(number) => Value::from(*number),
        babylon_persistence::ArchiveAtomValue::Bool(flag) => Value::from(*flag),
    }
}

pub(crate) fn hex_bytes(bytes: [u8; 32]) -> String {
    let mut rendered = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(rendered, "{byte:02x}");
    }
    rendered
}

fn write_jsonl(out: &mut impl Write, value: &Value) -> Result<(), String> {
    writeln!(out, "{value}").map_err(|error| error.to_string())
}

fn tick_status_row(
    reader: &SemanticArchiveReader,
    campaign_id: CampaignId,
) -> Result<Value, String> {
    let status = reader
        .committed_tick_status(campaign_id)
        .map_err(|error| error.to_string())?;
    let processed_tick = reader
        .archive_verification_status(campaign_id)
        .map_err(|error| error.to_string())?
        .map(|status| status.processed_tick());
    Ok(match status {
        Some(status) => json!({
            "record": "tick-status",
            "campaign_id": status.campaign_id().as_uuid().to_string(),
            "durable_tick": status.resolve_tick(),
            "processed_tick": processed_tick,
            "envelope_layout_version": status.envelope_layout_version(),
            "tick_content_hash": hex_bytes(*status.tick_content_hash()),
            "envelope_digest": hex_bytes(*status.envelope_digest()),
        }),
        None => json!({
            "record": "tick-status",
            "campaign_id": campaign_id.as_uuid().to_string(),
            "durable_tick": Value::Null,
            "processed_tick": Value::Null,
            "envelope_layout_version": Value::Null,
            "tick_content_hash": Value::Null,
            "envelope_digest": Value::Null,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_persistence::archive_revision::ArchiveChangePage;
    use babylon_persistence::{
        ArchiveAtomSubject, ArchiveAtomSubjectKind, ArchiveAtomValue, ArchiveCitation,
        ArchiveEvidenceClass, ArchiveSignal,
    };

    fn read(state: ArchiveDossierState) -> ArchiveDossierRead {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        ArchiveDossierRead {
            scope: ArchiveReadScope::committed(campaign, 3, [0xab; 32]).unwrap(),
            subject: county_subject("26163").unwrap(),
            durable_tick: 8,
            processed_tick: 8,
            state,
        }
    }
    fn page() -> ArchiveDossierPage {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        let citation =
            ArchiveCitation::try_new("original-source".into(), "original/locator".into()).unwrap();
        ArchiveDossierPage {
            revision_id: [1; 32],
            effective_tick: 2,
            content_source: ArchiveReadScope::committed(campaign, 1, [0xaa; 32]).unwrap(),
            title: "Retained title".into(),
            question: "Original question?".into(),
            signals: vec![ArchiveSignal::try_new(
                "observed-key".into(),
                "Original label".into(),
                "Original display value".into(),
                citation.clone(),
            )
            .unwrap()],
            markdown: "Original narrative bytes".into(),
            content_sha256: [2; 32],
            citations: vec![citation],
            atoms: vec![],
            links: vec![],
            changes: ArchiveChangePage {
                coverage_from_tick: 2,
                changes: vec![],
                next_cursor: None,
            },
        }
    }
    #[test]
    fn pending_retained_content_never_inherits_global_progress_verification() {
        let read = read(ArchiveDossierState::Pending {
            page: Some(page()),
            reason: ArchiveDossierPending::KnowledgeRefresh,
        });
        assert!(retained_page(&read).is_some());
        assert_eq!(verified_tick(&read), None);
        let json = dossier_json(&read);
        assert_eq!(json["state"], "pending");
        assert!(json["verified_tick"].is_null());
        assert_eq!(json["processed_tick"], 8);
        assert_eq!(json["scope"]["tick"], 3);
        assert_eq!(json["page"]["content_source"]["tick"], 1);
        assert_eq!(json["page"]["question"], "Original question?");
        assert_eq!(json["page"]["signals"][0]["label"], "Original label");
        assert_eq!(json["page"]["markdown"], "Original narrative bytes");
    }
    #[test]
    fn ready_uses_exact_selected_page_verification_and_absence_stays_typed() {
        let ready = read(ArchiveDossierState::Ready {
            page: page(),
            verified_through_tick: 3,
        });
        assert_eq!(verified_tick(&ready), Some(3));
        for reason in [
            ArchiveDossierUnavailable::FoundationHasNoPage,
            ArchiveDossierUnavailable::SubjectNotDisclosed,
            ArchiveDossierUnavailable::PageNotMaterialized,
        ] {
            let read = read(ArchiveDossierState::Unavailable(reason));
            assert!(retained_page(&read).is_none());
            assert_eq!(verified_tick(&read), None);
            let json = dossier_json(&read);
            assert_eq!(json["state"], "unavailable");
            assert!(json["page"].is_null());
            assert_eq!(json["availability"], unavailable_label(reason));
        }
    }
    #[test]
    fn exact_scope_requires_the_selected_hash_and_cannot_fabricate_foundation() {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        assert!(observation_scope(campaign, 0, None)
            .unwrap()
            .tick_content_hash()
            .is_none());
        assert!(observation_scope(campaign, 0, Some(&"a".repeat(64))).is_err());
        for hash in [
            None,
            Some("bad"),
            Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
        ] {
            assert!(observation_scope(campaign, 1, hash).is_err());
        }
        assert_eq!(
            observation_scope(campaign, 3, Some(&"ab".repeat(32)))
                .unwrap()
                .tick_content_hash(),
            Some([0xab; 32])
        );
    }
    #[test]
    fn removal_and_numeric_zero_have_different_json_evidence() {
        let atom = ArchiveAtom::try_new(
            CampaignId::from_uuid(uuid::Uuid::nil()),
            ArchiveAtomSubject::try_new(ArchiveAtomSubjectKind::County, "26163".into()).unwrap(),
            "jobs".into(),
            "jobs".into(),
            ArchiveEvidenceClass::Observed,
            &ArchiveAtomValue::U64(0),
            ArchiveCitation::try_new("source".into(), "locator".into()).unwrap(),
            2,
        )
        .unwrap();
        let mut change = ArchiveAtomChange {
            publication_tick: 3,
            signal_key: "jobs".into(),
            before: None,
            after: Some(atom.clone()),
        };
        assert_eq!(change_json(&change)["after"]["value"], 0);
        change.before = Some(atom);
        change.after = None;
        let json = change_json(&change);
        assert!(json["after"].is_null());
        assert_eq!(json["before"]["valid_tick"], 2);
        assert_eq!(json["publication_tick"], 3);
    }
}
