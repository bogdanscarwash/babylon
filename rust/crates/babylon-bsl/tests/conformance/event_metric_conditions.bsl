; Transcribes test_event_evaluator.py:329-369 (GraphCondition +
; calculate_graph_metric): the six named Python metrics become :metric
; bindings against the registered metric set (§2.5).
(rule event/conjuncture-metrics
  :role mechanic
  :evidence derived
  :material-basis "graph-level density and aggregate positions gate the conjuncture"
  :fuel 64
  (bindings
    (binding solidarity-density :metric solidarity-density)
    (binding total-wealth :metric total-wealth)
    (binding average-agitation :metric average-agitation))
  (when (and (> solidarity-density 0c)
             (>= total-wealth 550)
             (>= average-agitation 0.3c)))
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (gate 1))))
