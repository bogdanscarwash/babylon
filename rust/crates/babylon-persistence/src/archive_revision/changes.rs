//! Differences in retained assertions, never a reconstruction of missing history.

use std::collections::BTreeMap;

use super::record::RevisionRecord;
use super::ArchiveAtomChangeV2;
use crate::{ArchiveAtomV1, ArchiveAtomValueV1, SemanticArchiveErrorV1};

/// One page has at most 513 atoms, so a complete difference has at most 1,026 rows.
pub(super) fn between(
    previous: Option<&RevisionRecord>,
    next: &RevisionRecord,
) -> Result<Vec<ArchiveAtomChangeV2>, SemanticArchiveErrorV1> {
    next.validate()?;
    if let Some(previous) = previous {
        previous.validate()?;
        if previous.source.campaign_id() != next.source.campaign_id()
            || previous.subject != next.subject
            || (previous.effective_tick, previous.origin.tag())
                >= (next.effective_tick, next.origin.tag())
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
    }
    let before = previous.map_or_else(|| Ok(BTreeMap::new()), |page| indexed(&page.atoms))?;
    let mut after = indexed(&next.atoms)?;
    let mut changes = Vec::new();
    for (principal, before) in before {
        let current = after.remove(&principal);
        if current.is_none_or(|after| !same_assertion(before, after)) {
            changes.push(ArchiveAtomChangeV2 {
                publication_tick: next.effective_tick,
                signal_key: before.signal_key().to_owned(),
                before: Some(before.clone()),
                after: current.cloned(),
            });
        }
    }
    changes.extend(after.into_values().map(|atom| ArchiveAtomChangeV2 {
        publication_tick: next.effective_tick,
        signal_key: atom.signal_key().to_owned(),
        before: None,
        after: Some(atom.clone()),
    }));
    changes.sort_by(|left, right| change_key(left).cmp(&change_key(right)));
    Ok(changes)
}

type AtomIndex<'a> = BTreeMap<(u8, &'a str, &'a str), &'a ArchiveAtomV1>;

fn indexed(atoms: &[ArchiveAtomV1]) -> Result<AtomIndex<'_>, SemanticArchiveErrorV1> {
    let mut result = BTreeMap::new();
    for (position, atom) in atoms.iter().enumerate() {
        let (key, target) = atom_key(atom);
        let role = if position == 0 {
            0
        } else if super::record::is_link(atom) {
            2
        } else {
            1
        };
        if result.insert((role, key, target), atom).is_some() {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
    }
    Ok(result)
}

fn atom_key(atom: &ArchiveAtomV1) -> (&str, &str) {
    let target = if super::record::is_link(atom) {
        match atom.value() {
            ArchiveAtomValueV1::Text(target) => target.as_str(),
            _ => "",
        }
    } else {
        ""
    };
    (atom.signal_key(), target)
}

fn change_key(change: &ArchiveAtomChangeV2) -> (&str, &str) {
    change
        .after
        .as_ref()
        .or(change.before.as_ref())
        .map_or(("", ""), atom_key)
}

fn same_assertion(left: &ArchiveAtomV1, right: &ArchiveAtomV1) -> bool {
    left.signal_key() == right.signal_key()
        && left.grant_key() == right.grant_key()
        && left.evidence_class() == right.evidence_class()
        && left.value() == right.value()
        && left.citation() == right.citation()
}
