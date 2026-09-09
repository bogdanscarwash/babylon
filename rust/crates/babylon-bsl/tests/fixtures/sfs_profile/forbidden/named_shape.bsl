(rule synthetic-source/named-shape
  :role mechanic :evidence designed :material-basis "test-local named response shape" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (effects (update-node self synthetic-source/quanta (set (sigmoid available)))))
