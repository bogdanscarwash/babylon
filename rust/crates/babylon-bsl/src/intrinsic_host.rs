//! The named-intrinsic call boundary (§2.7: transcendentals "are **never**
//! language primitives — they exist only as named intrinsics with pinned
//! deterministic implementations"). Phase 1 defined the trait only.
//! `{exp, log}` now dispatch too (Task 2 of the #576 intrinsic-host train —
//! R10/ADR176 r21, pinned soft-float libm + golden vectors, via
//! `babylon_kernel::transcendental`). Amendment AJ keeps the ADR188 Row
//! 11/D69 kernel-seeded, keyed draw engine-private: finite-kernel
//! realization calls `draw_finite_kernel_ticket` directly, and no draw
//! name dispatches through [`IntrinsicHost`].
//! The ADR219 exact-arithmetic sextet (Director ruling 2026-08-22) —
//! `sqrt`, `round-half-even`, `min`, `max`, `abs`, `clamp` — dispatches
//! below: ADR188 Row 6's fallback rider taken, Row 3's ratified
//! housekeeping rider landed (D70 resolved), Rows 4/5's "no rider"
//! dispositions superseded on #591 item 2's accumulated evidence. Like
//! `floor`, each crosses via an IEEE-754 exactly-specified operation — no
//! pinned soft-float libm, no §4.3 golden-vector family (the per-name
//! disposition is recorded in §3.10's normative paragraphs).
//!
//! `floor` (ADR188 Row 2, §3.10 / Draft-Ruling Register D97) lands early
//! and separately from that gate: it is not a transcendental, needs no
//! pinned soft-float libm crate, and crosses via `f64::floor` — IEEE-754's
//! own `roundToIntegralTowardNegative`, exactly specified by the standard
//! itself (not by §4.3, whose closed basic-op list is `+ − × ÷` and
//! comparison only; its golden-vector clause is for transcendentals). ADR188's
//! "libm golden vectors at implementation" consequence therefore does not
//! apply to this rider — there is no libm crossing to pin a vector against
//! — and this is that consequence's explicit disposition, not a silent
//! omission. [`KernelIntrinsicHost`] is this crate's first non-empty,
//! non-test-only [`IntrinsicHost`] — wired into `babylon-tick::run_once_into`
//! (the production seam the CLI driver and `babylon-client`'s engine link
//! both call), not merely constructed in a test module.

use crate::evaluator::{EvalCode, EvalError, Value};
use babylon_graph::stable_element::{
    StableElementKeyV1, StableElementResolverV1, MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2,
};
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1, RngDomainV2};
use babylon_kernel::{KernelRng, SessionId};

/// The typed identity inputs for one engine-private deterministic draw.
pub enum DrawIdentityContext<'a> {
    /// Current V1 identity and string carriers.
    V1 {
        /// Current V1 session identity.
        session: &'a SessionId,
        /// Current V1 rule domain.
        domain: &'a str,
        /// Current V1 subject content id or fixture fallback.
        subject: &'a str,
        /// Current V1 node-name map used by active-element resolution.
        node_content_ids:
            Option<&'a std::collections::HashMap<babylon_graph::substrate::NodeId, String>>,
    },
    /// Seed-aware V2 identity with graph-owned stable provenance.
    V2 {
        /// Checked replay session identity.
        session: &'a ReplaySessionIdV1,
        /// Explicit replay seed.
        seed: ReplaySeed,
        /// Checked firing-rule qname.
        domain: RngDomainV2,
        /// Resolver that sealed every accepted graph identity.
        resolver: &'a StableElementResolverV1,
        /// Resolver-produced subject identity.
        subject: StableElementKeyV1,
    },
}

/// The non-operand half of a draw key. V1 keeps its existing session,
/// rule-domain, content-id, and fixture-fallback inputs byte-for-byte. V2
/// accepts only a checked replay session, explicit seed, checked rule qname,
/// and graph-resolver-produced stable subject identity. Neither layout lets
/// a rule supply session, tick, or domain as an operand.
pub struct DrawContext<'a> {
    /// Typed V1 or V2 identity inputs.
    pub identity: DrawIdentityContext<'a>,
    /// The host's construction-time tick — never an operand.
    pub tick: u64,
}

/// One active-element identity already resolved for the selected private draw layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawActiveElement {
    /// Current V1 content-id string or fixture fallback.
    V1(String),
    /// Resolver-produced V2 stable graph identity.
    V2(StableElementKeyV1),
}

/// The full context one `IntrinsicHost::call` sees: the optional
/// [`DrawContext`] (`None` for a context-free expression caller) plus the
/// chapter C8 element stack in outermost-first order. V1 entries retain
/// their current resolved string representation. V2 entries are stable
/// graph keys that the resolver validates again while composing the carrier.
///
/// Every author-declarable intrinsic ignores this entirely. The engine-owned
/// finite-kernel realization path consumes it directly rather than dispatching
/// an author-callable intrinsic.
pub struct IntrinsicCallCtx<'a> {
    /// `None` for a pure-expression caller (see this struct's own doc).
    pub draw_context: Option<&'a DrawContext<'a>>,
    /// The typed active-element chain, outermost-first.
    pub active_elements: Vec<DrawActiveElement>,
}

impl IntrinsicCallCtx<'_> {
    /// The context a pure-expression caller passes: no [`DrawContext`], no
    /// element chain. Named for the same "pure-expression caller" class
    /// this module's own doc and `EvalEnv::graph`'s doc already use —
    /// `:expr` binding resolution, the arithmetic conformance vectors,
    /// and every `EmptyIntrinsicHost` test path.
    #[must_use]
    pub fn context_free() -> Self {
        Self {
            draw_context: None,
            active_elements: Vec::new(),
        }
    }
}

/// Compose string segments into ONE string, injective by construction
/// (plan §3.3): each segment is emitted as `<decimal-len> ":" <segment>`,
/// segments joined by `"|"` — mirroring `babylon_kernel::rng::seed_for`'s
/// own length-prefix discipline, so two different segment chains can never
/// render to the same string (no ambiguity from where one segment ends and
/// the next begins).
#[must_use]
pub(crate) fn framed(segments: &[&str]) -> String {
    segments
        .iter()
        .map(|segment| format!("{}:{segment}", segment.len()))
        .collect::<Vec<_>>()
        .join("|")
}

/// Dispatches a named intrinsic call. The declared signature/cost checks
/// (`E-LOAD-020`/`E-LOAD-021`) are load-time gates; a host's failure here is
/// the evaluator's defense-in-depth, not the primary rejection point.
pub trait IntrinsicHost {
    /// Dispatch `name` over already-evaluated positional args, with the
    /// calling context retained by the shared evaluator boundary. Every
    /// author-declarable intrinsic implemented today is context-free and
    /// ignores `ctx`; the private finite-kernel draw consumes the same typed
    /// context outside intrinsic dispatch.
    ///
    /// # Errors
    ///
    /// [`EvalError`] when `name` is not provided by this host, or when the
    /// pinned implementation itself rejects the inputs.
    fn call(
        &self,
        name: &str,
        args: &[Value],
        _ctx: IntrinsicCallCtx<'_>,
    ) -> Result<Value, EvalError>;
}

/// A host with no registered intrinsics at all — every call fails loud.
/// Used by Phase-1 tests that exercise only arithmetic/comparison/boolean
/// forms, which never cross the intrinsic boundary.
pub struct EmptyIntrinsicHost;

impl IntrinsicHost for EmptyIntrinsicHost {
    fn call(
        &self,
        name: &str,
        _args: &[Value],
        _ctx: IntrinsicCallCtx<'_>,
    ) -> Result<Value, EvalError> {
        Err(EvalError::plain(format!(
            "no intrinsic registered: {name} (the kernel table is Phase 2)"
        )))
    }
}

