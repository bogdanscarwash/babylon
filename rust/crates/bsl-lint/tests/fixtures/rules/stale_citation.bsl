; Fixture content for bsl-lint's citation-drift integration test (W1.1 RED).
; Every :material-basis below cites the REAL frozen file
; src/babylon/engine/systems/solidarity.py at the p27-python-freeze tag
; (202 lines) — the fixture doesn't invent a fake target, it exercises the
; real resolution + tier machinery against a real, stable citation surface.

(rule fixture/clean :role mechanic :evidence derived :material-basis "Solidarity docstring names Proletarian Internationalism as the counterforce to imperial rent bribery (solidarity.py:1-14)." :fuel 64
  (bindings)
  (effects (update-node self social-class/agitation (add 0.01i))))

(rule fixture/out-of-bounds :role mechanic :evidence derived :material-basis "SolidaritySystem.step transmits consciousness across the whole file (solidarity.py:97-999)." :fuel 64
  (bindings)
  (effects (update-node self social-class/agitation (add 0.01i))))

(rule fixture/keyword-miss :role mechanic :evidence derived :material-basis "Xenotransplantation protocol details appear at this span (solidarity.py:1-3)." :fuel 64
  (bindings)
  (effects (update-node self social-class/agitation (add 0.01i))))
