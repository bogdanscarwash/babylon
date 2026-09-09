; CORRECTION 1 of 4 (event_evaluator.py:313): Python returns 0.0 for an
; unknown graph metric — silent degradation. BSL: an unregistered :metric
; is E-LOAD-011 at content load, never 0.0 (§2.5, §6.3).
(rule event/phantom-metric
  :role mechanic
  :evidence derived
  :material-basis "a metric that does not exist grounds nothing"
  :fuel 16
  (bindings
    (binding phantom :metric no-such-metric))
  (when (>= phantom 1))
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))
