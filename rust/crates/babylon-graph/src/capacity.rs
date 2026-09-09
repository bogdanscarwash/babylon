//! `K` — an organization's CAPACITY, and its allocation (Lane A brick 3;
//! heat dossier §5.1 B and §5.3's argmax).
//!
//! **Capacity belongs to organizations** (Director ruling 2026-07-30,
//! ADR184). "The state" is not a node type: [`crate::substrate`]'s
//! vocabulary has `organization` (the thing that acts), `institution` (what
//! houses it) and `sovereign` (what claims territory). A budget with no
//! owner would make the state a force of nature rather than a set of
//! organizations with means — and the earlier draft of this module said "the
//! state" eleven times while [`Candidate`] carried a target and no actor,
//! which is that error written into a type.
//!
//! **The consequence is that repression and revolutionary action are the
//! same allocation.** A union local and a political police force both rank
//! what they could do by yield per unit spent and fund down the list until
//! the money runs out. Neither gets a privileged mechanic; the historical
//! asymmetry is the *size* of the budget and nothing else. The class
//! difference lives entirely in **replenishment** — tax and tribute on one
//! side, dues and expropriation on the other — which is why the frozen
//! Python engine grew `StateFinance` and `RevolutionaryFinance` as two names
//! for one shape (ADR184; do not transcribe the duplicate).
//!
//! **This module is where escalation stops being a threshold.** The old
//! mechanic asked "has heat crossed 0.7?" and answered with a constant,
//! which is question-begging twice over: it needs a number nobody can
//! derive, and it keys repression to accumulated conduct. Here an actor has
//! a finite budget per instrument, ranks what it could do by yield per unit
//! spent, and takes candidates in that order until the budget runs out.
//! Escalation is what *happens* when a target rises in that ranking. **There
//! is no threshold constant in this file and none may be added** — a
//! reviewer finding one has found a bug.
//!
//! Two consequences fall out rather than being coded:
//!
//! - **Repression against a distributed organization declines itself.** A
//!   redundant structure divides its own yield ([`crate::exposure`]'s
//!   replaceability quotient), so it loses the ranking to a target worth
//!   more per unit spent. Nothing anywhere says "if the org is distributed,
//!   skip it."
//! - **Cheap modes beat expensive ones at equal damage.** Disinformation is
//!   the cheapest mode by state capacity and among the most expensive by
//!   movement cost — which is exactly why it dominates the historical
//!   record. That is the ranking's arithmetic, not a designer's thumb.
//!
//! **Yield is computed on `L`, never on truth.** A [`Candidate`]'s yield
//! comes from [`crate::exposure`] evaluated over a [`crate::dossier::Dossier`]
//! scope, so an actor allocates against what it BELIEVES it would gain. It
//! can spend its budget badly. That is the point, and it holds for both
//! sides: a movement misreads the state's apparatus exactly as the state
//! misreads a movement.
//!
//! **What a mode costs, and what modes exist, are content** (BSL rules), not
//! constants here. This module ranks and spends; it does not know what
//! `"INFILTRATION"` means.

use crate::substrate::{GraphError, NodeId};
use babylon_kernel::Currency;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// A zero balance. `Currency` has no `Default`, and an untracked instrument
/// reading zero is the honest answer rather than an absence.
fn zero() -> Currency {
    Currency::from_micro_units(0)
}

/// `i128 -> f64` for a ratio denominator, loud past exact representability.
///
/// Mirrors [`crate::backfire`]'s widening discipline. Micro-unit budgets sit
/// far below `2^53` at any plausible scale, but a silent precision loss in
/// the ranking denominator would be a determinism bug that only appears at
/// large budgets — exactly the kind that survives testing.
fn micro_units_as_f64(amount: Currency) -> Result<f64, GraphError> {
    /// `2^53` itself IS exactly representable; `2^53 + 1` is the first
    /// integer that is not. The bound is therefore inclusive.
    const EXACT: u128 = 1_u128 << 53;
    let micro = amount.micro_units();
    // `unsigned_abs`, not `abs`: `i128::MIN.abs()` panics in debug and wraps
    // in release. Callers validate positivity first, but a widening helper
    // must not carry a panic path on the strength of its callers' manners.
    if micro.unsigned_abs() > EXACT {
        return Err(GraphError {
            message: format!(
                "cost {micro} micro-units exceeds the exactly-representable \
                 f64 range — refusing to rank on a lossy denominator"
            ),
        });
    }
    #[allow(clippy::cast_precision_loss)]
    Ok(micro as f64)
}

