(rule synthetic-source/year-read
  :role mechanic :evidence designed :material-basis "test-local year read" :fuel 128
  (bindings (binding current :year))
  (effects (update-node self synthetic-source/quanta (set current))))
