(rule synthetic-source/absolute-schedule
  :role mechanic :evidence designed :material-basis "test-local absolute schedule" :fuel 128
  (bindings (binding current :tick))
  (when (> current 10))
  (effects (update-node self synthetic-source/quanta (set 1))))
