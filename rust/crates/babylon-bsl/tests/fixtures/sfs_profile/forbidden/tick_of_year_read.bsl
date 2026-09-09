(rule synthetic-source/tick-of-year-read
  :role mechanic :evidence designed :material-basis "test-local tick-of-year read" :fuel 128
  (bindings (binding current :tick-of-year))
  (effects (update-node self synthetic-source/quanta (set current))))
