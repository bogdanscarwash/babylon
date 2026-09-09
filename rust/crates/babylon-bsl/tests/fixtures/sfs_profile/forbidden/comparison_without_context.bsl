(rule synthetic-source/comparison-without-context
  :role mechanic :evidence designed :material-basis "test-local ungoverned comparison" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (when (> available 1))
  (effects (update-node self synthetic-source/quanta (set 1))))
