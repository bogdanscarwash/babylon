; Transcribes test_mechanics.py:54-64: "CLASS_ANALYSIS <= 0 AND MILITANCY <= 0".
(rule doctrine/liquidationism
  :role mechanic
  :evidence derived
  :material-basis "theory and militancy both abandoned dissolves the organization into the movement it tailed"
  :fuel 16
  (bindings
    (binding class-analysis :field organization/class-analysis :optional :default 0)
    (binding militancy :field organization/militancy :optional :default 0))
  (when (and (<= class-analysis 0) (<= militancy 0)))
  (effects
    (emit EventType/DOCTRINE_TRAP (trap-id 2))))