/// The kernel's intrinsic table, as far as it is implemented today: `floor`
/// (ADR188 Row 2), `{exp, log}` (R10/ADR176 r21, ADR188 cap, Task 2 of the
/// #576 intrinsic-host train) — both cross via `babylon_kernel::
/// transcendental`, pinned soft-float `libm 0.2.16` — and the ADR219 sextet
/// (Director ruling 2026-08-22): `sqrt`, `round-half-even`, `min`, `max`,
/// `abs`, `clamp`, each crossing via an IEEE-754 exactly-specified
/// operation (correctly-rounded `sqrt`, `roundTiesToEven`, comparison/
/// copysign arithmetic), needing no pinned libm and owing no §4.3
/// golden-vector family — the `floor` rider's D97 disposition, recorded
/// per-name in §3.10's normative paragraphs.
pub struct KernelIntrinsicHost;

impl IntrinsicHost for KernelIntrinsicHost {
    fn call(
        &self,
        name: &str,
        args: &[Value],
        _ctx: IntrinsicCallCtx<'_>,
    ) -> Result<Value, EvalError> {
        match name {
            "floor" => eval_floor(args),
            "exp" => eval_exp(args),
            "log" => eval_log(args),
            "sqrt" => eval_sqrt(args),
            "round-half-even" => eval_round_half_even(args),
            "min" => eval_min(args),
            "max" => eval_max(args),
            "abs" => eval_abs(args),
            "clamp" => eval_clamp(args),
            other => Err(EvalError::plain(format!(
                "no intrinsic registered: {other} ('floor' — ADR188 Row 2 —, the \
                 {{exp, log}} transcendental pair — R10/ADR176 r21 —, and the six exact-arithmetic names \
                 'sqrt'/'round-half-even'/'min'/'max'/'abs'/'clamp' — ADR219 — \
                 are implemented today; any other name is outside §3.10's cap)"
            ))),
        }
    }
}

/// Consume the one private `u64` draw for a finite-kernel realization.
///
/// This function is crate-private by design under Amendment AJ. The draw key
/// binds the V2 replay session, signed seed, tick, firing rule, sample `QName`,
/// append-only slot, stable subject, and ordered active elements. V1 has no
/// signed replay seed and is refused rather than silently substituting one.
pub(crate) fn draw_finite_kernel_ticket(
    sample: &str,
    slot: u32,
    ctx: &IntrinsicCallCtx<'_>,
) -> Result<u64, EvalError> {
    let Some(draw_context) = ctx.draw_context else {
        return Err(EvalError::plain(
            "finite-kernel realization requires a V2 replay context".to_owned(),
        ));
    };
    let DrawIdentityContext::V2 {
        session,
        seed,
        domain,
        resolver,
        subject,
    } = &draw_context.identity
    else {
        return Err(EvalError::plain(
            "finite-kernel realization requires the sealed V2 replay path; V1 has no signed replay seed"
                .to_owned(),
        ));
    };
    if ctx.active_elements.len() > MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 {
        return Err(EvalError::plain(format!(
            "finite-kernel active-element count {} exceeds {}",
            ctx.active_elements.len(),
            MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2
        )));
    }
    let mut stable = Vec::with_capacity(ctx.active_elements.len());
    for element in &ctx.active_elements {
        let DrawActiveElement::V2(key) = element else {
            return Err(EvalError::plain(
                "finite-kernel V2 realization refused a V1 active-element identity".to_owned(),
            ));
        };
        stable.push(key.clone());
    }
    let carrier = resolver
        .carrier_key(subject, &stable, i64::from(slot))
        .map_err(|error| {
            EvalError::plain(format!(
                "finite-kernel stable carrier identity refused: {error:?}"
            ))
        })?;
    let sample_len = u32::try_from(sample.len())
        .map_err(|_| EvalError::plain("finite-kernel sample QName is too long".to_owned()))?;
    let carrier_len = u32::try_from(carrier.validated_bytes().len())
        .map_err(|_| EvalError::plain("finite-kernel carrier is too long".to_owned()))?;
    let mut keyed_carrier = b"babylon.finite-kernel-carrier.v1\0".to_vec();
    keyed_carrier.extend_from_slice(&sample_len.to_be_bytes());
    keyed_carrier.extend_from_slice(sample.as_bytes());
    keyed_carrier.extend_from_slice(&carrier_len.to_be_bytes());
    keyed_carrier.extend_from_slice(carrier.validated_bytes());
    let mut rng =
        KernelRng::for_carrier_v2(session, *seed, draw_context.tick, domain, &keyed_carrier)
            .map_err(|error| {
                EvalError::plain(format!(
                    "finite-kernel V2 seed derivation refused: {error:?}"
                ))
            })?;
    Ok(rng.next_u64())
}

/// `2^63` as an `f64` — the exact, exclusive upper bound a `floor` result
/// must clear to convert losslessly to `i64` (`i64::MAX` itself is not
/// exactly representable in binary64; rounding it up to the nearest
/// representable value gives this constant, per `i64::MAX as f64`).
const I64_DOMAIN_CEILING: f64 = 9_223_372_036_854_775_808.0;

/// The `floor` intrinsic (ADR188 Row 2 rider): `Real → int`.
///
/// **Domain: `[0, ∞)`, matching the ratified call sites** — the frozen
/// estate's integer-PEOPLE demotions (`vitality.py::_calculate_deaths`:
/// `population` guarded `> 0` and `attrition_rate` clamped to `[0, 1]`
/// before `deaths = int(population * attrition_rate)`;
/// `decomposition.py`: `la_population <= 0` returns early, and
/// `enforcer_fraction`/`proletariat_fraction` are pydantic-constrained
/// non-negative fractions (`config/defines/territory.py`) before
/// `enforcer_pop_gain`/`proletariat_pop`). Not a claim of §3.4 — the
/// intensivity kind rule says nothing about sign — and not "wealth
/// counts" either: `decomposition.py`'s wealth lines
/// (`enforcer_wealth_gain`/`proletariat_wealth`) are NOT `int()`-demoted,
/// only the population lines are. On the ratified domain `floor` and
/// `trunc` — the rider's own paired candidate name, ADR188 Row 2 — are the
/// SAME function, which is exactly why the rider does not have to choose
/// between them. This implementation therefore does not choose either: a
/// negative argument is refused rather than silently rounded one way or
/// the other, so no unratified convention for the disputed domain is
/// baked in by construction (III.11 — a loud failure, never a
/// silently-picked default, matches the §3.3 anti-clamp precedent this
/// document already sets for bounded-scalar arithmetic).
///
/// **A non-`Real`-lane argument is refused, never coerced.** §3.3 promotes
/// `Int` to `Real` only *within* a binary64 expression (`+ − × ÷` and
/// comparison); it says nothing about the intrinsic-call boundary, and no
/// static typechecker exists yet to enforce a declared `:params` type
/// against a call site's argument type (Phase 2 work — §2.7). A bare
/// `(floor 5)` — an `Int` literal, not the result of arithmetic — is
/// therefore an uncoded, structural rejection here: consistent with the
/// no-coercions rule (§3.1) and with `IntrinsicHost`'s own contract that a
/// host's failure is defense-in-depth, not the primary rejection point.
/// Every ratified call site passes the *result* of `population * rate`,
/// which the evaluator's binary64 promotion already makes `Real` before it
/// ever reaches this function.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::DemotionOutOfDomain`] (`E-EVAL-039`) for
/// a negative, non-finite, or `i64`-overflowing argument; [`EvalError::plain`]
/// for a malformed call (wrong arity or a non-`Real` argument — a load-time
/// gate's defense-in-depth, per this module's `IntrinsicHost` contract).
fn eval_floor(args: &[Value]) -> Result<Value, EvalError> {
    let [Value::Real(x)] = args else {
        return Err(EvalError::plain(format!(
            "floor takes exactly one Real-lane argument, got {args:?}"
        )));
    };
    let x = *x;
    if !x.is_finite() {
        return Err(EvalError::coded(
            EvalCode::DemotionOutOfDomain,
            format!(
                "floor of a non-finite value ({x}): E-EVAL-039 — outside the \
                 ratified [0, ∞) domain (ADR188 Row 2)"
            ),
        ));
    }
    if x < 0.0 {
        return Err(EvalError::coded(
            EvalCode::DemotionOutOfDomain,
            format!(
                "floor of a negative value ({x}): E-EVAL-039 — ADR188 Row 2 ratifies \
                 the demotion over [0, ∞) (integer-people counts); floor and trunc \
                 disagree below zero and this rider does not pick one"
            ),
        ));
    }
    let floored = x.floor();
    if floored >= I64_DOMAIN_CEILING {
        return Err(EvalError::coded(
            EvalCode::DemotionOutOfDomain,
            format!("floor({x}) = {floored} exceeds Int's i64 domain (§3.1): E-EVAL-039"),
        ));
    }
    #[allow(clippy::cast_possible_truncation)]
    // In range by the check above; never a silent wraparound.
    Ok(Value::Int(floored as i64))
}

