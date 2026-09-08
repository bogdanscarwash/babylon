Babylon Architecture Rationale
==============================

``CONSTITUTION.md`` v4.1.0 is the live authority. ``NORTH_STAR.md`` gives the
game direction and gate order. This commentary explains the rationale behind
that direction.

Game before model
-----------------

Babylon is an entertainment-first game. It uses political economy as a causal
domain, not as a promise to reproduce history. Theory constrains the causal
model without a fixed result.

The game can use a declared liberty when it improves play. The liberty must
agree with the causal model. Each substantive value must use one of
four source classes: ``Observed``, ``Derived``, ``Calibrated``, or ``Designed``.

Emergence before scripts
------------------------

A shock adds pressure. It does not write unemployment, shortages, death,
default, victory, or systemic crisis. Ordered rules and feedback produce those
effects.

Historical data gives a benchmark, not a required trajectory. A benchmark must
test causal signatures, varied effects, hysteresis, and counterfactual response.

Determinism has a narrow meaning. Equal input bytes must produce equal output
bytes and hashes. That identity proves a repeatable computation, not scientific
truth.

Engine and content
------------------

The shipping path uses Rust. The relation, BSL, and tick crates judge the four-week
tick. Live BSL expresses governed causal rules. No executable shocks run.
In the planned action slice, BSL will let actions run, charge costs, choose
targets, and encode political results.

The Python engine is a frozen behavioral reference. Its tests and traces help
check a Rust port. Python also prepares data and runs selected periphery.

The Bevy client is an administrative viewer. It can show the county atlas and
move ticks forward, but committed BSL has no player action.

.. Vale: this paragraph preserves literal persistence and schema identifiers.
.. vale ste.UnapprovedWords = NO
.. vale ste.NounClusters = NO

The Python periphery has mutable SQLite replay, atomic Postgres tick commits,
partial ``babylon_meta`` state, an action pipeline, and ``pgvector``. The full
v4 Rust three-schema boundary, commit envelope, Archive outbox, and BSL-Bevy
player decision cycle are plans. They have not landed.

.. vale ste.NounClusters = YES
.. vale ste.UnapprovedWords = YES

AI observes
-----------

AI can parse, retrieve, and narrate. AI must not judge a game rule. A run must
produce the same mechanical result with or without narrative AI.

The planned Archive will give narrative context through player knowledge. Fog
must apply before retrieval. Narrative text must not change material truth.

Pure boundaries
---------------

Game data and game judgment must have a clear boundary. Core relation, BSL, and
tick crates have no database dependency. Storage starts only after a tick
passes its checks.

Types make the boundary clear. Constrained values reject invalid data near the
source. The byte layout is canonical. The frozen Python path has persisted
replay. The Rust port uses behavioral contracts. Gate 3 will add durable Rust
replay.

Data-driven rules are important. Coefficients belong in governed data, not
in hidden conditionals. BSL lets content express the licensed algebra without
the creation of a new mathematical primitive.

Player knowledge
----------------

Rich information has value when it helps a player choose. Each game map, chart,
relation diagram, or other display must answer a decision question.

Reference information belongs in the Archive or an administrative display. An
administrative display cannot pass a game milestone.

Tests as durable contracts
--------------------------

TDD uses a red test, a small behavior change, and a refactor. A port must keep
the observable contract, not the old internal structure.

Different tests find different faults. Babylon uses unit tests, property laws,
golden replay, scenario benchmarks, boundary tests, and mutation tests.

Python tests are useful when they specify behavior or Python periphery. A
Rust replacement can retire an engine-specific Python test after a durable
replacement contract exists.

Small and accurate documentation
--------------------------------

Documentation must answer a specific question. Each page must serve one audience and
one purpose. A small accurate set is better than a large stale set.

Keep old ADRs unchanged because they record the rationale for an earlier
choice. A new decision adds a new ADR and marks the prior rule as historical.

Related pages
-------------

.. Vale: the next role contains a literal Sphinx document path.
.. vale ste.Ambiguity = NO

- :doc:`/concepts/architecture`

.. vale ste.Ambiguity = YES

- ``CONTRIBUTORS.md`` for change and merge rules
- ``CLAUDE.md`` for repository commands and live facts
