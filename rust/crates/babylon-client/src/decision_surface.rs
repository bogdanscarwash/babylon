//! Contracts for the surfaces composed by the durable campaign observer.
//!
//! Observation and administrative presentation cannot satisfy a gameplay gate.
//! The county dossier declares the decision loop, but its Investigate action
//! stays visibly unavailable until Gate 5. `DeclaredSurface` binds rendered
//! entities to this manifest so the production composition is checked directly.

use bevy::prelude::Component;
use std::fmt;

/// Stable identifiers for the visible surfaces in the production client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceId {
    TitleLockup,
    CountyDossier,
    ObserverShell,
    ObserverProduction,
}

impl SurfaceId {
    /// Complete closed set used by the manifest-exhaustiveness sentinel.
    pub const ALL: [Self; 4] = [
        Self::TitleLockup,
        Self::CountyDossier,
        Self::ObserverShell,
        Self::ObserverProduction,
    ];
}

impl fmt::Display for SurfaceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TitleLockup => "title-lockup",
            Self::CountyDossier => "county-dossier",
            Self::ObserverShell => "observer-shell",
            Self::ObserverProduction => "observer-production",
        })
    }
}

/// Whether a surface is player-decision gameplay, semantic reference, or an
/// administrative/debug instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSurfaceRole {
    Gameplay,
    /// G4 observation and diagnosis; never certifies G5 player agency.
    Observer,
    ArchiveReference,
    AdminDebug,
}

/// Whether one surface action can be taken now, or is declared-but-sealed
/// with the honest reason shown to the player (ADR249 R9: Investigate appears
/// in the dossier's actions slot visibly unavailable until Gate 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionAvailability {
    /// The action can be taken on this surface today.
    Available,
    /// The action is declared but sealed; the carried reason is the
    /// player-facing honesty line.
    Unavailable(&'static str),
}

/// One player-facing action slot on a decision surface, with its typed
/// availability. The slot stays visible even while sealed: a declared,
/// honestly-unavailable action is presentation; an absent action slot is a
/// hole in the decision loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceAction {
    name: &'static str,
    availability: ActionAvailability,
}

impl SurfaceAction {
    /// Declare one actionable verb.
    #[must_use]
    pub const fn available(name: &'static str) -> Self {
        Self {
            name,
            availability: ActionAvailability::Available,
        }
    }

    /// Declare one sealed verb with its honest player-facing reason.
    #[must_use]
    pub const fn unavailable(name: &'static str, reason: &'static str) -> Self {
        Self {
            name,
            availability: ActionAvailability::Unavailable(reason),
        }
    }

    /// Borrow the stable verb name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Return the typed availability.
    #[must_use]
    pub const fn availability(&self) -> ActionAvailability {
        self.availability
    }

    /// Return whether the verb can be taken today.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self.availability, ActionAvailability::Available)
    }
}

/// One complete declaration of what a surface shows and what player loop it
/// closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecisionSurfaceContract {
    pub id: SurfaceId,
    pub role: DecisionSurfaceRole,
    pub decision_question: Option<&'static str>,
    pub visible_signals: &'static [&'static str],
    pub visible_uncertainty: &'static [&'static str],
    pub fog_requirements: &'static [&'static str],
    pub actions: &'static [SurfaceAction],
    pub expected_receipts: &'static [&'static str],
    pub archive_subjects: &'static [&'static str],
    pub admin_debug_exempt: bool,
}

/// A structurally incomplete or contradictory decision-surface declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSurfaceContractError {
    AdminExemptionMismatch,
    MissingDecisionQuestion,
    MissingVisibleSignals,
    MissingVisibleUncertainty,
    MissingFogRequirements,
    MissingActions,
    MissingExpectedReceipts,
    MissingArchiveSubjects,
}

