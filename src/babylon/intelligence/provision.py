"""Model provisioning: fetch weights per the signed manifest (D3, ADR096).

``babylon doctor --provision`` calls :func:`provision_models`. Downloads are
resumable (HTTP Range onto a ``.part`` file), sha256-verified before an atomic
rename-into-place, and bounded-retry. The fetcher is injected — the default
uses stdlib ``urllib`` with a Range header; tests inject a fake, so the core
carries zero network dependency.

Weights land in ``$XDG_DATA_HOME/babylon/models/`` (default
``~/.local/share/babylon/models/``); they are separate from the installed package.
"""

from __future__ import annotations

import hashlib
import logging
import urllib.request
from collections.abc import Callable, Iterator, Mapping
from pathlib import Path

from pydantic import BaseModel, ConfigDict

from babylon.config.paths import player_data_dir
from babylon.intelligence.model_manifest import ModelEntry, ModelKind, ModelManifest

logger = logging.getLogger("babylon.intelligence.provision")

#: (url, start_byte) -> byte chunks starting at ``start_byte`` (Range resume).
Fetcher = Callable[[str, int], Iterator[bytes]]

#: Bounded read loop cap: files are large but chunk count is bounded by size /
#: chunk; this fixed upper bound guards against a non-terminating stream.
_MAX_CHUNKS: int = 10_000_000
_CHUNK_BYTES: int = 1 << 20  # 1 MiB


class ProvisionResult(BaseModel):
    model_config = ConfigDict(frozen=True)

    name: str
    status: str  # "downloaded" | "skipped" | "gated"
    detail: str = ""


def default_models_dir(env: Mapping[str, str] | None = None) -> Path:
    """``$XDG_DATA_HOME/babylon/models`` else ``~/.local/share/babylon/models``."""
    return player_data_dir(env) / "models"


def _ext_for(kind: ModelKind) -> str:  # noqa: ARG001 — reserved for a future non-gguf kind
    return ".gguf"  # both chat and embed lanes ship gguf weights


def _default_fetcher(url: str, start: int) -> Iterator[bytes]:
    request = urllib.request.Request(url)  # noqa: S310 — manifest-pinned URL
    if start > 0:
        request.add_header("Range", f"bytes={start}-")
    with urllib.request.urlopen(request) as response:  # noqa: S310 - manifest-pinned URL
        for _ in range(_MAX_CHUNKS):
            chunk = response.read(_CHUNK_BYTES)
            if not chunk:
                return
            yield chunk
        raise ValueError(f"download of {url} exceeded {_MAX_CHUNKS} chunks — refusing")


def _download_one(
    entry: ModelEntry, dest_dir: Path, fetcher: Fetcher, max_retries: int
) -> ProvisionResult:
    assert entry.url is not None and entry.sha256 is not None  # noqa: S101 — available=>validated by ModelEntry._check_completeness
    final_path = dest_dir / f"{entry.name}{_ext_for(entry.kind)}"
    part_path = dest_dir / f"{entry.name}{_ext_for(entry.kind)}.part"
    if final_path.exists():
        return ProvisionResult(name=entry.name, status="skipped", detail="already present")

    last_error = ""
    for _attempt in range(max_retries):
        start = part_path.stat().st_size if part_path.exists() else 0
        hasher = hashlib.sha256()
        if start > 0:
            hasher.update(part_path.read_bytes())
        with part_path.open("ab") as handle:
            for chunk in fetcher(entry.url, start):
                handle.write(chunk)
                hasher.update(chunk)
        digest = hasher.hexdigest()
        if digest == entry.sha256:
            part_path.replace(final_path)  # atomic rename-into-place
            return ProvisionResult(name=entry.name, status="downloaded", detail=digest)
        last_error = f"sha256 mismatch: got {digest}, expected {entry.sha256}"
        part_path.unlink(missing_ok=True)  # corrupt — restart clean next attempt
        logger.warning("provision %s: %s (attempt %d)", entry.name, last_error, _attempt + 1)
    raise ValueError(f"provision {entry.name} failed after {max_retries} attempts: {last_error}")


def provision_models(
    manifest: ModelManifest,
    dest_dir: Path,
    *,
    fetcher: Fetcher | None = None,
    max_retries: int = 3,
) -> list[ProvisionResult]:
    """Provision every manifest entry into ``dest_dir``.

    Unavailable (owner-provisioned) entries are reported ``gated`` and never
    fetched — the loud signal that the owner has not yet uploaded weights to
    the babylon-data R2 bucket. Available entries are downloaded (resumable),
    sha256-verified, and renamed into place.
    """
    dest_dir.mkdir(parents=True, exist_ok=True)
    fetch = fetcher or _default_fetcher
    results: list[ProvisionResult] = []
    for entry in manifest.models:
        if not entry.available:
            results.append(
                ProvisionResult(
                    name=entry.name,
                    status="gated",
                    detail="owner-provisioned: no weights uploaded to R2 yet",
                )
            )
            continue
        results.append(_download_one(entry, dest_dir, fetch, max_retries))
    return results