/// One action an organization could take this tick, already priced.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// The organization proposing it — whose budget pays, and whose
    /// [`crate::dossier::Dossier`] the yield was computed over.
    ///
    /// A candidate without an actor would be an action nobody is taking;
    /// [`Capacity::allocate`] rejects one whose actor is not the budget's
    /// owner rather than quietly spending someone else's means.
    pub actor: NodeId,
    /// Which budget this draws on — modes sharing an instrument compete for
    /// the same pool (§5.3: "modes compete for the same `K`").
    pub instrument: String,
    /// The mode being considered. Opaque here; its meaning is content.
    pub mode: String,
    /// Who it would be aimed at.
    pub target: NodeId,
    /// Expected damage, as the actor BELIEVES it — normally
    /// [`crate::exposure::decapitation_value`] over a dossier scope.
    pub expected_yield: f64,
    /// Money consumed if taken. Never zero or negative (see
    /// [`Capacity::allocate`]).
    ///
    /// `Currency`, not an abstract unit (ADR184 R8, Director ruling
    /// 2026-07-30). An abstract unit would need a declared currency-per-unit
    /// rate to accept imperial rent, and that rate is precisely the
    /// underivable coefficient ADR172 r5 forbids — the Φ → replenishment
    /// path is legitimate *because* it needs no conversion at all.
    pub cost: Currency,
}

/// One candidate the actor actually funded.
#[derive(Debug, Clone, PartialEq)]
pub struct Allocation {
    /// The funded candidate.
    pub candidate: Candidate,
    /// Its yield per unit spent — the quantity that won it the slot.
    pub ratio: f64,
}

/// One candidate the actor wanted and could not afford.
///
/// Unaffordability is the ONLY decline path: a malformed candidate (zero or
/// negative cost, non-finite yield, foreign actor) is a hard error, not a
/// decline. If a second path ever appears this becomes an enum; it is a
/// struct today because there is exactly one reason.
#[derive(Debug, Clone, PartialEq)]
pub struct Declined {
    /// What the actor wanted.
    pub candidate: Candidate,
    /// What its instrument had left when the candidate was considered.
    ///
    /// Always less than `candidate.cost`. Carried because "wanted the raid,
    /// had enough for the wiretap" is the narratable fact — the shortfall,
    /// not merely the refusal.
    pub remaining: Currency,
}

/// What one allocation pass did — funded AND declined.
///
/// Declines are derivable (`candidates − funded`), so this type buys
/// explicitness rather than information (ADR184 R9, Director ruling
/// 2026-07-30). It is worth the struct because **constraint is the
/// pedagogy**: an actor that wanted the raid and could only afford the
/// wiretap is the mechanic showing its work, and a caller that has to
/// re-derive that will mostly not bother.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AllocationOutcome {
    /// Funded, in the order funded (descending ratio).
    pub funded: Vec<Allocation>,
    /// Wanted and unaffordable, in the same ranking order.
    pub declined: Vec<Declined>,
}

/// `K` — one organization's finite capacity to act, per instrument.
///
/// Owned by the organization node whose means it is. Set by fiscal and
/// institutional capacity, spent by allocation, replenished between ticks —
/// and **replenishment is where the class difference lives**, not here.
/// This type cannot tell a police budget from a strike fund.
///
/// No `Default`: a capacity with no owner is the unowned budget ADR184
/// rules out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capacity {
    owner: NodeId,
    budgets: BTreeMap<String, Currency>,
}

