# Rust rule and scenario evidence

The `.bsl`, `.bscn`, and vector files here are inputs to native conformance tests.
They establish the behavior their tests assert; they do not imply that every
rule pack is admitted by the current Michigan material campaign. Michigan V5
uses its captured material definitions and no BSL economic rules.

Python scripts formerly beside these scenarios transcribed the frozen engine.
They imported the retired Python implementation and are no longer executable
repository tools. Their measured vectors and native tests remain. Their source,
and the old `src/babylon/data/defines.yaml` values cited in historical comments,
are recoverable from Git at `3b6ea836dd658e9f92bf6cbb3f42ec074cea6f10`.
Those citations describe provenance, not the current game's parameter authority.

Current authored numeric values live in
`content/scenarios/michigan/defines.toml` under ADR258. In particular,
`production.bsl` still uses a 52-week labor denominator as conformance content;
it needs an explicit period conversion before future game admission.