/// The `exp` intrinsic (R10/ADR176 r21, ADR188 cap): `Real → Real`, *e*ˣ via
/// the pinned soft-float crossing (`babylon_kernel::transcendental::exp`,
/// `libm 0.2.16`, `default-features = false` — see that module's doc for the
/// verified dispatch analysis).
///
/// **Domain.** `exp` has no mathematical domain restriction over the reals
/// (`babylon_kernel::transcendental::exp`'s own doc: "`f64` has no domain
/// restriction for `exp`"), so the only argument-side rejection is
/// non-finiteness: `NaN`/`±inf` are not real numbers, so never a legal
/// input, coded [`EvalCode::TranscendentalOutOfDomain`] (`E-EVAL-043`).
/// **This check is load-bearing, not defense in depth mirroring `floor`'s**:
/// `exp(-inf)` is mathematically `0.0` — a FINITE result — so without this
/// check a `NEG_INFINITY` argument would silently succeed with
/// `Ok(Value::Real(0.0))` rather than being refused.
///
/// **Result.** A finite input can still overflow the pinned crossing to
/// `±inf` (e.g. `exp(1e10)`); that is [`EvalCode::NonFinite`] (`E-EVAL-014`),
/// the same code §4.3 already uses for every other binary64 operation
/// producing a non-finite result — never a new code for the same law
/// re-checked at a new call site.
///
/// **A non-`Real`-lane argument is refused, never coerced** — same
/// no-coercions rule as `eval_floor` above (§3.1, §3.3): the intrinsic-call
/// boundary does not promote `Int` to `Real`.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::TranscendentalOutOfDomain`]
/// (`E-EVAL-043`) for a non-finite argument; coded [`EvalCode::NonFinite`]
/// (`E-EVAL-014`) for a non-finite result; [`EvalError::plain`] for a
/// malformed call (wrong arity or a non-`Real` argument).
fn eval_exp(args: &[Value]) -> Result<Value, EvalError> {
    let [Value::Real(x)] = args else {
        return Err(EvalError::plain(format!(
            "exp takes exactly one Real-lane argument, got {args:?}"
        )));
    };
    let x = *x;
    if !x.is_finite() {
        return Err(EvalError::coded(
            EvalCode::TranscendentalOutOfDomain,
            format!(
                "exp of a non-finite value ({x}): E-EVAL-043 — NaN/±inf are not \
                 real numbers, so never a legal e^x argument (R10/ADR176 r21)"
            ),
        ));
    }
    let result = babylon_kernel::transcendental::exp(x);
    if !result.is_finite() {
        return Err(EvalError::coded(
            EvalCode::NonFinite,
            format!("exp({x}) produced a non-finite result ({result}): E-EVAL-014"),
        ));
    }
    Ok(Value::Real(result))
}

/// The `log` intrinsic (R10/ADR176 r21, ADR188 cap): `Real → Real`, the
/// natural logarithm via the pinned soft-float crossing
/// (`babylon_kernel::transcendental::ln`).
///
/// **Domain: `(0, ∞)`.** Unlike `exp`, `log` has a genuine mathematical
/// domain restriction, checked in two steps mirroring [`eval_exp`]'s
/// structure: a non-finite argument (`NaN`/`±inf`) is refused first, coded
/// [`EvalCode::TranscendentalOutOfDomain`] (`E-EVAL-043`) — load-bearing for
/// the same reason as `exp`'s: `log(+inf)` clears the `x <= 0.0` check below
/// (`+inf > 0.0`) and would otherwise fall through to the crossing, which
/// returns `+inf` — itself non-finite, so the call would still fail, but
/// coded [`EvalCode::NonFinite`] instead of the domain code, which is the
/// wrong reason. A non-positive argument is refused second, same code:
/// `x <= 0.0` catches `-0.0` too (`-0.0 <= 0.0` is `true`) — the mirror of
/// `floor`'s negative-zero row, which *accepts* `-0.0` because `-0.0 < 0.0`
/// is `false`. `log` rejects it instead, because `0.0`/`-0.0` sit exactly
/// on the excluded domain boundary.
///
/// **Result.** As `exp`: a non-finite result from the crossing itself
/// (unreachable for any finite `x > 0.0` at `f64` precision, but checked
/// as defense in depth per this crate's own precedent, e.g. `Ratio`'s
/// re-check in `evaluator.rs`) is [`EvalCode::NonFinite`] (`E-EVAL-014`).
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::TranscendentalOutOfDomain`]
/// (`E-EVAL-043`) for a non-finite or non-positive argument; coded
/// [`EvalCode::NonFinite`] (`E-EVAL-014`) for a non-finite result;
/// [`EvalError::plain`] for a malformed call (wrong arity or a non-`Real`
/// argument).
fn eval_log(args: &[Value]) -> Result<Value, EvalError> {
    let [Value::Real(x)] = args else {
        return Err(EvalError::plain(format!(
            "log takes exactly one Real-lane argument, got {args:?}"
        )));
    };
    let x = *x;
    if !x.is_finite() {
        return Err(EvalError::coded(
            EvalCode::TranscendentalOutOfDomain,
            format!(
                "log of a non-finite value ({x}): E-EVAL-043 — outside log's \
                 (0, ∞) domain (R10/ADR176 r21)"
            ),
        ));
    }
    if x <= 0.0 {
        return Err(EvalError::coded(
            EvalCode::TranscendentalOutOfDomain,
            format!(
                "log of a non-positive value ({x}): E-EVAL-043 — log's ratified \
                 domain is (0, ∞); -0.0 is rejected too (-0.0 <= 0.0)"
            ),
        ));
    }
    let result = babylon_kernel::transcendental::ln(x);
    if !result.is_finite() {
        return Err(EvalError::coded(
            EvalCode::NonFinite,
            format!("log({x}) produced a non-finite result ({result}): E-EVAL-014"),
        ));
    }
    Ok(Value::Real(result))
}

/// The one-argument gate shared by the ADR219 sextet's unary members: the
/// `Real`-lane destructuring refuses a non-`Real` argument as a malformed
/// call (uncoded — the no-coercions rule, §3.1/§3.3, the `eval_floor`
/// precedent), and a non-finite argument is `E-EVAL-044`
/// ([`EvalCode::IntrinsicOutOfDomain`]) — defense in depth under §4.3's
/// unrepresentability law, refused at the INPUT so no arm can silently
/// propagate NaN or fail downstream with the wrong code. Factored so the
/// six arms cannot drift apart on either check.
fn one_real_arg(args: &[Value], name: &str) -> Result<f64, EvalError> {
    let [Value::Real(x)] = args else {
        return Err(EvalError::plain(format!(
            "{name} takes exactly one Real-lane argument, got {args:?}"
        )));
    };
    if !x.is_finite() {
        return Err(EvalError::coded(
            EvalCode::IntrinsicOutOfDomain,
            format!(
                "{name} of a non-finite value ({x}): E-EVAL-044 — NaN/±inf are not \
                 real numbers, so never a legal {name} argument (ADR219)"
            ),
        ));
    }
    Ok(*x)
}

/// The two-argument form of [`one_real_arg`], for `min`/`max`.
fn two_real_args(args: &[Value], name: &str) -> Result<(f64, f64), EvalError> {
    let [Value::Real(a), Value::Real(b)] = args else {
        return Err(EvalError::plain(format!(
            "{name} takes exactly two Real-lane arguments, got {args:?}"
        )));
    };
    for (position, x) in [("first", a), ("second", b)] {
        if !x.is_finite() {
            return Err(EvalError::coded(
                EvalCode::IntrinsicOutOfDomain,
                format!(
                    "{name}'s {position} argument is non-finite ({x}): E-EVAL-044 \
                     (ADR219) — refused at the input, never silently propagated \
                     nor silently dropped"
                ),
            ));
        }
    }
    Ok((*a, *b))
}

/// The three-argument form of [`one_real_arg`], for `clamp`.
fn three_real_args(args: &[Value], name: &str) -> Result<(f64, f64, f64), EvalError> {
    let [Value::Real(x), Value::Real(lo), Value::Real(hi)] = args else {
        return Err(EvalError::plain(format!(
            "{name} takes exactly three Real-lane arguments (x, lo, hi), got {args:?}"
        )));
    };
    for (position, v) in [("x", x), ("lo", lo), ("hi", hi)] {
        if !v.is_finite() {
            return Err(EvalError::coded(
                EvalCode::IntrinsicOutOfDomain,
                format!(
                    "{name}'s {position} argument is non-finite ({v}): E-EVAL-044 \
                     (ADR219)"
                ),
            ));
        }
    }
    Ok((*x, *lo, *hi))
}

/// The `sqrt` intrinsic (ADR219 — ADR188 Row 6's fallback rider, taken by
/// the Director ruling of 2026-08-22): `Real → Real` via `f64::sqrt`,
/// IEEE-754's own `squareRoot`, **correctly rounded by the standard's
/// mandate**. There is no platform-libm divergence to pin against, so the
/// pinned soft-float crossing and §4.3's golden-vector clause are DECLINED
/// for this rider — exactly as the `floor` landing declined them (ADR188
/// Row 2, D97) — and this sentence is that disposition made explicit, not
/// a silent omission.
///
/// **Domain: `[0, ∞)`.** A negative argument is refused, coded
/// [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`), never silently
/// answered `NaN`. `-0.0` is IN-domain — `-0.0 < 0.0` is `false`, the
/// `floor` negative-zero precedent — and IEEE's `squareRoot(-0)` is `-0`,
/// so `sqrt(-0.0)` returns `-0.0` with the sign pinned by the standard.
///
/// **No result-side re-check**: `sqrt` of a finite in-domain argument is
/// finite and exactly specified; unlike `exp`'s overflow lane
/// (`E-EVAL-014`), a result guard would be dead code.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`)
/// for a negative or non-finite argument; [`EvalError::plain`] for a
/// malformed call (wrong arity or a non-`Real` argument).
fn eval_sqrt(args: &[Value]) -> Result<Value, EvalError> {
    let x = one_real_arg(args, "sqrt")?;
    if x < 0.0 {
        return Err(EvalError::coded(
            EvalCode::IntrinsicOutOfDomain,
            format!(
                "sqrt of a negative value ({x}): E-EVAL-044 — sqrt's ratified \
                 domain is [0, ∞) (ADR219); -0.0 is accepted (-0.0 < 0.0 is false)"
            ),
        ));
    }
    Ok(Value::Real(x.sqrt()))
}

