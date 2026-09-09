; Transcribes test_event_evaluator.py:275-283 (aggregation "all"):
; all -> forall (§2.4 coverage table).
(rule event/agitation-everywhere
  :role mechanic
  :evidence derived
  :material-basis "a threshold held by every fraction, not just the vanguard"
  :fuel 512
  (bindings
    (binding agitation :field social-class/agitation))
  (when (forall (nodes NodeType/SOCIAL_CLASS) (>= agitation 0.5p)))
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (threshold agitation))))