impl fmt::Display for DecisionSurfaceContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AdminExemptionMismatch => {
                "admin_debug_exempt must be true exactly for AdminDebug surfaces"
            }
            Self::MissingDecisionQuestion => "gameplay surface has no decision question",
            Self::MissingVisibleSignals => "gameplay surface has no visible signals",
            Self::MissingVisibleUncertainty => "gameplay surface has no visible uncertainty",
            Self::MissingFogRequirements => "gameplay surface has no fog requirements",
            Self::MissingActions => "gameplay surface declares no actions",
            Self::MissingExpectedReceipts => "gameplay surface has no expected receipts",
            Self::MissingArchiveSubjects => "gameplay surface has no Archive subjects",
        })
    }
}

impl std::error::Error for DecisionSurfaceContractError {}

fn has_only_declared_entries(entries: &[&str]) -> bool {
    !entries.is_empty() && entries.iter().all(|entry| !entry.trim().is_empty())
}

fn declares_actions(actions: &[SurfaceAction]) -> bool {
    !actions.is_empty()
        && actions
            .iter()
            .all(|action| !action.name().trim().is_empty())
}

impl DecisionSurfaceContract {
    /// Checks the structural requirements of the declared role.
    ///
    /// Administrative and Archive/reference surfaces may describe signals,
    /// but only gameplay surfaces must close every part of the player decision
    /// loop. The exemption flag is deliberately redundant with the role: that
    /// makes an exemption explicit in every manifest row and lets validation
    /// reject contradictory declarations.
    ///
    /// # Errors
    ///
    /// Returns the first missing gameplay field or an exemption/role
    /// contradiction.
    pub fn validate(&self) -> Result<(), DecisionSurfaceContractError> {
        if self.admin_debug_exempt != (self.role == DecisionSurfaceRole::AdminDebug) {
            return Err(DecisionSurfaceContractError::AdminExemptionMismatch);
        }
        if !matches!(
            self.role,
            DecisionSurfaceRole::Gameplay | DecisionSurfaceRole::Observer
        ) {
            return Ok(());
        }
        if self
            .decision_question
            .is_none_or(|question| question.trim().is_empty())
        {
            return Err(DecisionSurfaceContractError::MissingDecisionQuestion);
        }
        if !has_only_declared_entries(self.visible_signals) {
            return Err(DecisionSurfaceContractError::MissingVisibleSignals);
        }
        if !has_only_declared_entries(self.visible_uncertainty) {
            return Err(DecisionSurfaceContractError::MissingVisibleUncertainty);
        }
        if !has_only_declared_entries(self.fog_requirements) {
            return Err(DecisionSurfaceContractError::MissingFogRequirements);
        }
        if !declares_actions(self.actions) {
            return Err(DecisionSurfaceContractError::MissingActions);
        }
        if !has_only_declared_entries(self.expected_receipts) {
            return Err(DecisionSurfaceContractError::MissingExpectedReceipts);
        }
        if !has_only_declared_entries(self.archive_subjects) {
            return Err(DecisionSurfaceContractError::MissingArchiveSubjects);
        }
        Ok(())
    }

    /// Returns true only for a valid, non-exempt gameplay declaration with at
    /// least one action the player can actually take today. A surface whose
    /// every action is declared-but-sealed renders a decision question it
    /// cannot answer; it stays outside every gameplay gate (ADR249 R9) — the
    /// dossier card is exactly that surface until Gate 5 enables Investigate.
    #[must_use]
    pub fn satisfies_gameplay_gate(&self) -> bool {
        self.role == DecisionSurfaceRole::Gameplay
            && !self.admin_debug_exempt
            && self.validate().is_ok()
            && self.actions.iter().any(SurfaceAction::is_available)
    }
}

/// ECS marker connecting a rendered entity to its manifest row.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeclaredSurface {
    pub id: SurfaceId,
}

impl DeclaredSurface {
    #[must_use]
    pub const fn new(id: SurfaceId) -> Self {
        Self { id }
    }
}

const NONE: &[&str] = &[];
const NO_ACTIONS: &[SurfaceAction] = &[];
const UNFOGGED_ADMIN: &[&str] = &["unfogged material truth; no player knowledge state"];

