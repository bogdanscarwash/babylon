; W_c > V_c: while core wages exceed the value core labour produces, the
; difference is imperial rent and revolution in the core is materially
; foreclosed. The first rule the Rust engine ever ran.
(rule economics/fundamental-theorem
  :role mechanic
  :evidence derived
  :material-basis "core wages above the value core labour produces is imperial rent; while the gap holds, revolution in the core is materially foreclosed"
  :fuel 64
  (bindings
    (binding wages :field social-class/wages)
    (binding value-produced :field social-class/value-produced))
  (when (> wages value-produced))
  (effects
    (update-node self social-class/imperial-rent (set (- wages value-produced)))))
