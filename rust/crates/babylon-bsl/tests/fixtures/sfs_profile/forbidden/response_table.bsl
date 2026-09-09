(rule synthetic-source/response-table
  :role mechanic :evidence designed :material-basis "test-local response table" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (effects
    (guard (> available 1)
      (update-node self synthetic-source/quanta (set 1)))
    (guard (> available 2)
      (update-node self synthetic-source/quanta (set 2)))))