/// The `round-half-even` intrinsic (ADR219, landing ADR188 Row 3's
/// ratified housekeeping rider and resolving D70): `Real → Real` via
/// `f64::round_ties_even`, IEEE-754's `roundTiesToEven` — exactly
/// specified by the standard, so like `sqrt` it owes no pinned libm and
/// no §4.3 golden-vector family (disposition recorded, not silent).
///
/// **The signature reading (Draft-Ruling Register, open to Director
/// correction).** §3.2 defines the half-even ALGORITHM over exact
/// rationals at a target granularity (micro-units, from exact integer
/// arithmetic, "never by converting to binary64") and §2.7 obliges the
/// kernel to expose "the same algorithm" to rules — without pinning a
/// signature. This landing reads the obligation minimally: the binary64
/// lane rounds WITHIN itself, `(round-half-even x)` = the nearest integer
/// VALUE to `x` as a binary64, ties to the even neighbor. A binary64
/// argument is already an exact rational, so "exactly midway" is decidable
/// exactly and §3.2's exact-arithmetic law is satisfied by construction.
/// The Real→Int demotion remains `floor`'s alone (ADR188 Row 2), and a
/// granularity-general `(round-half-even x g)` form is declined as
/// speculative until a call site needs it.
///
/// **Domain: total over finite reals** — at magnitudes ≥ 2^52 every
/// binary64 is already integral, so the intrinsic is the identity there.
/// A non-finite argument is refused at the shared input gate
/// (`E-EVAL-044`). No result-side re-check (dead code — see `eval_sqrt`).
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`)
/// for a non-finite argument; [`EvalError::plain`] for a malformed call.
fn eval_round_half_even(args: &[Value]) -> Result<Value, EvalError> {
    let x = one_real_arg(args, "round-half-even")?;
    Ok(Value::Real(x.round_ties_even()))
}

/// The comparison rule the ADR219 `min`/`max`/`clamp` share — deliberately
/// NOT `f64::min`/`f64::max`, which are licensed to return EITHER zero on
/// a ±0.0 tie and to propagate NaN: implementation-defined answers of
/// exactly the kind this crate forbids. IEEE-754 comparisons are exactly
/// specified, so the pinned rule is: the second argument wins only on a
/// STRICT comparison; on an equal comparison (`+0.0` vs `-0.0` included —
/// the two compare equal) the FIRST argument wins, bit-pinned by the
/// conformance tests. Non-finite arguments never reach here (the shared
/// gates above refuse them `E-EVAL-044`).
fn pick_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// The `max` half of [`pick_min`]'s pinned comparison rule.
fn pick_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}

/// The `min` intrinsic (ADR219, superseding ADR188 Row 4's "no rider"
/// disposition on #591 item 2's accumulated port evidence):
/// `(real real) → real`, the lesser argument under [`pick_min`]'s pinned
/// rule.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`)
/// for a non-finite argument on either side; [`EvalError::plain`] for a
/// malformed call.
fn eval_min(args: &[Value]) -> Result<Value, EvalError> {
    let (a, b) = two_real_args(args, "min")?;
    Ok(Value::Real(pick_min(a, b)))
}

/// The `max` intrinsic (ADR219 — see [`eval_min`]): the greater argument
/// under [`pick_max`]'s pinned rule.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`)
/// for a non-finite argument on either side; [`EvalError::plain`] for a
/// malformed call.
fn eval_max(args: &[Value]) -> Result<Value, EvalError> {
    let (a, b) = two_real_args(args, "max")?;
    Ok(Value::Real(pick_max(a, b)))
}

/// The `abs` intrinsic (ADR219, superseding ADR188 Row 4): `(real) → real`
/// via `f64::abs` — IEEE-754's sign-bit `abs`, an exact operation with
/// nothing platform-variant to pin (the same D97-style disposition as
/// `sqrt`). `abs(-0.0)` is `+0.0` — the canonical zero, pinned to the bit.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`)
/// for a non-finite argument (`abs(±inf)` would be `+inf` and `abs(NaN)`
/// would be NaN — neither escapes); [`EvalError::plain`] for a malformed
/// call.
fn eval_abs(args: &[Value]) -> Result<Value, EvalError> {
    let x = one_real_arg(args, "abs")?;
    Ok(Value::Real(x.abs()))
}

