; Transcribes test_event_evaluator.py:200-214 + 337-343 (sum/max/min over
; an EXTENSIVE field are legal, §3.4 rows 1 and 5) and 185-193 via the
; weighted-mean row: mean of an intensive field with an extensive :weight.
(rule event/wealth-aggregates
  :role mechanic
  :evidence derived
  :material-basis "total and extremal wealth positions gate the conjuncture"
  :fuel 1024
  (bindings
    (binding wealth :field social-class/wealth)
    (binding agitation :field social-class/agitation)
    (binding population :field social-class/population))
  (when (and (>= (fold sum (nodes NodeType/SOCIAL_CLASS) wealth) 550)
             (>= (fold max (nodes NodeType/SOCIAL_CLASS) wealth) 500)
             (<= (fold min (nodes NodeType/SOCIAL_CLASS) wealth) 50)
             (>= (fold mean (nodes NodeType/SOCIAL_CLASS) agitation :weight population) 0.3i)))
  (effects
    (emit EventType/CONSCIOUSNESS_SHIFT (gate 2))))
