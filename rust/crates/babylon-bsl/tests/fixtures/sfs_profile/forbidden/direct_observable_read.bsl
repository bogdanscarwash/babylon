(rule synthetic-source/direct-observable-read
  :role mechanic :evidence designed :material-basis "test-local direct observable read" :fuel 128
  (bindings (binding aggregate :field sfs/aggregate))
  (effects (update-node self synthetic-source/quanta (set aggregate))))
