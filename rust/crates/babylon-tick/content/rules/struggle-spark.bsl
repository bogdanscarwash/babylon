; Amendment AJ's narrow Struggle spark pilot. The kernel changes material
; state; the adjacent recognizer observes that state. Probability is never an
; event field, and the no-incident branch still realizes a receipted choice.
(rule struggle/spark-mechanic
  :role mechanic
  :evidence designed
  :material-basis "Repression faced supplies the bounded material pressure for a finite excessive-force/no-incident transition; the chosen excessive-force alternative adds bounded agitation backfire and records the observed incident tick."
  :fuel 384
  (bindings
    (binding repression-faced :field social-class/repression-faced)
    (binding agitation-backfire :field social-class/agitation-backfire)
    (binding current-tick :tick)
    (binding spark-scale :const struggle/spark-scale)
    (binding backfire-step :const struggle/backfire-step)
    (binding spark-mass :expr
      (quantize-mass (* repression-faced spark-scale)))
    (binding no-incident-mass :expr (- 1m spark-mass))
    (binding raised-backfire :expr (+ agitation-backfire backfire-step))
    (binding bounded-backfire :expr
      (if (> raised-backfire 1c) 1c raised-backfire)))
  (when (> repression-faced 0i))
  (effects
    (choose :sample struggle/spark :slot 0
      (branch StruggleSparkOutcome/EXCESSIVE_FORCE
        :mass spark-mass
        (effects
          (update-node self social-class/agitation-backfire
            (set bounded-backfire))
          (update-node self social-class/last-incident-known (set 1))
          (update-node self social-class/last-incident-tick
            (set current-tick))))
      (branch StruggleSparkOutcome/NO_INCIDENT
        :mass no-incident-mass
        (effects)))))

(rule struggle/spark-recognizer
  :role recognizer
  :evidence derived
  :projects-kernel struggle/spark
  :material-basis "The post-state incident latch and exact incident tick deterministically recognize the excessive-force branch for the same social-class carrier."
  :fuel 192
  (bindings
    (binding repression-faced :field social-class/repression-faced)
    (binding agitation-backfire :field social-class/agitation-backfire)
    (binding incident-known :field social-class/last-incident-known)
    (binding incident-tick :field social-class/last-incident-tick)
    (binding current-tick :tick))
  (when (and (= incident-known 1) (= incident-tick current-tick)))
  (effects
    (emit EventType/EXCESSIVE_FORCE
      (subject self)
      (repression repression-faced)
      (backfire agitation-backfire)
      (incident-tick incident-tick))))