const fn admin_surface(
    id: SurfaceId,
    visible_signals: &'static [&'static str],
    visible_uncertainty: &'static [&'static str],
) -> DecisionSurfaceContract {
    DecisionSurfaceContract {
        id,
        role: DecisionSurfaceRole::AdminDebug,
        decision_question: None,
        visible_signals,
        visible_uncertainty,
        fog_requirements: UNFOGGED_ADMIN,
        actions: NO_ACTIONS,
        expected_receipts: NONE,
        archive_subjects: NONE,
        admin_debug_exempt: true,
    }
}

/// The Investigate action's honest unavailability reason. Spelled identically
/// in `ui::dossier_compose::INVESTIGATE_UNAVAILABLE_REASON` (the R6
/// placeholder seal); the contract test pins both spellings to each other so
/// the manifest and the rendered card cannot drift.
const INVESTIGATE_SEALED: &[SurfaceAction] = &[SurfaceAction::unavailable(
    "investigate",
    "Investigation opens this page — unavailable until Gate 5.",
)];

#[allow(clippy::too_many_arguments)]
const fn gameplay_surface(
    id: SurfaceId,
    decision_question: &'static str,
    visible_signals: &'static [&'static str],
    visible_uncertainty: &'static [&'static str],
    fog_requirements: &'static [&'static str],
    actions: &'static [SurfaceAction],
    expected_receipts: &'static [&'static str],
    archive_subjects: &'static [&'static str],
) -> DecisionSurfaceContract {
    DecisionSurfaceContract {
        id,
        role: DecisionSurfaceRole::Gameplay,
        decision_question: Some(decision_question),
        visible_signals,
        visible_uncertainty,
        fog_requirements,
        actions,
        expected_receipts,
        archive_subjects,
        admin_debug_exempt: false,
    }
}

/// Authoritative inventory of the surfaces composed by `main.rs`.
///
pub const SHIPPED_SURFACE_MANIFEST: &[DecisionSurfaceContract] = &[
    DecisionSurfaceContract {
        id: SurfaceId::ObserverProduction,
        role: DecisionSurfaceRole::Observer,
        decision_question: Some("Whose work does this industry depend on, and who relies on its output?"),
        visible_signals: &["exact inventories and input requirements", "planned and produced batch receipts", "labor and capacity budgets", "actual freight lots and quantity realization"],
        visible_uncertainty: &["county aggregate cohorts have no factory coordinates", "designed physical budgets are separate from observed employment", "physical dependence does not prove organization or readiness for collective action", "absent or ungranted circuit data is explicit"],
        fog_requirements: &["separate full-observer and grant-filtered reader capabilities", "same campaign, committed period, perspective and generation as the map"],
        actions: INVESTIGATE_SEALED,
        expected_receipts: &["production, dispatch, arrival and delivery receipts bound to the committed identity"],
        archive_subjects: &["county/<geoid>", "material cohort identity"],
        admin_debug_exempt: false,
    },
    DecisionSurfaceContract {
        id: SurfaceId::ObserverShell,
        role: DecisionSurfaceRole::Observer,
        decision_question: Some("What constrains this economy, where does it propagate, and what evidence explains the change?"),
        visible_signals: &["acknowledged economic observations", "campaign and selected period", "source units and vintage"],
        visible_uncertainty: &["missing is not zero", "Archive verification lag", "historical inspection"],
        fog_requirements: &["explicit full-observer capability or separately grant-filtered player preview", "no cached facts across perspective changes"],
        actions: INVESTIGATE_SEALED,
        expected_receipts: &["committed tick identity", "cited Archive verification"],
        archive_subjects: &["county/<geoid>", "place/<geoid>"],
        admin_debug_exempt: false,
    },
    gameplay_surface(
        SurfaceId::CountyDossier,
        "What is true here, and what would Investigation reveal?",
        &[
            "county identity and containment",
            "grant-visible signal atoms with provenance citations",
            "place-link chips with fog and pending states",
            "durable tick beside the page's verified tick",
            "chronicle strip from atom supersession",
        ],
        &[
            "earned-tier knowledge stays sealed pending until Gate 5",
            "Archive materialization lag: verified tick behind the durable tail",
            "uninvestigated place subjects render R6 placeholder pages",
        ],
        &[
            "known-only Archive reads through the fog-safe reader role",
            "unacknowledged place subjects render fog chips with zero label bytes",
            "reader credentials carry no writer authority",
        ],
        INVESTIGATE_SEALED,
        &[
            "acknowledged-commit tick status",
            "county card atoms through the fog-safe view",
            "subject atom history for the chronicle strip",
        ],
        &["county/<geoid>", "place/<geoid>"],
    ),
    admin_surface(SurfaceId::TitleLockup, &["application identity"], NONE),
];