impl Capacity {
    /// An organization with no capacity at all — it can see, but it cannot
    /// act.
    #[must_use]
    pub fn new(owner: NodeId) -> Self {
        Self {
            owner,
            budgets: BTreeMap::new(),
        }
    }

    /// The organization whose means these are.
    #[must_use]
    pub fn owner(&self) -> NodeId {
        self.owner
    }

    /// Add money to one instrument's budget.
    ///
    /// **This is the seam where the class difference lives** (ADR184 R4).
    /// Where the money came from — tax, tribute, dues, expropriation — is
    /// the caller's business and invisible here. `tribute_income` is imperial
    /// rent, so Φ reaches a repressive budget through this method and no
    /// conversion (R5).
    ///
    /// # Errors
    /// Returns [`GraphError`] on i128 overflow — loud, never saturating.
    pub fn replenish(&mut self, instrument: &str, amount: Currency) -> Result<(), GraphError> {
        let slot = self
            .budgets
            .entry(instrument.to_owned())
            .or_insert_with(zero);
        *slot = slot.checked_add(amount).map_err(|overflow| GraphError {
            message: format!("replenishing {instrument} overflowed: {overflow:?}"),
        })?;
        Ok(())
    }

    /// Money currently available to one instrument. An instrument never
    /// funded reads zero — the honest answer, since an organization that has
    /// not built an apparatus does not have one.
    #[must_use]
    pub fn available(&self, instrument: &str) -> Currency {
        self.budgets.get(instrument).copied().unwrap_or_else(zero)
    }

    /// Total unspent capacity across every instrument.
    ///
    /// # Errors
    /// Returns [`GraphError`] on i128 overflow.
    pub fn total_available(&self) -> Result<Currency, GraphError> {
        let mut total = zero();
        for amount in self.budgets.values() {
            total = total.checked_add(*amount).map_err(|overflow| GraphError {
                message: format!("summing capacity overflowed: {overflow:?}"),
            })?;
        }
        Ok(total)
    }

    /// Rank `candidates` by yield per unit spent and fund them in that order
    /// until each instrument's budget is exhausted. **This is the escalation
    /// mechanic in full.**
    ///
    /// Spends from `self`, returning both halves of what happened
    /// ([`AllocationOutcome`], ADR184 R9): what was funded, in the order
    /// funded, and what was wanted and unaffordable, with the shortfall.
    ///
    /// A candidate whose cost exceeds its instrument's *remaining* budget is
    /// declined, and cheaper candidates behind it may still be funded. That is
    /// deliberate: an actor that cannot afford the raid still runs the
    /// surveillance.
    ///
    /// **Determinism.** Ranking is a total order — ratio descending, then
    /// `(instrument, mode, target, cost, expected_yield)` ascending — so a
    /// permuted candidate list produces a byte-identical allocation. Float
    /// ratios alone are only a partial order, and leaving ties to input order
    /// would put the tick hash at the mercy of iteration order upstream. The
    /// chain runs to the last field that can differ: two candidates equal on
    /// all of them are indistinguishable, so their order cannot be observed.
    ///
    /// # Errors
    /// Returns [`GraphError`] if any candidate has non-positive cost (zero
    /// makes the ratio infinite — a free action that always outranks
    /// everything; negative would be an action that PAYS the actor to take
    /// it, which is a content bug either way, not a strategy), a non-finite
    /// yield, an actor other than this budget's owner, or a cost past f64's
    /// exactly-representable range.
    pub fn allocate(&mut self, candidates: &[Candidate]) -> Result<AllocationOutcome, GraphError> {
        let mut ranked = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            if candidate.actor != self.owner {
                return Err(GraphError {
                    message: format!(
                        "candidate {}/{} is proposed by {:?} but this capacity \
                         belongs to {:?} — an organization spends its own means",
                        candidate.instrument, candidate.mode, candidate.actor, self.owner
                    ),
                });
            }
            // Zero and negative are both content bugs, but they fail
            // DIFFERENTLY and the message must say which: a free action's
            // ratio is infinite and outranks every priced one, while a
            // negative cost yields a negative ratio and sinks to the BOTTOM
            // of the ranking — silently deprioritized rather than loudly
            // dominant. Reporting them as one condition would send a reader
            // hunting for the wrong symptom.
            if candidate.cost.micro_units() == 0 {
                return Err(GraphError {
                    message: format!(
                        "candidate {}/{} against {:?} costs nothing — a free action's \
                         ratio is unbounded and outranks every priced one; \
                         price it in content",
                        candidate.instrument, candidate.mode, candidate.target
                    ),
                });
            }
            if candidate.cost.micro_units() < 0 {
                return Err(GraphError {
                    message: format!(
                        "candidate {}/{} against {:?} costs {} micro-units — a negative \
                         price pays its taker to act, and would rank LAST instead of \
                         being refused; price it in content",
                        candidate.instrument,
                        candidate.mode,
                        candidate.target,
                        candidate.cost.micro_units()
                    ),
                });
            }
            if !candidate.expected_yield.is_finite() {
                return Err(GraphError {
                    message: format!(
                        "candidate {}/{} against {:?} has a non-finite expected yield — \
                         refusing to rank it",
                        candidate.instrument, candidate.mode, candidate.target
                    ),
                });
            }
            // `-0.0` and `+0.0` are one value that compares equal and one
            // value that does not serialize equal. Left alone, a sign bit
            // arriving from upstream arithmetic (`-1.0 * 0.0`) would survive
            // the `total_cmp` tiebreak below, decide which of two otherwise
            // identical candidates is funded, and reach the tick hash.
            // Canonicalizing the VALUE — not merely the comparison — is what
            // closes it: normalizing only inside the comparator would make
            // the two candidates tie on every field and fall back to input
            // order, which is the defect the tiebreak exists to remove.
            // Done at the single funnel every yield crosses.
            let mut candidate = candidate.clone();
            if candidate.expected_yield == 0.0 {
                candidate.expected_yield = 0.0;
            }
            let ratio = candidate.expected_yield / micro_units_as_f64(candidate.cost)?;
            ranked.push(Allocation { candidate, ratio });
        }

