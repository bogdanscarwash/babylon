//! Conservative witness candidates. Only an exact forward-render proof admits one.

use super::emission::{ArchiveEmissionLinkV2, ArchiveEmissionManifestV2};
use super::record::{is_link, parse_page_key, RevisionRecord};
use crate::{ArchiveAtomValueV1, ArchiveSignalV1, SemanticArchiveErrorV1};

pub(super) fn recover(
    record: &RevisionRecord,
) -> Result<Option<ArchiveEmissionManifestV2>, SemanticArchiveErrorV1> {
    record.validate()?;
    let Some(candidate) = candidate(record) else {
        return Ok(None);
    };
    match candidate.verify(record) {
        Ok(()) => Ok(Some(candidate)),
        Err(SemanticArchiveErrorV1::StoredPageMismatch) => Ok(None),
        Err(error) => Err(error),
    }
}

fn candidate(record: &RevisionRecord) -> Option<ArchiveEmissionManifestV2> {
    if record.template_sha256 != crate::ARCHIVE_PAGE_TEMPLATE_SHA256_V1 {
        return None;
    }
    let prefix = format!("---\nschema: babylon.archive-page.v1\nsubject: {}/{}\nverified_tick: {}\ntick_content_hash: {}\n---\n# {}\n",
        record.subject.kind().as_str(), record.subject.id(), record.source.tick(),
        crate::archive::hex_digest(&record.source.tick_content_hash()?), record.title);
    let body = record.markdown.strip_prefix(&prefix)?;
    let signals_at = body.find("\n## Signals\n");
    let links_at = body.find("\n## Related\n");
    if signals_at
        .zip(links_at)
        .is_some_and(|(signals, links)| links < signals)
    {
        return None;
    }
    let first_section = signals_at
        .into_iter()
        .chain(links_at)
        .min()
        .unwrap_or(body.len());
    let question = body[..first_section].trim_matches('\n').to_owned();
    let signals = signal_candidates(
        record,
        signals_at
            .map(|start| &body[start + "\n## Signals\n".len()..links_at.unwrap_or(body.len())]),
    )?;
    let links = link_candidates(links_at.map(|start| &body[start + "\n## Related\n".len()..]))?;
    ArchiveEmissionManifestV2::try_new(
        record.atoms.first()?.citation().clone(),
        question,
        signals,
        links,
    )
    .ok()
}

fn signal_candidates(
    record: &RevisionRecord,
    section: Option<&str>,
) -> Option<Vec<ArchiveSignalV1>> {
    let atoms: Vec<_> = record
        .atoms
        .iter()
        .skip(1)
        .filter(|atom| !is_link(atom))
        .collect();
    let lines: Vec<_> = section
        .unwrap_or("")
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    if atoms.len() != lines.len() {
        return None;
    }
    atoms
        .into_iter()
        .zip(lines)
        .map(|(atom, line)| {
            let ArchiveAtomValueV1::Text(value) = atom.value() else {
                return None;
            };
            let suffix = format!(
                ":** {value} — {}; {}",
                atom.citation().source_id(),
                atom.citation().locator()
            );
            let label = line.strip_prefix("- **")?.strip_suffix(&suffix)?;
            ArchiveSignalV1::try_new(
                atom.grant_key().to_owned(),
                label.to_owned(),
                value.clone(),
                atom.citation().clone(),
            )
            .ok()
        })
        .collect()
}

fn link_candidates(section: Option<&str>) -> Option<Vec<ArchiveEmissionLinkV2>> {
    let lines: Vec<_> = section
        .unwrap_or("")
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    if lines.len() > crate::archive::MAX_LINKS {
        return None;
    }
    lines
        .into_iter()
        .map(|line| {
            let (label, target) = line
                .strip_prefix("- [")?
                .strip_suffix(')')?
                .rsplit_once("](subject:")?;
            ArchiveEmissionLinkV2::try_new(
                parse_page_key(target).ok()?,
                (!label.is_empty()).then(|| label.to_owned()),
            )
            .ok()
        })
        .collect()
}
