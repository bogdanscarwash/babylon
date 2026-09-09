; Transcribes test_event_evaluator.py:247-273 (NodeCondition, aggregation
; "any"): any -> exists over the node query (§2.4 coverage table).
(rule event/agitation-anywhere
  :role mechanic
  :evidence derived
  :material-basis "any class fraction past the agitation threshold marks the conjuncture"
  :fuel 512
  (bindings
    (binding agitation :field social-class/agitation))
  (when (exists (nodes NodeType/SOCIAL_CLASS) (>= agitation 0.6p)))
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (threshold agitation))))
