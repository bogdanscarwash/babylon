(rule synthetic-source/exp-response
  :role mechanic :evidence designed :material-basis "test-local exponential response" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (effects (update-node self synthetic-source/quanta (set (exp available)))))
