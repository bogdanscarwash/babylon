(rule synthetic-source/threshold-ladder
  :role mechanic :evidence designed :material-basis "test-local threshold ladder" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (when (and (> available 1) (and (> available 2) (> available 3))))
  (effects (update-node self synthetic-source/quanta (set 1))))
