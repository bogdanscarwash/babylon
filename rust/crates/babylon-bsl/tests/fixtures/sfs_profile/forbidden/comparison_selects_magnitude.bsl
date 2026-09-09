(rule synthetic-source/comparison-selects-magnitude
  :role mechanic :evidence designed :material-basis "test-local magnitude selector" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (effects
    (update-node self synthetic-source/quanta
      (set (if (> available 1) 2 3)))))