        ranked.sort_by(|a, b| {
            b.ratio
                .partial_cmp(&a.ratio)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.candidate.instrument.cmp(&b.candidate.instrument))
                .then_with(|| a.candidate.mode.cmp(&b.candidate.mode))
                .then_with(|| a.candidate.target.cmp(&b.candidate.target))
                // The same action against the same target at two prices ties
                // on every field above. Cheaper first, then `total_cmp` as
                // the closing tiebreak — a genuine total order on f64,
                // correct for negative yields where a bit-pattern compare
                // would invert. Without these two the sort is stable-but-
                // input-ordered and the tick hash inherits upstream
                // iteration order.
                .then_with(|| a.candidate.cost.cmp(&b.candidate.cost))
                .then_with(|| {
                    a.candidate
                        .expected_yield
                        .total_cmp(&b.candidate.expected_yield)
                })
        });

        let mut outcome = AllocationOutcome::default();
        for allocation in ranked {
            let remaining = self.available(&allocation.candidate.instrument);
            if allocation.candidate.cost <= remaining {
                let left =
                    remaining
                        .checked_sub(allocation.candidate.cost)
                        .map_err(|overflow| GraphError {
                            message: format!("spending underflowed: {overflow:?}"),
                        })?;
                self.budgets
                    .insert(allocation.candidate.instrument.clone(), left);
                outcome.funded.push(allocation);
            } else {
                outcome.declined.push(Declined {
                    candidate: allocation.candidate,
                    remaining,
                });
            }
        }
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::{Candidate, Capacity};
    use crate::substrate::NodeId;
    use babylon_kernel::Currency;

    /// The organization whose budget every test below spends.
    const ACTOR: NodeId = NodeId(7);

    /// Micro-unit money, so test literals stay readable.
    fn money(units: i128) -> Currency {
        Currency::from_micro_units(units)
    }

    fn candidate(mode: &str, target: u64, expected_yield: f64, cost: i128) -> Candidate {
        Candidate {
            actor: ACTOR,
            instrument: "political-police".to_owned(),
            mode: mode.to_owned(),
            target: NodeId(target),
            expected_yield,
            cost: money(cost),
        }
    }

    #[test]
    fn a_negative_zero_yield_cannot_decide_who_is_funded() {
        // -0.0 and +0.0 are one value that compares equal and one value that
        // does not serialize equal. A sign bit from upstream arithmetic must
        // not survive into the ranking and out to the tick hash.
        let mut negative = candidate("RAID", 1, -0.0, 4);
        let positive = candidate("RAID", 1, 0.0, 4);
        assert!(
            negative.expected_yield.is_sign_negative(),
            "the fixture must actually carry the sign bit"
        );

        let mut results = Vec::new();
        for order in [
            vec![negative.clone(), positive.clone()],
            vec![positive.clone(), negative.clone()],
        ] {
            let mut capacity = Capacity::new(ACTOR);
            capacity.replenish("political-police", money(4)).unwrap(); // room for exactly one
            let outcome = capacity.allocate(&order).unwrap();
            let funded = &outcome.funded;
            assert_eq!(funded.len(), 1);
            assert!(
                !funded[0].candidate.expected_yield.is_sign_negative(),
                "the stored yield is canonicalized, not merely compared"
            );
            results.push(funded[0].candidate.expected_yield.to_bits());
        }
        assert_eq!(
            results[0], results[1],
            "the funded candidate is bit-identical either way"
        );

        // And the guard is real: without canonicalization these two differ.
        negative.expected_yield = -0.0;
        assert_ne!(
            negative.expected_yield.to_bits(),
            positive.expected_yield.to_bits(),
            "-0.0 and +0.0 really do have different bit patterns"
        );
    }

    #[test]
    fn an_organization_spends_only_its_own_means() {
        // The ADR184 invariant. Silently funding another organization's
        // candidate would make the budget a shared pool — which is the
        // unowned "state capacity" this module was rebuilt to remove.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(100)).unwrap();
        let mut someone_else = candidate("RAID", 1, 0.9, 5);
        someone_else.actor = NodeId(999);
        let err = capacity.allocate(&[someone_else]).unwrap_err();
        assert!(
            err.message.contains("spends its own means"),
            "{}",
            err.message
        );
    }

    #[test]
    fn the_same_allocation_serves_a_movement_and_a_police_force() {
        // ADR184's substantive claim: nothing in this type distinguishes
        // repression from revolutionary action. Identical ranking, identical
        // budget, identical outcome — the only difference between the two
        // actors is which node owns the capacity and where it was
        // replenished from, neither of which the allocator can see.
        let police = NodeId(7);
        let local = NodeId(8);
        let priced = |actor: NodeId, mode: &str, target: u64, y: f64, cost: i128| Candidate {
            actor,
            instrument: "cadre".to_owned(),
            mode: mode.to_owned(),
            target: NodeId(target),
            expected_yield: y,
            cost: money(cost),
        };

        let mut state_side = Capacity::new(police);
        state_side.replenish("cadre", money(4)).unwrap();
        let state_funded = state_side
            .allocate(&[
                priced(police, "INFILTRATE", 1, 0.9, 3),
                priced(police, "RAID", 2, 0.8, 8),
            ])
            .unwrap()
            .funded;

        let mut movement_side = Capacity::new(local);
        movement_side.replenish("cadre", money(4)).unwrap();
        let movement_funded = movement_side
            .allocate(&[
                priced(local, "INFILTRATE", 1, 0.9, 3),
                priced(local, "RAID", 2, 0.8, 8),
            ])
            .unwrap()
            .funded;

        assert_eq!(
            state_funded
                .iter()
                .map(|a| (a.candidate.mode.as_str(), a.candidate.cost))
                .collect::<Vec<_>>(),
            movement_funded
                .iter()
                .map(|a| (a.candidate.mode.as_str(), a.candidate.cost))
                .collect::<Vec<_>>(),
            "the allocator cannot tell a police budget from a strike fund"
        );
        assert_eq!(state_side.owner(), police);
        assert_eq!(movement_side.owner(), local);
    }

    #[test]
    fn an_organization_with_no_capacity_can_see_but_not_act() {
        let mut capacity = Capacity::new(ACTOR);
        let funded = capacity
            .allocate(&[candidate("RAID", 1, 0.9, 5)])
            .unwrap()
            .funded;
        assert!(funded.is_empty(), "no budget, no action — and no error");
    }

    #[test]
    fn escalation_is_what_a_bigger_budget_buys_not_a_threshold_crossing() {
        // THE contract this module exists for. The same three candidates,
        // ranked identically, funded to different depths purely by budget.
        // Nothing in the engine asks "has heat crossed X".
        let candidates = [
            candidate("SURVEIL", 1, 0.2, 1),    // ratio 0.20
            candidate("INFILTRATE", 2, 0.9, 3), // ratio 0.30
            candidate("RAID", 3, 0.8, 8),       // ratio 0.10
        ];

        let mut lean = Capacity::new(ACTOR);
        lean.replenish("political-police", money(3)).unwrap();
        let lean_funded = lean.allocate(&candidates).unwrap().funded;
        assert_eq!(
            lean_funded
                .iter()
                .map(|a| a.candidate.mode.as_str())
                .collect::<Vec<_>>(),
            vec!["INFILTRATE"],
            "a lean state buys the best ratio it can afford"
        );

        let mut flush = Capacity::new(ACTOR);
        flush.replenish("political-police", money(12)).unwrap();
        let flush_funded = flush.allocate(&candidates).unwrap().funded;
        assert_eq!(
            flush_funded
                .iter()
                .map(|a| a.candidate.mode.as_str())
                .collect::<Vec<_>>(),
            vec!["INFILTRATE", "SURVEIL", "RAID"],
            "a flush state escalates by reaching further down the same ranking"
        );
    }

    #[test]
    fn a_redundant_target_declines_the_strike_by_itself() {
        // exposure's replaceability quotient divides a distributed org's
        // yield down; the ranking then declines it. There is no
        // "if distributed, skip" branch anywhere.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(4)).unwrap();
        let funded = capacity
            .allocate(&[
                candidate("RAID", 1, 0.08, 4), // the distributed org
                candidate("RAID", 2, 0.60, 4), // the centralized one
            ])
            .unwrap()
            .funded;
        assert_eq!(funded.len(), 1);
        assert_eq!(
            funded[0].candidate.target,
            NodeId(2),
            "capacity went to the target worth more per unit spent"
        );
    }

    #[test]
    fn a_cheap_mode_outranks_an_expensive_one_at_higher_damage() {
        // Why disinformation dominates the historical record: cheapest by
        // state capacity, dear by movement cost. Arithmetic, not a thumb.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(2)).unwrap();
        let funded = capacity
            .allocate(&[
                candidate("LIQUIDATE", 1, 0.95, 20), // ratio 0.0475
                candidate("BAD_JACKET", 2, 0.40, 1), // ratio 0.40
            ])
            .unwrap()
            .funded;
        assert_eq!(funded[0].candidate.mode, "BAD_JACKET");
    }

    #[test]
    fn an_unaffordable_candidate_does_not_block_a_cheaper_one_behind_it() {
        // A state that cannot afford the raid still runs the surveillance.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(2)).unwrap();
        let funded = capacity
            .allocate(&[
                candidate("RAID", 1, 9.0, 10), // best ratio, unaffordable
                candidate("SURVEIL", 2, 0.5, 2),
            ])
            .unwrap()
            .funded;
        assert_eq!(funded.len(), 1);
        assert_eq!(funded[0].candidate.mode, "SURVEIL");
    }

    #[test]
    fn allocation_is_invariant_under_candidate_permutation() {
        // The tick-hash contract. Float ratios alone are a PARTIAL order;
        // ties broken by input order would leave the hash at the mercy of
        // upstream iteration.
        let a = candidate("RAID", 1, 0.5, 5); // ratio 0.1
        let b = candidate("RAID", 2, 0.4, 4); // ratio 0.1 — an exact tie
        let c = candidate("SURVEIL", 3, 0.2, 2); // ratio 0.1 — another
        let orders = [
            vec![a.clone(), b.clone(), c.clone()],
            vec![c.clone(), b.clone(), a.clone()],
            vec![b.clone(), a.clone(), c.clone()],
        ];
        let mut results = Vec::new();
        for order in &orders {
            let mut capacity = Capacity::new(ACTOR);
            capacity.replenish("political-police", money(100)).unwrap();
            let funded = capacity.allocate(order).unwrap().funded;
            results.push(
                funded
                    .iter()
                    .map(|x| (x.candidate.mode.clone(), x.candidate.target))
                    .collect::<Vec<_>>(),
            );
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[1], results[2]);
    }

    #[test]
    fn candidates_differing_only_in_price_still_rank_deterministically() {
        // The guard above cannot reach this case: its candidates differ in
        // mode or target, which resolves every tie before the comparator
        // runs out of fields. Two candidates sharing (instrument, mode,
        // target) at an equal ratio tie on ALL of them, and a stable sort
        // then falls back to input order — putting the tick hash at the
        // mercy of whatever produced the list. The same action against the
        // same target at two prices is a content shape nothing forbids.
        let dear = candidate("RAID", 1, 0.5, 5); // ratio 0.1
        let cheap = candidate("RAID", 1, 0.4, 4); // ratio 0.1, same triple
        let orders = [
            vec![dear.clone(), cheap.clone()],
            vec![cheap.clone(), dear.clone()],
        ];
        let mut results = Vec::new();
        for order in &orders {
            let mut capacity = Capacity::new(ACTOR);
            capacity.replenish("political-police", money(7)).unwrap(); // room for exactly one
            let funded = capacity.allocate(order).unwrap().funded;
            results.push(funded.iter().map(|a| a.candidate.cost).collect::<Vec<_>>());
        }
        assert_eq!(
            results[0], results[1],
            "input order must not decide which of two equally-rated \
             candidates the state can afford"
        );
    }

    #[test]
    fn spending_draws_down_the_budget_and_never_overspends() {
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(10)).unwrap();
        let funded = capacity
            .allocate(&[
                candidate("A", 1, 1.0, 6),
                candidate("B", 2, 1.0, 6),
                candidate("C", 3, 1.0, 4),
            ])
            .unwrap()
            .funded;
        assert_eq!(funded.len(), 2, "6 + 4 fits in 10; the second 6 does not");
        assert_eq!(capacity.available("political-police"), money(0));
        assert_eq!(capacity.total_available().unwrap(), money(0));
    }

    #[test]
    fn instruments_hold_separate_budgets() {
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(5)).unwrap();
        capacity.replenish("courts", money(5)).unwrap();
        let funded = capacity
            .allocate(&[
                Candidate {
                    actor: ACTOR,
                    instrument: "courts".to_owned(),
                    mode: "PROSECUTE".to_owned(),
                    target: NodeId(1),
                    expected_yield: 0.5,
                    cost: money(5),
                },
                candidate("RAID", 2, 0.5, 5),
            ])
            .unwrap()
            .funded;
        assert_eq!(funded.len(), 2, "each drew on its own pool");
        assert_eq!(capacity.available("courts"), money(0));
        assert_eq!(capacity.available("political-police"), money(0));
    }

    #[test]
    fn a_free_action_is_loud_never_infinitely_attractive() {
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(10)).unwrap();
        let err = capacity
            .allocate(&[candidate("FREE", 1, 0.5, 0)])
            .unwrap_err();
        assert!(err.message.contains("costs nothing"), "{}", err.message);
    }

    #[test]
    fn an_action_that_pays_its_taker_is_loud_too() {
        // Currency is SIGNED (kernel OQ-28), so a negative cost is
        // expressible where the old u64 made it unrepresentable. It would
        // give a negative ratio and sort last — quietly, when it is really a
        // content bug: an action nobody pays for.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(10)).unwrap();
        let err = capacity
            .allocate(&[candidate("PAID_TO_ACT", 1, 0.5, -5)])
            .unwrap_err();
        assert!(
            err.message.contains("pays its taker"),
            "a negative price fails for its OWN reason, not the free-action one: {}",
            err.message
        );
    }

    #[test]
    fn the_exactly_representable_boundary_is_inclusive() {
        // 2^53 IS exactly representable in f64; 2^53 + 1 is the first
        // integer that is not. An exclusive bound here would refuse to rank
        // a perfectly exact cost — a spurious loud failure, which is its own
        // kind of dishonesty.
        const EXACT: i128 = 1_i128 << 53;
        let mut capacity = Capacity::new(ACTOR);
        capacity
            .replenish("political-police", money(EXACT))
            .unwrap();

        let at_bound = capacity.allocate(&[candidate("RAID", 1, 0.5, EXACT)]);
        assert!(
            at_bound.is_ok(),
            "2^53 is exact and must rank: {:?}",
            at_bound.err()
        );
        assert_eq!(at_bound.unwrap().funded.len(), 1, "and it is affordable");

        let mut over = Capacity::new(ACTOR);
        over.replenish("political-police", money(EXACT)).unwrap();
        let err = over
            .allocate(&[candidate("RAID", 1, 0.5, EXACT + 1)])
            .unwrap_err();
        assert!(err.message.contains("lossy denominator"), "{}", err.message);
    }

    #[test]
    fn a_decline_carries_the_shortfall_not_merely_the_refusal() {
        // ADR184 R9. "Wanted the raid, had enough for the wiretap" is the
        // narratable fact, and it is the mechanic showing its work.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(5)).unwrap();
        let outcome = capacity
            .allocate(&[
                candidate("SURVEIL", 1, 0.20, 4), // ratio 0.05, affordable
                candidate("RAID", 2, 0.90, 12),   // ratio 0.075, ranks FIRST
            ])
            .unwrap();

        assert_eq!(outcome.funded.len(), 1);
        assert_eq!(outcome.funded[0].candidate.mode, "SURVEIL");
        assert_eq!(outcome.declined.len(), 1, "the raid is visible, not lost");
        assert_eq!(outcome.declined[0].candidate.mode, "RAID");
        assert_eq!(
            outcome.declined[0].remaining,
            money(5),
            "the shortfall is against what was left WHEN IT WAS CONSIDERED — \
             the raid outranked the surveillance, so it saw the full budget"
        );
        // And the decline did not consume anything.
        assert_eq!(capacity.available("political-police"), money(1));
    }

    #[test]
    fn nothing_declined_is_an_empty_list_never_a_missing_one() {
        // III.11: "no declines" must be observably different from "declines
        // not computed". An empty Vec says the pass ran and refused nobody.
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(100)).unwrap();
        let outcome = capacity
            .allocate(&[candidate("SURVEIL", 1, 0.2, 1)])
            .unwrap();
        assert_eq!(outcome.funded.len(), 1);
        assert!(outcome.declined.is_empty());
    }

    #[test]
    fn imperial_rent_funds_the_budget_with_no_conversion() {
        // ADR184 R5. tribute_income is Currency and a budget is Currency, so
        // Φ reaches a repressive instrument through replenish() and NOTHING
        // else — no rate, no coefficient, no conversion to derive. This test
        // exists to fail loudly if anyone reintroduces one.
        let tribute = money(750_000); // as it would arrive from CLIENT_STATE
        let mut political_police = Capacity::new(ACTOR);
        political_police
            .replenish("political-police", tribute)
            .unwrap();
        assert_eq!(
            political_police.available("political-police"),
            tribute,
            "imperial rent arrives at face value"
        );
    }

    #[test]
    fn a_non_finite_yield_is_refused_not_ranked() {
        let mut capacity = Capacity::new(ACTOR);
        capacity.replenish("political-police", money(10)).unwrap();
        for bad in [f64::NAN, f64::INFINITY] {
            let err = capacity
                .allocate(&[candidate("RAID", 1, bad, 2)])
                .unwrap_err();
            assert!(err.message.contains("non-finite"), "{}", err.message);
        }
    }
}