/// The `clamp` intrinsic (ADR219): `(clamp x lo hi) → real` — the LEGIBLE
/// saturation that §3.3's silent-clamping prohibition points at: the
/// author writes the bound explicitly, and an argument error is loud.
/// `lo > hi` is `E-EVAL-044`, never a silent swap of the bounds; `lo ==
/// hi` is legal (the result is that bound); an in-range `x` returns
/// bit-identical (the identity, never a re-rounded copy); a crossed bound
/// saturates to the bound. Composed on [`pick_max`]/[`pick_min`] so the
/// signed-zero disposition is the same pinned first-argument-wins rule.
///
/// # Errors
///
/// [`EvalError`] coded [`EvalCode::IntrinsicOutOfDomain`] (`E-EVAL-044`)
/// for `lo > hi` or a non-finite argument in any position;
/// [`EvalError::plain`] for a malformed call.
fn eval_clamp(args: &[Value]) -> Result<Value, EvalError> {
    let (x, lo, hi) = three_real_args(args, "clamp")?;
    if lo > hi {
        return Err(EvalError::coded(
            EvalCode::IntrinsicOutOfDomain,
            format!(
                "clamp's lo ({lo}) exceeds hi ({hi}): E-EVAL-044 — loud, never a \
                 silent swap of the bounds (§3.3; ADR219)"
            ),
        ));
    }
    Ok(Value::Real(pick_min(pick_max(x, lo), hi)))
}

#[cfg(test)]
mod tests {
    use super::{
        draw_finite_kernel_ticket, DrawActiveElement, DrawContext, DrawIdentityContext, EvalCode,
        IntrinsicCallCtx, IntrinsicHost, KernelIntrinsicHost, Value,
    };

    fn floor(x: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call("floor", &[Value::Real(x)], IntrinsicCallCtx::context_free())
    }

    fn exp(x: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call("exp", &[Value::Real(x)], IntrinsicCallCtx::context_free())
    }

    fn log(x: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call("log", &[Value::Real(x)], IntrinsicCallCtx::context_free())
    }

    #[test]
    fn floor_of_zero_is_zero() {
        assert_eq!(floor(0.0), Ok(Value::Int(0)));
    }

    /// The mutation-catching case: a ceiling (or round-half-even, or any
    /// convention that rounds up) would answer `4`, not `3`. If a future
    /// edit flips `x.floor()` to `x.ceil()` — or reverses the comparison
    /// direction anywhere in this function — this assertion fails.
    #[test]
    fn floor_of_a_fractional_value_rounds_toward_zero_not_away_from_it() {
        assert_eq!(floor(3.9), Ok(Value::Int(3)));
        assert_eq!(floor(0.1), Ok(Value::Int(0)));
    }

    #[test]
    fn floor_of_an_exact_integer_is_unchanged() {
        assert_eq!(floor(5.0), Ok(Value::Int(5)));
    }

    /// The domain boundary: `floor` and `trunc` are the SAME function on
    /// `[0, ∞)`, so a large positive value in range must succeed exactly
    /// like the small ones above.
    #[test]
    fn floor_of_a_large_in_range_value_succeeds() {
        assert_eq!(floor(1_000_000.7), Ok(Value::Int(1_000_000)));
    }

