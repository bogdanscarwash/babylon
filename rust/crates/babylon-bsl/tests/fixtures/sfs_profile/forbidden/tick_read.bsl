(rule synthetic-source/tick-read
  :role mechanic :evidence designed :material-basis "test-local time read" :fuel 128
  (bindings (binding current :tick))
  (effects (update-node self synthetic-source/quanta (set current))))
