; CORRECTION 4 of 4 (event_evaluator.py:103): Python treats an empty
; precondition set as True — silent permissiveness. BSL: (when) is
; E-PARSE-020; "always" is written by OMITTING the clause or (when #t),
; so the empty case can never be an accident (§2.3, §6.3).
(rule event/empty-when
  :role mechanic
  :evidence derived
  :material-basis "an event with no stated preconditions has no material trigger"
  :fuel 16
  (bindings)
  (when)
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))
