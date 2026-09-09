; The worldview/* rule pack at the MINT (ADR206): ONE never-firing load
; probe. The rule pipeline refuses a zero-rule content set outright
; (rule_pipeline.rs's §2.2 check — "a content set needs at least one
; (rule …) top-form, found 0"), so a comment-only pack cannot exercise
; the load-and-tick path at all; the never-firing probe is the in-repo
; idiom for exactly this (production_conformance.rs's scenario-load
; smoke, territory_conformance.rs's no-op rule). The guard is false for
; every legal population, so `fired == 0` and `before == after` hold by
; construction. What the byte pin guards is the substrate LOAD of the
; mint scenario — the canonical state hash covers graph facts only
; (nodes/attributes/edges/hyperedges/edge attributes), so the `defenum`
; declaration itself does NOT move it; the ruled member ORDER is guarded
; by the explicit EnumRegistry ordinal assertion in the same test file
; (worldview_member_order_is_the_ruled_ordinal), not by the hash.
;
; The rule anchors under the ALREADY-registered `consciousness`
; namespace (babylon-tick/src/lib.rs's systems set): the worldview
; estate IS the consciousness domain's content kind, and a content mint
; changes no Rust source. The WorldView enum's first real consumers
; arrive with the class-surface migration port (ADR204 W10's second
; half).
(rule consciousness/worldview-mint-probe
  :role mechanic
  :evidence derived
  :material-basis "load-only smoke: the mint scenario loads and ticks; the mint's pins are the substrate-load hash plus the registry ordinal assertion"
  :fuel 8
  (bindings (binding population :field social-class/population))
  (when (< population 0))
  (effects
    (update-node self social-class/population (set population))))
