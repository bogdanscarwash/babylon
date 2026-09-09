; Transcribes test_mechanics.py:85-122 (the U11/ADR137 absorbing state):
; practice variables + @coeff thresholds -> :field bindings + :const
; bindings (the @ sigil does not survive, per the §2.2 keyword table).
(rule doctrine/liquidation-absorbing
  :role mechanic
  :evidence derived
  :material-basis "solidarity collapsed, co-optation dominant, base embourgeoised: the absorbing state"
  :fuel 32
  (bindings
    (binding solidarity-mass :field organization/solidarity-mass :optional :default 0)
    (binding co-optive-share :field organization/co-optive-share :optional :default 0)
    (binding petty-bourgeois-drift :field organization/petty-bourgeois-drift :optional :default 0)
    (binding solidarity-liquidation-floor :const doctrine/solidarity-liquidation-floor)
    (binding co-optive-liquidation-threshold :const doctrine/co-optive-liquidation-threshold)
    (binding petty-bourgeois-liquidation-threshold :const doctrine/petty-bourgeois-liquidation-threshold))
  (when (and (<= solidarity-mass solidarity-liquidation-floor)
             (>= co-optive-share co-optive-liquidation-threshold)
             (>= petty-bourgeois-drift petty-bourgeois-liquidation-threshold)))
  (effects
    (emit EventType/DOCTRINE_TRAP (trap-id 3))))
