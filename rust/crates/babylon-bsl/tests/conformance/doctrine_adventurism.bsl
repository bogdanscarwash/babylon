; Transcribes test_mechanics.py:45-52 (TestRealMvpConditions, adventurism)
; and 67-75 (TestMissingTagIsZero): "MASS_LINK <= 0" with absent-tag-reads-0.
; The :optional :default 0 binding IS the trap DSL's pinned
; absent-reads-as-0 site — honest-null: absent = no accumulated strength,
; declared in content, carried on the DEFAULT_ALLOWLIST (§3.5 item 4).
(rule doctrine/adventurism
  :role mechanic
  :evidence derived
  :material-basis "isolation from the mass base severs the practice loop"
  :fuel 16
  (bindings
    (binding mass-link :field organization/mass-link :optional :default 0))
  (when (<= mass-link 0))
  (effects
    (emit EventType/DOCTRINE_TRAP (trap-id 1))))
