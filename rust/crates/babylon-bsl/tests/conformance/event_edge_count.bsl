; Transcribes test_event_evaluator.py:289-305 (EdgeCondition metric
; "count"): count over the typed edge query; the empty query counts 0 and
; the comparison decides — never a silent skip.
(rule event/solidarity-web
  :role mechanic
  :evidence derived
  :material-basis "the solidarity web's existence conditions rupture routing"
  :fuel 128
  (bindings)
  (when (>= (fold count (edges EdgeType/SOLIDARITY) it) 1))
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (web 1))))