    #[test]
    fn floor_of_a_negative_value_is_e_eval_039_not_a_silent_pick() {
        let err = floor(-0.1).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DemotionOutOfDomain));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-039");
    }

    #[test]
    fn floor_of_negative_zero_is_accepted_zero_not_rejected() {
        // -0.0 < 0.0 is false in IEEE-754, so this stays in-domain — a
        // regression guard against an implementation that tests the sign
        // bit instead of the value.
        assert_eq!(floor(-0.0), Ok(Value::Int(0)));
    }

    #[test]
    fn floor_of_nan_is_e_eval_039() {
        let err = floor(f64::NAN).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DemotionOutOfDomain));
    }

    #[test]
    fn floor_of_infinity_is_e_eval_039() {
        assert_eq!(
            floor(f64::INFINITY).unwrap_err().code,
            Some(EvalCode::DemotionOutOfDomain)
        );
        assert_eq!(
            floor(f64::NEG_INFINITY).unwrap_err().code,
            Some(EvalCode::DemotionOutOfDomain)
        );
    }

    #[test]
    fn floor_of_a_value_exceeding_i64_range_is_e_eval_039_never_a_wraparound() {
        let err = floor(1e30).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DemotionOutOfDomain));
    }

    #[test]
    fn floor_at_exactly_the_i64_domain_ceiling_is_rejected() {
        // i64::MAX itself is not exactly representable in f64; the nearest
        // representable value is 2^63, which is already one past the true
        // maximum — so a floor result AT that boundary must reject, not
        // silently saturate to i64::MAX.
        let err = floor(super::I64_DOMAIN_CEILING).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DemotionOutOfDomain));
    }

    /// The accept side of that same boundary, at real magnitude — not
    /// `1e6` (too small to catch a mutated ceiling; a verifier confirmed a
    /// ceiling of `2_000_000.0` still passed the whole suite before this
    /// row existed, and that mutation already rejects a US-population-scale
    /// call site, ~3.3e8). `9_223_372_036_854_774_784.0` is the LARGEST
    /// `f64` strictly below `2^63` (binary64's spacing near `2^63` is
    /// `2^10 = 1024`, so `2^63 - 1024`) and fits exactly in `i64` — it must
    /// succeed, byte-exact, never rejected and never rounded to a
    /// different in-range value.
    #[test]
    fn floor_accepts_the_largest_f64_strictly_below_the_i64_domain_ceiling() {
        assert_eq!(
            floor(9_223_372_036_854_774_784.0),
            Ok(Value::Int(9_223_372_036_854_774_784))
        );
    }

    #[test]
    fn floor_rejects_a_non_real_argument_rather_than_coercing() {
        assert!(KernelIntrinsicHost
            .call("floor", &[Value::Int(3)], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call("floor", &[], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call(
                "floor",
                &[Value::Real(1.0), Value::Real(2.0)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
    }

    #[test]
    fn an_undeclared_name_fails_loud_exactly_like_the_empty_host() {
        // `exp` is no longer undeclared as of this task — see the `exp`/`log`
        // tests below. The probe name moved to `tanh` under ADR219:
        // `round-half-even` (this test's former probe) DISPATCHES as of the
        // exact-arithmetic rider train (ADR188 Row 3 landed), while `tanh`
        // stays outside §3.10's cap (Row 8: elimination presented first).
        assert!(KernelIntrinsicHost
            .call(
                "tanh",
                &[Value::Real(1.0)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
    }

    #[test]
    fn exp_of_zero_is_one() {
        assert_eq!(exp(0.0), Ok(Value::Real(1.0)));
    }

    #[test]
    fn log_of_one_is_zero() {
        assert_eq!(log(1.0), Ok(Value::Real(0.0)));
    }

    #[test]
    fn log_of_zero_is_transcendental_out_of_domain() {
        let err = log(0.0).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-043");
    }

    #[test]
    fn log_of_a_negative_value_is_transcendental_out_of_domain() {
        let err = log(-1.0).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    /// The mirror of `floor_of_negative_zero_is_accepted_zero_not_rejected`:
    /// there, `-0.0 < 0.0` is `false`, so `floor` accepts. Here, `log`'s own
    /// domain check is `x <= 0.0`, and `-0.0 <= 0.0` is `true` — so `log`
    /// rejects, and a mutation that swapped this comparison for `<` (making
    /// `-0.0` slip through to `babylon_kernel::transcendental::ln`, which
    /// returns `-inf` for `0.0`/`-0.0`) would flip this test, catching the
    /// swap at the domain guard rather than downstream at the non-finite
    /// result guard.
    #[test]
    fn log_of_negative_zero_is_rejected_not_accepted() {
        let err = log(-0.0).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    #[test]
    fn exp_of_a_large_value_overflows_to_a_non_finite_result() {
        let err = exp(1e10).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::NonFinite));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-014");
    }

    #[test]
    fn exp_rejects_a_non_real_argument_rather_than_coercing() {
        assert!(KernelIntrinsicHost
            .call("exp", &[Value::Int(5)], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call("exp", &[], IntrinsicCallCtx::context_free())
            .is_err());
    }

    /// The non-finite-**input** guard, isolated from the non-finite-**result**
    /// guard below. `exp(-inf)` is mathematically `0.0` — a FINITE result —
    /// so without a dedicated input check, this call would silently SUCCEED
    /// with `Ok(Value::Real(0.0))` rather than being refused. This is the
    /// case that proves the input guard is load-bearing, not dead code
    /// duplicating the result guard's coverage.
    #[test]
    fn exp_of_negative_infinity_is_rejected_at_the_input_guard_not_silently_zero() {
        let err = exp(f64::NEG_INFINITY).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    #[test]
    fn exp_of_positive_infinity_is_rejected_at_the_input_guard() {
        let err = exp(f64::INFINITY).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    #[test]
    fn exp_of_nan_is_rejected_at_the_input_guard() {
        let err = exp(f64::NAN).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    /// `log`'s own non-finite-input differentiator: `+inf` clears the
    /// `x <= 0.0` domain check (`+inf > 0.0`), so without a dedicated
    /// finite-input guard this call would fall through to
    /// `babylon_kernel::transcendental::ln(f64::INFINITY)` — which returns
    /// `+inf`, itself non-finite, so the call would still fail, but coded
    /// `NonFinite` (`E-EVAL-014`) instead of `TranscendentalOutOfDomain`
    /// (`E-EVAL-043`). A mutation deleting the input guard flips this
    /// test's code assertion even though it stays `Err` either way.
    #[test]
    fn log_of_positive_infinity_is_rejected_at_the_input_guard_not_the_result_guard() {
        let err = log(f64::INFINITY).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    #[test]
    fn log_of_nan_is_rejected_at_the_input_guard() {
        let err = log(f64::NAN).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::TranscendentalOutOfDomain));
    }

    #[test]
    fn log_rejects_a_non_real_argument_rather_than_coercing() {
        assert!(KernelIntrinsicHost
            .call("log", &[Value::Int(5)], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call("log", &[], IntrinsicCallCtx::context_free())
            .is_err());
    }

    // ---- ADR219 rider train (Director ruling 2026-08-22): the six
    // exact-arithmetic intrinsics — `sqrt`, `round-half-even`, `min`, `max`,
    // `abs`, `clamp`. None is a transcendental: each crosses via an IEEE-754
    // exactly-specified operation (correctly-rounded `sqrt`;
    // `roundTiesToEven`; comparison/copysign arithmetic), so the pinned
    // soft-float libm and §4.3's golden-vector clause do not apply — the
    // floor rider's disposition (§3.10, D97), recorded per-intrinsic in the
    // spec's normative paragraphs. All six refuse a non-finite argument
    // loudly (E-EVAL-044, `IntrinsicOutOfDomain`) rather than propagate
    // NaN/±inf (§4.3: non-finite values are unrepresentable at any
    // observable point, so the check is defense in depth), and refuse a
    // non-`Real`-lane argument rather than coercing (§3.1/§3.3, the
    // `eval_floor` rule). No result-side re-check exists on any of the six:
    // every one of them maps finite inputs to a finite, exactly-specified
    // result (unlike `exp`'s overflow lane, E-EVAL-014), so a result guard
    // would be dead code — that omission is this comment's explicit
    // disposition, not an oversight.

    fn sqrt(x: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call("sqrt", &[Value::Real(x)], IntrinsicCallCtx::context_free())
    }

    fn round_half_even(x: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call(
            "round-half-even",
            &[Value::Real(x)],
            IntrinsicCallCtx::context_free(),
        )
    }

    fn min2(a: f64, b: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call(
            "min",
            &[Value::Real(a), Value::Real(b)],
            IntrinsicCallCtx::context_free(),
        )
    }

    fn max2(a: f64, b: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call(
            "max",
            &[Value::Real(a), Value::Real(b)],
            IntrinsicCallCtx::context_free(),
        )
    }

    fn abs1(x: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call("abs", &[Value::Real(x)], IntrinsicCallCtx::context_free())
    }

    fn clamp3(x: f64, lo: f64, hi: f64) -> Result<Value, crate::evaluator::EvalError> {
        KernelIntrinsicHost.call(
            "clamp",
            &[Value::Real(x), Value::Real(lo), Value::Real(hi)],
            IntrinsicCallCtx::context_free(),
        )
    }

    fn real_bits(r: Result<Value, crate::evaluator::EvalError>) -> u64 {
        match r {
            Ok(Value::Real(x)) => x.to_bits(),
            other => panic!("expected Ok(Value::Real), got {other:?}"),
        }
    }

    #[test]
    fn sqrt_of_a_perfect_square_is_exact() {
        assert_eq!(sqrt(4.0), Ok(Value::Real(2.0)));
        assert_eq!(sqrt(2.25), Ok(Value::Real(1.5)));
    }

    /// The mutation-catching pin: `sqrt` crosses via IEEE-754's own
    /// correctly-rounded square root, so the result for a non-square is
    /// pinned to the bit. The oracle is `f64::consts::SQRT_2` — std's
    /// compile-time constant, correctly rounded by definition, reached
    /// through NO libm call — so an edit swapping the crossing for an
    /// approximation (a Newton loop without the standard's final rounding)
    /// flips this assertion.
    #[test]
    fn sqrt_of_two_is_the_correctly_rounded_value_to_the_bit() {
        assert_eq!(real_bits(sqrt(2.0)), std::f64::consts::SQRT_2.to_bits());
    }

    /// The `-0.0` boundary, dispositioned exactly like `floor`'s: `-0.0 <
    /// 0.0` is `false` in IEEE-754, so `-0.0` is in-domain — and IEEE's
    /// own `squareRoot(-0)` is `-0`, so the sign is pinned, not accidental.
    #[test]
    fn sqrt_of_zero_and_negative_zero_preserve_the_ieee_signs() {
        assert_eq!(real_bits(sqrt(0.0)), 0.0_f64.to_bits());
        assert_eq!(real_bits(sqrt(-0.0)), (-0.0_f64).to_bits());
    }

    #[test]
    fn sqrt_of_a_negative_is_e_eval_044_not_a_silent_nan() {
        let err = sqrt(-1.0).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::IntrinsicOutOfDomain));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-044");
    }

    /// The input guard is load-bearing the same way `log`'s is: `sqrt` of
    /// `+inf` through the crossing would be `+inf` — non-finite, so the
    /// call would fail anyway but with the wrong code; and `sqrt` of NaN
    /// would be NaN — a silent propagation §4.3 forbids.
    #[test]
    fn sqrt_of_a_non_finite_argument_is_e_eval_044_at_the_input_guard() {
        for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = sqrt(x).unwrap_err();
            assert_eq!(err.code, Some(EvalCode::IntrinsicOutOfDomain), "{x}");
        }
    }

    #[test]
    fn sqrt_rejects_a_non_real_argument_and_wrong_arity_rather_than_coercing() {
        assert!(KernelIntrinsicHost
            .call("sqrt", &[Value::Int(4)], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call("sqrt", &[], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call(
                "sqrt",
                &[Value::Real(1.0), Value::Real(2.0)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
    }

    /// The ruled tie cases (§3.2's half-even algorithm exposed to rules,
    /// ADR188 Row 3, landed by ADR219): exactly-midway values choose the
    /// EVEN neighbor, in both signs — and `-0.5` ties to `-0.0` (zero is
    /// even; the sign is pinned because `roundTiesToEven` specifies it).
    #[test]
    fn round_half_even_ties_choose_the_even_neighbor_in_both_signs() {
        assert_eq!(real_bits(round_half_even(2.5)), 2.0_f64.to_bits());
        assert_eq!(real_bits(round_half_even(3.5)), 4.0_f64.to_bits());
        assert_eq!(real_bits(round_half_even(0.5)), 0.0_f64.to_bits());
        assert_eq!(real_bits(round_half_even(-0.5)), (-0.0_f64).to_bits());
        assert_eq!(real_bits(round_half_even(-2.5)), (-2.0_f64).to_bits());
    }

    #[test]
    fn round_half_even_non_ties_round_to_nearest() {
        assert_eq!(round_half_even(2.4), Ok(Value::Real(2.0)));
        assert_eq!(round_half_even(2.6), Ok(Value::Real(3.0)));
        assert_eq!(round_half_even(-2.6), Ok(Value::Real(-3.0)));
    }

    /// The return type is `Real`, never `Int` — the D-row reading of §3.2's
    /// obligation (the binary64 lane rounds within itself; the Real→Int
    /// demotion remains `floor`'s alone, ADR188 Row 2). A mutation that
    /// demoted would flip this variant assertion.
    #[test]
    fn round_half_even_returns_real_never_a_demoted_int() {
        assert_eq!(round_half_even(7.0), Ok(Value::Real(7.0)));
    }

    /// At `1e300` every binary64 value is already integral (the spacing
    /// exceeds 1), so the intrinsic is the identity there — finite in,
    /// finite out, unchanged.
    #[test]
    fn round_half_even_of_an_already_integral_magnitude_is_the_identity() {
        assert_eq!(round_half_even(7.0), Ok(Value::Real(7.0)));
        assert_eq!(round_half_even(1e300), Ok(Value::Real(1e300)));
    }

    #[test]
    fn round_half_even_of_a_non_finite_argument_is_e_eval_044() {
        for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = round_half_even(x).unwrap_err();
            assert_eq!(err.code, Some(EvalCode::IntrinsicOutOfDomain), "{x}");
        }
    }

    #[test]
    fn round_half_even_rejects_a_non_real_argument_rather_than_coercing() {
        assert!(KernelIntrinsicHost
            .call(
                "round-half-even",
                &[Value::Int(3)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
    }

    #[test]
    fn min_and_max_pick_the_extreme_in_both_argument_orders() {
        assert_eq!(min2(1.5, 2.5), Ok(Value::Real(1.5)));
        assert_eq!(min2(2.5, 1.5), Ok(Value::Real(1.5)));
        assert_eq!(max2(1.5, 2.5), Ok(Value::Real(2.5)));
        assert_eq!(max2(2.5, 1.5), Ok(Value::Real(2.5)));
        assert_eq!(min2(2.0, 2.0), Ok(Value::Real(2.0)));
    }

    /// The signed-zero disposition (ADR219, D-row): `min`/`max` are
    /// comparison-based, never `f64::min`/`f64::max`, because those are
    /// licensed to return EITHER zero on a `±0.0` tie (and propagate NaN)
    /// — an implementation-defined answer of exactly the kind this crate
    /// forbids. IEEE comparisons are exactly specified, so the rule here is
    /// pinned: on an equal comparison (including `+0.0` vs `-0.0`, which
    /// compare equal) the FIRST argument wins.
    #[test]
    fn min_and_max_on_a_signed_zero_tie_return_the_first_argument_to_the_bit() {
        assert_eq!(real_bits(min2(-0.0, 0.0)), (-0.0_f64).to_bits());
        assert_eq!(real_bits(min2(0.0, -0.0)), 0.0_f64.to_bits());
        assert_eq!(real_bits(max2(-0.0, 0.0)), (-0.0_f64).to_bits());
        assert_eq!(real_bits(max2(0.0, -0.0)), 0.0_f64.to_bits());
    }

    #[test]
    fn min_and_max_refuse_a_non_finite_argument_on_either_side() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                min2(bad, 1.0).unwrap_err().code,
                Some(EvalCode::IntrinsicOutOfDomain),
                "min lhs {bad}"
            );
            assert_eq!(
                max2(1.0, bad).unwrap_err().code,
                Some(EvalCode::IntrinsicOutOfDomain),
                "max rhs {bad}"
            );
        }
    }

    #[test]
    fn min_and_max_reject_non_real_arguments_and_wrong_arity() {
        assert!(KernelIntrinsicHost
            .call("min", &[Value::Real(1.0)], IntrinsicCallCtx::context_free())
            .is_err());
        assert!(KernelIntrinsicHost
            .call(
                "max",
                &[Value::Int(1), Value::Real(2.0)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
    }

    #[test]
    fn abs_drops_the_sign_and_canonicalizes_negative_zero() {
        assert_eq!(abs1(-3.5), Ok(Value::Real(3.5)));
        assert_eq!(abs1(3.5), Ok(Value::Real(3.5)));
        // IEEE `abs(-0.0)` is `+0.0` — pinned to the bit, so the result is
        // the canonical zero, never the signed one.
        assert_eq!(real_bits(abs1(-0.0)), 0.0_f64.to_bits());
    }

    #[test]
    fn abs_of_a_non_finite_argument_is_e_eval_044() {
        for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                abs1(x).unwrap_err().code,
                Some(EvalCode::IntrinsicOutOfDomain),
                "{x}"
            );
        }
    }

    /// The in-range passthrough must be bit-exact — the identity on `x`,
    /// never a re-rounded or re-scaled copy of it.
    #[test]
    fn clamp_of_an_in_range_value_is_the_identity_to_the_bit() {
        assert_eq!(clamp3(0.5, 0.0, 1.0), Ok(Value::Real(0.5)));
        assert_eq!(
            real_bits(clamp3(0.1 + 0.2, 0.0, 1.0)),
            (0.1_f64 + 0.2_f64).to_bits()
        );
    }

    #[test]
    fn clamp_saturates_to_the_crossed_bound_and_accepts_the_bounds_themselves() {
        assert_eq!(clamp3(1.5, 0.0, 1.0), Ok(Value::Real(1.0)));
        assert_eq!(clamp3(-0.5, 0.0, 1.0), Ok(Value::Real(0.0)));
        assert_eq!(clamp3(1.0, 0.0, 1.0), Ok(Value::Real(1.0)));
        assert_eq!(clamp3(0.0, 0.0, 1.0), Ok(Value::Real(0.0)));
    }

    /// `lo > hi` is a loud `E-EVAL-044`, never a silent swap of the bounds:
    /// §3.3 frames silent clamping as forbidden quiet degradation — this
    /// intrinsic is the LEGIBLE saturation (the author writes it), and its
    /// argument error is loud for the same reason. `lo == hi` is not an
    /// error: the result is that bound.
    #[test]
    fn clamp_with_lo_above_hi_is_e_eval_044_never_a_silent_swap() {
        let err = clamp3(0.5, 1.0, 0.0).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::IntrinsicOutOfDomain));
        assert_eq!(clamp3(0.5, 1.0, 1.0), Ok(Value::Real(1.0)));
    }

    #[test]
    fn clamp_refuses_a_non_finite_argument_in_any_position() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                clamp3(bad, 0.0, 1.0).unwrap_err().code,
                Some(EvalCode::IntrinsicOutOfDomain),
                "x {bad}"
            );
            assert_eq!(
                clamp3(0.5, bad, 1.0).unwrap_err().code,
                Some(EvalCode::IntrinsicOutOfDomain),
                "lo {bad}"
            );
            assert_eq!(
                clamp3(0.5, 0.0, bad).unwrap_err().code,
                Some(EvalCode::IntrinsicOutOfDomain),
                "hi {bad}"
            );
        }
    }

    #[test]
    fn clamp_rejects_non_real_arguments_and_wrong_arity() {
        assert!(KernelIntrinsicHost
            .call(
                "clamp",
                &[Value::Real(0.5), Value::Real(0.0)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
        assert!(KernelIntrinsicHost
            .call(
                "clamp",
                &[Value::Real(0.5), Value::Int(0), Value::Real(1.0)],
                IntrinsicCallCtx::context_free()
            )
            .is_err());
    }

    /// The lockstep invariant the retired e2e refusal test protected
    /// (`floor_intrinsic_e2e.rs`'s pre-ADR219 `round-half-even` leg),
    /// stated positively: EVERY member of `DECLARABLE_INTRINSICS` carries a
    /// `kernel_signature` row AND a dispatch arm in this match — a cap
    /// widening that forgot either half fails this test rather than
    /// shipping unchecked (`declarations::kernel_signature`'s own doc names
    /// the same invariant for the declaration side). Arguments are shaped
    /// from the signature itself, so the loop needs no per-name table; a
    /// successful call proves the arm, and an `Err` proves it too as long
    /// as it is NOT the fallthrough's "no intrinsic registered".
    #[test]
    fn every_declarable_intrinsic_has_a_signature_row_and_a_dispatch_arm() {
        use crate::declarations::{kernel_signature, IntrinsicTypeName, DECLARABLE_INTRINSICS};
        for name in DECLARABLE_INTRINSICS {
            let (params, _) = kernel_signature(name)
                .unwrap_or_else(|| panic!("{name}: in the cap but no signature row"));
            let args: Vec<Value> = params
                .iter()
                .map(|p| match p {
                    IntrinsicTypeName::Real => Value::Real(0.5),
                    IntrinsicTypeName::Scalar(_) => Value::Int(0),
                })
                .collect();
            match KernelIntrinsicHost.call(name, &args, IntrinsicCallCtx::context_free()) {
                Ok(_) => {}
                Err(err) => assert!(
                    !err.message.contains("no intrinsic registered"),
                    "{name}: has a signature row but no dispatch arm — {err}"
                ),
            }
        }
    }

    // The context-probe name is test-only: this checks that the generic
    // `ctx: IntrinsicCallCtx` parameter reaches `IntrinsicHost::call` without
    // reintroducing an author-visible draw intrinsic.
    struct DrawContextProbeHost;

    impl IntrinsicHost for DrawContextProbeHost {
        fn call(
            &self,
            name: &str,
            _args: &[Value],
            ctx: IntrinsicCallCtx<'_>,
        ) -> Result<Value, crate::evaluator::EvalError> {
            if name == "context-probe" && ctx.draw_context.is_none() {
                return Err(crate::evaluator::EvalError::plain(
                    "context probe called with no DrawContext — missing session/tick \
                     (III.11: a driver that never supplied a session/tick fails \
                     loud, never silently draws 0.0)"
                        .to_owned(),
                ));
            }
            Ok(Value::Real(0.5))
        }
    }

    #[test]
    fn a_host_context_probe_with_no_draw_context_names_the_missing_session_and_tick() {
        let ctx = IntrinsicCallCtx::context_free();
        let err = DrawContextProbeHost
            .call("context-probe", &[], ctx)
            .unwrap_err();
        assert!(err.message.contains("session"), "{}", err.message);
        assert!(err.message.contains("tick"), "{}", err.message);
    }

    #[test]
    #[allow(clippy::items_after_statements, clippy::too_many_lines)]
    fn private_finite_draw_is_keyed_by_every_aj_identity_component_and_has_no_call_order_state() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
        use babylon_graph::substrate::GraphSubstrate as _;
        use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1, RngDomainV2};
        use std::collections::HashMap;

        let mut graph = MemoryGraph::new();
        let subject_a = graph.add_node("class").unwrap();
        let subject_b = graph.add_node("class").unwrap();
        let active_a = graph.add_node("organization").unwrap();
        let active_b = graph.add_node("organization").unwrap();
        let resolver = StableElementResolverV1::seal(
            &graph,
            "demo/world",
            &HashMap::from([
                (subject_a, "subject-a".to_owned()),
                (subject_b, "subject-b".to_owned()),
                (active_a, "active-a".to_owned()),
                (active_b, "active-b".to_owned()),
            ]),
            &HashMap::new(),
        )
        .unwrap();
        let subject_a = resolver.node_key(subject_a).unwrap().clone();
        let subject_b = resolver.node_key(subject_b).unwrap().clone();
        let active_a = resolver.node_key(active_a).unwrap().clone();
        let active_b = resolver.node_key(active_b).unwrap().clone();
        let ordered_active = vec![active_a.clone(), active_b.clone()];
        let reversed_active = vec![active_b, active_a];
        let session_a = ReplaySessionIdV1::try_from("demo/session-a").unwrap();
        let session_b = ReplaySessionIdV1::try_from("demo/session-b").unwrap();

        #[derive(Clone, Copy)]
        struct Key<'a> {
            session: &'a ReplaySessionIdV1,
            seed: ReplaySeed,
            tick: u64,
            domain: &'a str,
            sample: &'a str,
            slot: u32,
            subject: &'a StableElementKeyV1,
            active: &'a [StableElementKeyV1],
        }
        let draw = |key: Key<'_>| {
            let draw_context = DrawContext {
                identity: DrawIdentityContext::V2 {
                    session: key.session,
                    seed: key.seed,
                    domain: RngDomainV2::try_from(key.domain).unwrap(),
                    resolver: &resolver,
                    subject: key.subject.clone(),
                },
                tick: key.tick,
            };
            draw_finite_kernel_ticket(
                key.sample,
                key.slot,
                &IntrinsicCallCtx {
                    draw_context: Some(&draw_context),
                    active_elements: key
                        .active
                        .iter()
                        .cloned()
                        .map(DrawActiveElement::V2)
                        .collect(),
                },
            )
            .unwrap()
        };

        let base_key = Key {
            session: &session_a,
            seed: ReplaySeed::new(7),
            tick: 11,
            domain: "demo/rule-a",
            sample: "demo/sample-a",
            slot: 0,
            subject: &subject_a,
            active: &ordered_active,
        };
        let base = draw(base_key);
        let _unrelated = draw(Key {
            session: &session_b,
            seed: ReplaySeed::new(-9),
            tick: 99,
            domain: "demo/unrelated",
            sample: "demo/unrelated-sample",
            slot: 8,
            subject: &subject_b,
            active: &reversed_active,
        });
        let after_unrelated = draw(base_key);
        assert_eq!(
            base, after_unrelated,
            "an unrelated draw cannot advance hidden state"
        );

        let changed = [
            (
                "session",
                Key {
                    session: &session_b,
                    ..base_key
                },
            ),
            (
                "signed seed",
                Key {
                    seed: ReplaySeed::new(-7),
                    ..base_key
                },
            ),
            (
                "tick",
                Key {
                    tick: 12,
                    ..base_key
                },
            ),
            (
                "firing rule domain",
                Key {
                    domain: "demo/rule-b",
                    ..base_key
                },
            ),
            (
                "sample",
                Key {
                    sample: "demo/sample-b",
                    ..base_key
                },
            ),
            (
                "slot",
                Key {
                    slot: 1,
                    ..base_key
                },
            ),
            (
                "stable subject",
                Key {
                    subject: &subject_b,
                    ..base_key
                },
            ),
            (
                "ordered active elements",
                Key {
                    active: &reversed_active,
                    ..base_key
                },
            ),
        ];
        for (component, key) in changed {
            assert_ne!(base, draw(key), "changing {component} must change the draw");
        }
    }

    // ---- `framed` (plan §3.3): the length-prefix injectivity property
    // `evaluator::eval_intrinsic` relies on when it renders an `Element::
    // Edge`'s source/target/edge-type into ONE chain entry (Task 4.3, I1).
    #[test]
    fn framed_renders_each_segment_length_prefixed_and_pipe_joined() {
        assert_eq!(super::framed(&["ab", "c"]), "2:ab|1:c");
        assert_eq!(super::framed(&["a"]), "1:a");
        assert_eq!(super::framed(&[]), "");
    }

    /// The whole point of the discipline: naive concatenation would let
    /// `("ab", "c")` and `("a", "bc")` collide on `"abc"`. Length-prefixing
    /// makes that impossible.
    #[test]
    fn framed_is_injective_where_naive_concatenation_would_collide() {
        assert_ne!(super::framed(&["ab", "c"]), super::framed(&["a", "bc"]));
    }
}