/// Resolves one surface id to its sole manifest declaration.
///
/// # Panics
///
/// Panics when `id` is absent. The manifest exhaustiveness test proves that
/// every closed [`SurfaceId`] variant resolves in the shipped build.
#[must_use]
pub fn contract_for(id: SurfaceId) -> &'static DecisionSurfaceContract {
    SHIPPED_SURFACE_MANIFEST
        .iter()
        .find(|contract| contract.id == id)
        .unwrap_or_else(|| panic!("surface {id} has no manifest contract"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIGNAL: &[&str] = &["declared"];

    fn gameplay_contract(actions: &'static [SurfaceAction]) -> DecisionSurfaceContract {
        DecisionSurfaceContract {
            id: SurfaceId::ObserverShell,
            role: DecisionSurfaceRole::Gameplay,
            decision_question: Some("What is true here?"),
            visible_signals: SIGNAL,
            visible_uncertainty: SIGNAL,
            fog_requirements: SIGNAL,
            actions,
            expected_receipts: SIGNAL,
            archive_subjects: SIGNAL,
            admin_debug_exempt: false,
        }
    }

    #[test]
    fn availability_constructors_pin_their_fields() {
        let open = SurfaceAction::available("investigate");
        assert_eq!(open.name(), "investigate");
        assert!(open.is_available());
        assert_eq!(open.availability(), ActionAvailability::Available);

        let sealed = SurfaceAction::unavailable("investigate", "sealed until Gate 5");
        assert_eq!(sealed.name(), "investigate");
        assert!(!sealed.is_available());
        assert_eq!(
            sealed.availability(),
            ActionAvailability::Unavailable("sealed until Gate 5")
        );
    }

    /// ADR249 R9, pinned as executable policy: a structurally complete
    /// gameplay declaration whose EVERY action is declared-but-sealed is
    /// valid but cannot satisfy the gameplay gate — the contract test
    /// `current_client_cannot_claim_a_gameplay_gate` depends on exactly this
    /// rule staying false for the shipped dossier row.
    #[test]
    fn all_actions_unavailable_is_valid_but_never_satisfies_the_gate() {
        const SEALED: &[SurfaceAction] = &[SurfaceAction::unavailable(
            "investigate",
            "sealed until Gate 5",
        )];
        let contract = gameplay_contract(SEALED);
        assert!(contract.validate().is_ok());
        assert!(!contract.satisfies_gameplay_gate());
    }

    #[test]
    fn one_available_action_flips_the_gate_true() {
        const MIXED: &[SurfaceAction] = &[
            SurfaceAction::unavailable("investigate", "sealed until Gate 5"),
            SurfaceAction::available("survey"),
        ];
        assert!(gameplay_contract(MIXED).satisfies_gameplay_gate());
    }

    #[test]
    fn blank_action_names_fail_structural_validation() {
        const BLANK: &[SurfaceAction] = &[SurfaceAction::available("  ")];
        let contract = gameplay_contract(BLANK);
        assert_eq!(
            contract.validate(),
            Err(DecisionSurfaceContractError::MissingActions)
        );
        assert!(!contract.satisfies_gameplay_gate());
    }
}
