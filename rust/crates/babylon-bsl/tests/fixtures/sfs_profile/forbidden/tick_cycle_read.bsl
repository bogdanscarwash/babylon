(rule synthetic-source/tick-cycle-read
  :role mechanic :evidence designed :material-basis "test-local tick-cycle read" :fuel 128
  (bindings (binding current :tick-in-cycle 52))
  (effects (update-node self synthetic-source/quanta (set current))))
