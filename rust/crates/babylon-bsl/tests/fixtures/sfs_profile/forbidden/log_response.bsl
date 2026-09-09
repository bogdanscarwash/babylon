(rule synthetic-source/log-response
  :role mechanic :evidence designed :material-basis "test-local logarithmic response" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (effects (update-node self synthetic-source/quanta (set (log available)))))
