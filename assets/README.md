# Babylon runtime assets

This is the canonical root for shipped map, visual, music and sound-effect
bytes. Bevy embeds these files, so launching from another working directory
needs no source checkout or runtime synthesizer.

- `map/`: the deterministic county atlas.
- `visual/`: the interface rasters and illustration estate tracked by
  `design/bevy-assets/manifest.toml`.
- `music/`: original MIDI masters and the rendered Phi and Panopticon themes.
- `sfx/`: the original 58 MIDI cues and six rendered observer cues.

Regenerate the eight Ogg Vorbis files with
`mise run midi:render-observer`. Add `--check` to prove
byte-identical rendering and provenance against `audio-renders.json`. The
FluidR3 GM soundfont is a build input, never bundled or loaded by the game;
its hash and MIT attribution are recorded in that manifest and
`licenses/FluidR3-GM.txt`.

The SFX estate and the 17-track soundtrack in `music/{ambient,superstructure,
periphery,rift,endgame,entity}` retain their CC0 dedication in `audio-LICENSE`.
The older themes and suites retain their separate, unresolved licensing
status in [LICENSING.md](../LICENSING.md). Moving or rendering them grants no
new license. Visuals retain their recorded AGPL-3.0-or-later attribution.

Audio composition generators live in `tools/audio/`; MIDI masters stay here.
The remaining sections describe the CC0 soundtrack and cue estate.

## Layout

```
sfx/
  manifest.toml     ← the source of truth: every note, bend and CC, as data
  ui/              10  menu micro-sounds (move, select, back, deny, open, close, tab,
                       hover, toggle on/off)
  state/            8  simulation state (tick_advance, save/load/fault, autosave,
                       order_committed, game_start/quit)
  alert/            6  notification ladder (info → favorable → warning → tritone klaxon,
                       event minor/major)
  stinger/         10  dialectical punctuation (rupture, solidarity + its dark mirror
                       false_solidarity, atomization, repression, imperial_rent,
                       market_correction, election, policy pass/fail)
  endgame/          5  one terminal stinger per canonical outcome
  entity/           6  the beast's body (#641): the three-beat tick heartbeat
                       (material/action/consequence), extraction bleed, dispossession
                       severance, the missed-beat capital strike
  resistance/      13  human timbre against machine drone: the verb ladder
                       (organize → educate → agitate → protest → alliance → strike →
                       expropriate → sabotage → dual_power → clandestine) and three
                       intercept stingers (capacity, doctrine, repression — drama,
                       not errors)
```

The suite passed a three-lens adversarial critique (thematic power, coverage,
in-UI usability); both arms of the bifurcation are sounded, pitch-bend gestures
carry a declared ±12-semitone RPN range, and the whole set was re-audited for
loudness and MIDI integrity after the fix round.

## The soundtrack (`music/`)

```
music/
  ../../tools/audio/music/composer.py         deterministic composition kit (Score: notes/CCs/bends/tempo/markers)
  ../../tools/audio/music/generate_music.py   registry + renderer: tracks/ → <suite>/<nn>_<name>.mid
  ../../tools/audio/music/tracks/*.py         one pure compose() module per track — the compositions themselves
  ambient/          1  history_breathing — menu music from five coprime cycles (never realigns)
  superstructure/   3  the_ballot (bourgeois waltz vs gavel + jackboots),
                       the_reform_ceiling (the PASOK bleed as form: rising theme, folding pitches),
                       officeholder (a workers' theme captured note-by-note by the Machine)
  periphery/        2  unequal_exchange (velocity transfers periphery→core round by round),
                       superwage (unchanging comfort over a rising Phrygian substrate)
  rift/             2  overshoot (7-beat consumption laps 9-beat regeneration; voices die per lap),
                       the_silent_spring (birdsong shrinking 7…1 notes; one final drop)
  endgame/          5  red_dawn, the_long_winter, iron_consolidation, dual_power, shattered_map
                       — one full theme per canonical outcome, leitmotif-linked to the SFX stingers
  entity/           4  beast_engine (the self-beating idle bed; loop 240.0),
                       tribute_bleed (unequal exchange, vertical), dissection (seven
                       organ solos, reassembled wronger), the_mask (the ticker's one
                       clean harmony, because the mask lies — Director-ruled Arm A;
                       loop 160.0); cue bindings in CUE_MAP.md
```

`red_dawn` settles the estate's oldest debt: the Φ motif's D–A–D–F becomes
**D–A–D–A** (the surplus note deleted) and the theme's eternal dominant finally
resolves to E. Regenerate with `mise run midi:generate-soundtrack`; byte-identity
pinned by `tests/unit/assets/test_music_assets.py`. The legacy `assets/music/`
estate (crisis / revolutionary / fascist suites) still serves the bifurcation
arc; this estate covers what had no music: menu, superstructure, periphery,
rift, endings. Same loudness doctrine and render gain (0.45) as the SFX suite;
the ambient pieces fade in by design.

Each sound's `concept` (why it sounds the way it does) and `trigger_hint` (the
engine `EventType` / system it punctuates) live in `manifest.toml` — that file is
the integration guide.

## Sonic identity

Follows `ai/epochs/epoch3/music-system.yaml`: root note **E**; the Phrygian ♭2
(E–F) is denial; the tritone **E–B♭ is the crisis signal**; minor→major resolution
is reserved for revolutionary outcomes; harmony is solidarity, unison is conformity;
CC93 chorus = solidarity, CC94 detune = atomization, CC71 resonance = repression.
`game_start` and `stinger_imperial_rent` quote the Φ theme's D–A–D–F motif.

## Regenerate / verify

```sh
mise run midi:generate-sfx        # manifest.toml → 58 .mid files (deterministic)
mise run test:q -- tests/unit/assets/test_sfx_assets.py   # byte-identity contract
```

Rendering is a pure function of the manifest: same manifest bytes → same `.mid`
bytes, pinned by `tests/unit/assets/test_sfx_assets.py`. Edit `manifest.toml`,
regenerate, commit both.

## Audition / render

```sh
mise run midi:play -- assets/sfx/alert/alert_critical.mid       # hear one
for f in assets/sfx/*/*.mid; do mise run midi:play -- "$f"; done  # hear all
mise run midi:to-ogg -- assets/sfx/ui/ui_select.mid             # game-format render
```

**Recommended synth gain: 0.45** (e.g. `fluidsynth -g 0.45` with FluidR3_GM).
The suite is mixed hot — at gain 0.45 the largest stingers peak at −0.8…−4 dBFS
with **zero clipped samples**, verified computationally across all 58 SFX + all
17 soundtrack tracks (2026-08-18: the 19 entity/resistance SFX + 4 entity-suite
tracks re-audited at #641's landing, worst case −5.79 dBFS/zero clips, joining
the original 39 SFX + 13 tracks' own gain-0.45 audit from ADR152/ADR153)
(the revolutionary-victory orchestra hit is the suite's absolute peak and clears
full scale up to gain 0.47). Bell and reverb tails ring past the notated end by
design; every note-off is present (integrity-audited: no stuck notes, all pitch
bends reset). Mix priority between concurrently-firing sounds is the client
mixer's job — assets ship full-scale.

Every `.mid` re-asserts its own channel state (program, volume, expression,
reverb, chorus, pan, bend reset) at tick 0, so files are safe to fire in any
order on a shared synth.
