(rule territory/detroit-admin-control-noop
  :role mechanic :evidence derived :material-basis "administrative identity witness only; no player or geographic inference"
  :fuel 32
  (bindings
    (binding fips :field territory/administrative-fips))
  (when (> fips 99999))
  (effects
    (update-node self territory/administrative-fips (set fips))))
