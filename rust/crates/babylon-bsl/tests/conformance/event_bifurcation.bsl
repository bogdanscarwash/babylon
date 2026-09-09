; Transcribes test_event_evaluator.py:526-590 (resolution selection):
; Python's two conditioned Resolutions become two guards in ONE effect
; list — the routing is content, not engine machinery.
(rule event/bifurcation
  :role mechanic
  :evidence derived
  :material-basis "agitation routes to national identity or class consciousness by solidarity density"
  :fuel 512
  (bindings
    (binding agitation :field social-class/agitation)
    (binding solidarity-density :metric solidarity-density))
  (when (>= agitation 0.5p))
  (effects
    (guard (< solidarity-density 0.1c)
      (update-node self social-class/national-identity (add 0.15i)))
    (guard (>= solidarity-density 0.1c)
      (update-node self social-class/class-consciousness (add 0.15i)))))
