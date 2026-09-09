# Archive Interface — stack research & opinionated recommendation (addendum to the design brief)

**Provenance:** four parallel research agents over primary sources (PyPI timestamps, GitHub commit logs/releases, official docs, CVE databases), 2026-07-19, followed by Context7 documentation verification of the finalists' APIs. Companion to `2026-07-19-archive-interface-design.md`. **Status: advisory evidence + opinion.** The brief's non-prescription clause stands — the implementing session confirms these choices in-env and records them in the stack ADR. Where this doc says "pin," treat it as a starting pin, not law.

## The recommendation, in one table

| Decision | Pick | Starting pin | One-line why |
|---|---|---|---|
| TUI framework | **Textual** | `8.2.x` (exact) | Only Python candidate with hover events, CSS theming, fuzzy command palette, and a proven SVG snapshot-test lane |
| Markdown parse | **markdown-it-py + mdit-py-plugins** | `4.2.x` / `0.6.x` | Already Textual's parser; `parser_factory` injects our plugins; flat fence tokens carry the directive info string |
| Directive grammar | **Fenced directives only** (```` ```{name} args ````) | — | Textual's Markdown walker breaks on paired container tokens (see §2 — the key finding) |
| Wikilinks | **~30-line custom inline rule → standard link tokens** | — | Emitting `link_open/text/link_close` with `babylon://` hrefs makes `Markdown.LinkClicked` work for free |
| Templating | **Jinja2 `ImmutableSandboxedEnvironment`** | `3.1.6` | Sandbox CVEs fixed as of 3.1.6; minijinja-py disclaims its own sandbox for passed Python objects |
| Choropleth / images | **Kitty graphics protocol via textual-image; half-block cells as the portable floor** | `textual-image 0.13.x` | Braille is 1 color/cell — wrong for filled maps; half-block gives 2 colors/cell; TGP gives raster on Kitty/Ghostty/iTerm2 |
| Charts / canvas | **textual-plot + textual-hires-canvas** | `0.10.x` / `0.14.x` | Actively maintained 2026 (mouse zoom/pan, braille/quadrant modes); textual-plotext is 20 months stale |
| Full-text search | **Postgres FTS (tsvector + GIN, pg_trgm)** | in-tree | Already a dependency; the epistemic fog filter is literally `WHERE entity_id = ANY(:known)` in the same query |
| Vault git | **dulwich** | `1.2.x` | Pure Python, active (May 2026), no system git; commit dates pinned to sim time for deterministic hashes |
| Snapshot tests | **pytest-textual-snapshot** | `1.1.0` | Textual's own regression gate (448 SVG baselines upstream); verify against repo's pytest 9 |
| Export/publish | **Sphinx (custom `babylon` domain), export path only** | existing | Whole-project builder: right for AAR publishing, wrong for per-tick rendering |

## 1. TUI framework — Textual, eyes open

**Post-shutdown health is better than feared.** Textualize Inc. wound down May 2025, but the release flow never stopped: 7.0.0 (2026-01-03) → 8.0.0 (2026-02-16) → 8.2.8 (2026-06-30, latest), with Will McGugan authoring ~92% of commits since Nov 2025 (381/412) and 20–80 commits/month through June 2026. Kitty keyboard protocol landed in 8.2.7 ("The more Kitty Release"). Rich also still moves (15.0.0, Apr 2026). No fork or successor exists.

**Why it wins on requirements:** hover is a first-class concept (`Enter`/`Leave` events, `:hover` CSS) — load-bearing for the Vic3 peek mechanism; `Theme` + design tokens map cleanly onto §9b ksbc tokens; the built-in command palette has fuzzy search and a `Provider` API for custom entity search; `Pilot` (`press`/`click`/`hover`) + pytest-textual-snapshot give the III.12-grade deterministic test lane — Textual itself is regression-gated by 448 byte-compared SVG snapshots.

**Risks, honestly:** bus factor ≈ 1 (mitigate: pin exact, budget upgrade time, vendor-readiness — it's pure Python; `textual-speedups` optional Rust accelerator exists). Major-version churn is real (7 majors in ~12 months) but each break was narrow renames/semantics; pin and upgrade deliberately. `pytest-textual-snapshot` pins `syrupy==4.8.0` exactly and declares `pytest>=8` — **verify against the repo's pytest ^9 in the spike**.

**Runners-up:** urwid is surprisingly alive (4.0.4, 2026-07-13; multiple maintainers — arguably better bus factor) but enables mouse modes 1000/1002/1006 only — **no motion-without-button reporting, so no hover**, and no snapshot/test harness. That fails R3's spirit and the testing lane; keep it as the break-glass fallback. prompt_toolkit is in maintenance mode (8 commits in 2026, 611 open issues). pyTermTk is a perpetual-alpha solo project (has 1003 motion, no test story). PyRatatui (PyO3 bindings to Ratatui, announced by Ratatui's maintainer, first release 2026-03) is embryonic — worth a glance in a year; interesting adjacency given hypergraph-rs is already PyO3-shaped, but pre-1.0 with no track record.

## 2. Markdown pipeline — the key engineering finding

Read `textual/widgets/_markdown.py` before writing any directive code. Two hard facts from source (confirmed against current main, 1,723 lines):

1. **Paired custom tokens break the widget.** The token walker handles closes generically — any `*_close` unconditionally pops the block stack. A `mdit_py_plugins.container` pair (`container_x_open`/`container_x_close`) reaches `unhandled_token` only for the open; the close then wrong-pops or `IndexError`s. **Therefore: no colon-fence/container directive syntax. Fenced directives only.** A fence arrives as a single flat token with the full info string (`{statblock} county/26163`) in `token.info` — no stack interaction.
2. **Fence dispatch is a subclass, not a registry.** `BLOCKS["fence"]` must map to a `MarkdownFence` subclass (there's an `assert issubclass` in the dispatch path), and ONE class handles all fences. So the pattern is: subclass `MarkdownFence`, inspect `self._token.info` — if it starts with `{`, route to the directive widget registry (`statblock`, `paoh`, `absence`, `narrative`, `peek`); otherwise fall through to normal syntax-highlighted code. This stays entirely within the documented extension surface (`BLOCKS` / `get_block_class` / `unhandled_token` / `parser_factory`), which the 5.0.0 changelog explicitly calls "the extension mechanism." Degradation elsewhere is automatic: Obsidian/nvim show the same fence as a code block.

**Wikilinks:** no maintained Python wikilink plugin emits widget-compatible tokens. `mdformat-wikilink` (0.3.0, 2025-10) contains a clean single-token inline rule — but Textual's inline content builder **silently drops unknown inline tokens**, so single-token approaches render as nothing. Write our own ~30-line inline rule that rewrites `[[target|alias]]` into standard `link_open`/`text`/`link_close` triples with `href="babylon://target"` — then `Markdown.LinkClicked` fires natively with `open_links=False`. Use mdformat-wikilink's regex as the reference implementation.

**Front matter:** `mdit_py_plugins.front_matter` emits a flat, hidden, self-closing token — safe with the widget; read the YAML at materialization time, not render time.

**Rejected:** **MyST-Parser runtime** — 5.x unconditionally depends on Sphinx 8/9 + docutils even for parse-only use, is doc-build-shaped, py≥3.11; its concepts (already in the brief) survive, its runtime stays on the export path. **mistune 3.3.x** — genuinely good `FencedDirective`/`DirectivePlugin` API and active (3.3.3, 2026-07-09), but adopting it abandons Textual's Markdown widget entirely (it consumes markdown-it tokens), meaning hand-building every block widget; also the author's energy is visibly moving to his successor parser (Wenmode). **marko** — clean extension model, small community, spec-over-speed; no advantage here.

## 3. Templating — Jinja2 sandbox, with the NetBox lesson written into law

**Pick:** Jinja2 3.1.6 `ImmutableSandboxedEnvironment` (blocks mutations of list/set/dict on top of the standard sandbox).

**Evidence:** the three sandbox-escape CVEs (2024-56201, 2024-56326, 2025-27516) are all fixed as of 3.1.6 (2025-03-05, current), and all required attacker-controlled template content — exactly the modding threat model, so the patched state matters. The 2026 real-world incident (NetBox, CVE-2026-29514) was **not** a Jinja flaw: the app let data configure the environment constructor (`finalize=subprocess.getoutput` → RCE outside sandbox interception).

**Rules for the spec (each one traces to an incident or a doc warning):** environment construction is code, never data — `finalize`, globals, and filters are hardcoded; the filter list is small, vetted, and pure (both 2024 CVEs route through custom filters); template context is plain dicts from named projections (already brief law — now also a security boundary); no `now()`/`random` (not in Jinja core; simply never inject them); Jinja has **no execution-step limit** — acceptable while templates are first-party, so add a render watchdog at the materializer level and revisit when modding opens.

**Rejected (for now): minijinja-py** — the Rust engine's value-model sandbox and `fuel` instruction budget are exactly the right design for untrusted templates, **but the Python binding explicitly disclaims it**: "no extra security layer in use at the moment so take care of what you pass there"; passed Python objects expose all non-underscore methods; known compat gaps ("quite a few templates will refuse to render"). Wrong maturity today. Long-term footnote: if page materialization ever moves into a Rust tool (the hypergraph-rs precedent), Rust-side minijinja with fuel limits becomes the natural modding-surface engine.

## 4. Graphics — two lanes, and braille is the wrong lane for the map

**Protocol reality (mid-2026):** Kitty graphics protocol is the winning raster protocol for this audience — kitty, Ghostty 1.3.x, iTerm2 3.6+, Konsole (partial), WezTerm (partial/nonconformant, stale stable since 2024-02). Sixel is deliberately absent from kitty and Ghostty, flickery inside Textual, and CPU-heavy — **skip sixel as a target entirely**. TGP works over ssh (direct escape-code transmission, base64 ≤4096-byte chunks); tmux doesn't pass TGP (PoC branch only, 2026-03) — acceptable, the app is full-screen and doesn't need tmux.

**The cell-encoding fact that settles the choropleth:** braille gives 2×4 dots but **one foreground color per cell** — right for line art and scatter, wrong for filled area color. Half-block (▄) gives 1×2 px with **two colors per cell**; quadrant 2×2 likewise. At a 200×50-cell map pane: half-block = 400×100 px ≈ 13 px per county mean at ~3,100 counties — county-resolution color needs the raster lane or an aggregated tier. This confirms the brief's level-lattice ruling empirically: **cell-art choropleths render at EA/state tiers; county-resolution fills use TGP raster; braille is reserved for line-work** (transport corridors, sparklines, PAOH connectors).

**Libraries:** `textual-image` 0.13.x (active through May–Jul 2026, requires py≥3.12 — matches the repo) for TGP with automatic cell fallback; caveat from its docs: terminal-capability detection must run **before** the Textual app starts. `rich-pixels` is dormant (2024) but its half-block algorithm is ~50 lines — vendor it if textual-image's fallback doesn't fit. Charts: **textual-plot + textual-hires-canvas** (David Fokkema, releases Jan–Feb 2026, mouse zoom/pan, braille/quadrant/half-block modes, NumPy inner loops) over Textualize's stale `textual-plotext`. PAOH needs no canvas library at all — rows, columns, dots, and vertical rules are styled text. **Avoid:** drawille (AGPL — license contamination risk, and monochrome), notcurses (separate render model, admittedly incomplete Python bindings), mapscii (dormant JS; its earcut→bresenham→braille pipeline is prior art to imitate, not a dependency). `chafa` (active, 1.18.2) is a fine external tool for AAR export imagery.

**No published repaint benchmarks exist** for large cell canvases in Textual — the spike must measure the map-room pane empirically.

## 5. Search — the fog is a WHERE clause

**Pick: Postgres FTS.** `GENERATED ALWAYS AS ... STORED` tsvector column on the page-projection tables + GIN index; `ts_rank_cd` for ranking; `ts_headline` only on the final displayed page of results (docs warn it re-reads source documents); `pg_trgm` for typo tolerance. A ~10k-page wiki is trivial scale for it. The decisive argument is epistemic gating: filtering results to what the player's org knows composes as `AND entity_id = ANY(:known_entities)` **in the same SQL query** — the fog filter is a WHERE clause, no second search infrastructure, no sync problem. psycopg3 is already in the tree.

**Rejected:** tantivy-py (healthy — 0.26.0, 2026-04 — but a second datastore with index files to rebuild and no SQL-composable epistemic filter); whoosh and whoosh-reloaded (both explicitly unmaintained as of 2026).

## 6. Vault git + snapshot testing

**dulwich 1.2.x** (pure Python, active May 2026, no system-git dependency) for programmatic vault commits. Determinism detail worth institutionalizing: git commit hashes embed author/committer timestamps — set them from **sim time** (tick-derived), not wall-clock. Deterministic vault + deterministic dates = reproducible commit hashes (III.7 extended to the vault), and the Archive's revision history is dated in-world. `pytest-textual-snapshot` 1.1.0 for SVG snapshots; flag the `syrupy==4.8.0` exact pin and `pytest>=8` declaration for compatibility testing against the repo's pytest ^9.

## 7. What the spike must verify in-env (falsifiable checklist)

1. `MarkdownFence` subclass dispatching on `token.info` renders `{statblock}` / `{absence}` / `{narrative}` fences from a real materialized page — and unknown fences still syntax-highlight.
2. The wikilink inline rule produces clickable links firing `Markdown.LinkClicked` with `babylon://` hrefs; unknown-entity links style as red links.
3. `textual-image` TGP path inside Kitty, and over an ssh hop; confirm the detect-before-app-start constraint is manageable in the app's boot sequence.
4. Half-block map pane at ~200×50 cells repaints at interactive feel (no published benchmarks — measure).
5. `pytest-textual-snapshot` + syrupy pin coexists with the repo's pytest ^9 lane; one SVG baseline committed and byte-stable across two runs.
6. Hover `Enter`/`Leave` reliability in Kitty for the peek plate; keyboard peek parity.
7. Command palette `Provider` fuzzy-matching over ~10k entity names at acceptable latency (or a custom index behind it).
8. Jinja `ImmutableSandboxedEnvironment` renders the county template from a projection dict; a template attempting `__class__` access raises `SecurityError` (test it).

## Sources (primary)

- Textual: https://github.com/Textualize/textual · https://textual.textualize.io/blog/2025/05/07/the-future-of-textualize/ · https://textual.textualize.io/widgets/markdown/ · https://textual.textualize.io/guide/command_palette/ · https://github.com/Textualize/pytest-textual-snapshot · https://github.com/Textualize/textual/discussions/6414 · https://github.com/Textualize/textual/discussions/5616
- Parsing: https://github.com/executablebooks/markdown-it-py · https://github.com/executablebooks/mdit-py-plugins · https://github.com/tmr232/mdformat-wikilink · https://github.com/executablebooks/MyST-Parser · https://github.com/lepture/mistune · https://github.com/frostming/marko
- Templating: https://jinja.palletsprojects.com/en/stable/sandbox/ · https://github.com/advisories/GHSA-cpwx-vrp4-4pq7 · https://nvd.nist.gov/vuln/detail/CVE-2024-56326 · https://chocapikk.com/posts/2026/netbox-export-template-rce/ · https://github.com/mitsuhiko/minijinja/tree/main/minijinja-py
- Graphics: https://sw.kovidgoyal.net/kitty/graphics-protocol/ · https://github.com/lnqs/textual-image · https://github.com/davidfokkema/textual-plot · https://pypi.org/project/textual-hires-canvas · https://github.com/darrenburns/rich-pixels · https://github.com/ghostty-org/ghostty/discussions/2496 · https://github.com/tmux/tmux/issues/4902 · https://www.arewesixelyet.com/ · https://github.com/rastapasta/mapscii · https://github.com/hpjansson/chafa
- Search & git: https://www.postgresql.org/docs/current/textsearch-controls.html · https://github.com/quickwit-oss/tantivy-py/releases · https://github.com/Sygil-Dev/whoosh-reloaded · https://pypi.org/project/dulwich/ · https://pypi.org/project/pygit2/
