(rule synthetic-source/scoped-mechanic
  :role mechanic
  :evidence designed
  :material-basis "a test-local source transfers one declared quantum only when its stock and an existing synthetic link permit the transfer"
  :fuel 128
  (bindings
    (binding available :field synthetic-source/quanta)
    (binding quantum :const synthetic/transfer-quantum)
    (binding minimum-link-strength :const synthetic/minimum-link-strength))
  (when
    (and
      (> available quantum)
      (> (fold sum (edges EdgeType/SYNTHETIC_LINK)
           (field-of it synthetic-link/strength)) minimum-link-strength)))
  (effects
    (update-node self synthetic-source/quanta (sub quantum))))
