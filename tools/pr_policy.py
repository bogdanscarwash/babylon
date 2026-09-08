"""Typed, checked-in check manifests for Babylon pull-request policy."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Literal

CheckKind = Literal["blocking", "advisory"]


@dataclass(frozen=True, slots=True)
class CheckProducer:
    """Immutable GitHub App identity allowed to produce one check."""

    integration_id: int
    slug: str


@dataclass(frozen=True, slots=True)
class CheckRequirement:
    """One expected check and its exact accepted conclusions."""

    context: str
    kind: CheckKind
    allowed_conclusions: frozenset[str]
    producer: CheckProducer


BASELINE_CEREMONY_CONTEXT: Final[str] = "Baseline Ceremony Gate (§6.5 provenance)"
GITHUB_ACTIONS_PRODUCER: Final[CheckProducer] = CheckProducer(
    integration_id=15368,
    slug="github-actions",
)

# These jobs may skip only when their owning successful CI Gate accepted scope.
# GitHub may display a skipped matrix parent before expanding its focus value.
SCOPED_CI_CHECKS: Final[frozenset[str]] = frozenset(
    {
        "Rust Validation",
        "Python Tooling Tests",
        "Security Audit (pip-audit policy — blocking since item-41)",
        "PostgreSQL Contract",
        "PostgreSQL Contract ()",
        "PostgreSQL Contract (${{ matrix.focus }})",
        "PostgreSQL Contract (runtime_smoke)",
    }
)


DEV_CHECK_MANIFEST: Final[tuple[CheckRequirement, ...]] = (
    CheckRequirement(
        "CI Gate",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
)

MAIN_QUALIFICATION_CHECK_MANIFEST: Final[tuple[CheckRequirement, ...]] = (
    CheckRequirement(
        "Main Qualification / Native Download / Linux x86_64",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
    CheckRequirement(
        "Main Qualification / Event Contract",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
    CheckRequirement(
        "Main Qualification / Non-Unit Behavioral Contracts",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
    CheckRequirement(
        "Main Qualification / PostgreSQL Determinism Bundle",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
    CheckRequirement(
        "Main Qualification / Reference-Data Contracts",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
    CheckRequirement(
        "Main Qualification / Release Documentation",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
    CheckRequirement(
        "Main Qualification / Container Image Scan",
        "blocking",
        frozenset({"SUCCESS"}),
        GITHUB_ACTIONS_PRODUCER,
    ),
)

MAIN_CHECK_MANIFEST: Final[tuple[CheckRequirement, ...]] = (
    *DEV_CHECK_MANIFEST,
    *MAIN_QUALIFICATION_CHECK_MANIFEST,
)

DEV_BLOCKING_CONTEXTS: Final[tuple[str, ...]] = tuple(
    requirement.context for requirement in DEV_CHECK_MANIFEST if requirement.kind == "blocking"
)

MAIN_BLOCKING_CONTEXTS: Final[tuple[str, ...]] = tuple(
    requirement.context for requirement in MAIN_CHECK_MANIFEST if requirement.kind == "blocking"
)


def manifest_for_base(base_ref: str) -> tuple[CheckRequirement, ...]:
    """Return the exact expected manifest for one sanctioned base."""
    if base_ref == "dev":
        return DEV_CHECK_MANIFEST
    if base_ref == "main":
        return MAIN_CHECK_MANIFEST
    raise ValueError(f"no check manifest for base {base_ref!r}")
