(rule synthetic-source/rng-read
  :role mechanic :evidence designed :material-basis "test-local random read" :fuel 128
  (bindings)
  (effects (update-node self synthetic-source/quanta (set (rng-draw 0)))))
