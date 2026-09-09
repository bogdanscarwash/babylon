# The Textual Manual (Babylon Archive)

> **Verified against textual 8.2.8, 2026-07-21.**
> Companion packages: `textual-image 0.13.2`, `pytest-textual-snapshot 1.1.0`, `syrupy 5.5.3`,
> `rich` (vendored dep). Repo pin `textual>=8.2,<8.3` (ADR099).
>
> **Register:** Diataxis *reference* only — no tutorial prose, no "getting started." Every factual
> claim traces to a source location. Two citation namespaces are used throughout:
> - **`module.py:line`** — the installed Textual/Rich/plugin tree under
>   `/home/user/projects/game/babylon/.venv/lib/python3.12/site-packages/` (Textual paths are
>   relative to `.../site-packages/textual/`).
> - **`repo:path:line`** — a file in this repository (root `/home/user/projects/game/babylon/`).
>
> Claims that could not be verified against installed source are marked **`[UNVERIFIED]`** with a
> reason. The [Verification ledger](#verification-ledger) at the end records every flag from the
> four source drafts and its disposition (resolved this pass, or kept unverified and why).

---

## 0. Quick reference — the 20 things a lane agent needs hourly

One screen. Each row links to the Part that owns the detail.

### 0.1 Launch under test (Pilot)

```python
async with app.run_test() as pilot:          # app.py:2133 — async context manager, yields Pilot
    await pilot.press("t", "ctrl+o")          # send keys (see 0.4)
    await pilot.click("#dossier")             # mouse; selector or (widget, offset)
    await pilot.pause()                        # settle to CPU-idle (adaptive, ≤1s)
    assert app.query_one("#status", Label)...  # assert against live DOM
# on __aexit__: app._shutdown(); re-raises app._exception if the app panicked (app.py:2216-2218)
```
`run_test` defaults: `headless=True`, `size=(80,24)`, `tooltips=False`, `notifications=False`,
`message_hook=None`. Full Pilot API → **Part III §3.1**.

### 0.2 Message-handler naming (the `on_*` table)

Handler name = `on_<name>` — this `on_<name>` string IS the `handler_name` ClassVar (`cls.handler_name
= f"on_{name}"`, `message.py:86`); the bare stem `name` is derived from the message class's
**qualname**, last two `.`-segments, `camel_to_snake`-joined (`message.py:75-85`). Dispatch is **both**
decorated handlers **and** the naming-convention method, always (Part I §5.2).

| Message | `name` | `handler_name` / Handler method | Notes |
|---|---|---|---|
| `events.Load` | `load` | `on_load` | pre-terminal setup |
| `events.Mount` | `mount` | `on_mount` | first setup with visuals |
| `events.Ready` | `ready` | `on_ready` | fires once per run (Part I §1.4) |
| `events.Key` | `key` | `on_key` | also `key_<name>` per-node (Part I §6) |
| `Button.Pressed` | `button_pressed` | `on_button_pressed` | qualname `Button.Pressed` |
| `Worker.StateChanged` | `worker_state_changed` | `on_worker_state_changed` | `namespace="worker"` (worker.py:123) |
| `Markdown.LinkClicked` | `markdown_link_clicked` | `on_markdown_link_clicked` | fires for wikilinks too (Part IV §4.3) |
| custom `@dataclass class Closed(Message)` | `closed` | `on_closed` | override `control` to use `@on(..., "#sel")` |

`@on(MessageType, "#selector")` requires `MessageType.control` to be overridden or it raises
`OnDecoratorError` at import (`_on.py:67-71`).

### 0.3 Binding syntax

```python
BINDINGS: ClassVar[list[BindingType]] = [
    Binding("t", "advance_tick", "Advance Tick", show=False),      # full form
    Binding("ctrl+q", "quit", "Quit", show=False, priority=True),  # priority = checked top-down pre-DOM
    ("ctrl+o", "jump_back"),                                        # 2-tuple: key, action
    ("j,down", "scroll_down", "Down"),                             # comma keys → multiple Bindings
]
```
`BindingType = Binding | tuple[str,str] | tuple[str,str,str]` (`binding.py:23`). Priority vs normal
resolution → **Part I §6.2**. Action resolves to `action_<name>`/`_action_<name>` (Part I §6.3).

### 0.4 `pilot.press(...)` / `App._press_keys` key grammar (app.py:2050-2068)

- `"wait:<ms>"` → `await asyncio.sleep(ms/1000)` + animation-wait. **Only** special token in the engine.
- single non-alnum char → `_character_to_key(c)` (e.g. `"!"` → its key name). `"_"` is **not** a
  pause here — it is an ordinary character key (`"_".isalnum()` is `False` → `_character_to_key`);
  the "short pause" meaning of `"_"` is a snapshot-plugin convention only ([resolved](#verification-ledger)).
- otherwise → `events.Key(key, char)`, each keypress sandwiched by `wait_for_idle(0)` + animation-wait.

### 0.5 Text export (there is **no** `App.export_text()`)

`rg -n export_text` over `textual/` = 0 matches (Part II §10). Whole-screen text capture pattern
(mirror `export_screenshot`, app.py:1855-1878):
```python
console = Console(width=w, height=h, file=io.StringIO(), force_terminal=True,
                  color_system="truecolor", record=True, legacy_windows=False)
console.print(app.screen._compositor.render_update(full=True, screen_stack=app.screen_stack))
text = console.export_text()          # rich/console.py:2192 — style-stripped full compositor text
```
Per-widget, no screenshot machinery: `widget.render_line(y).text` / `widget.render_lines(crop)`
(`strip.py:118-121`, `widget.py:4250-4283`). First-class capture surfaces are **SVG only**:
`export_screenshot` / `save_screenshot` / `deliver_screenshot` / `action_screenshot`.

### 0.6 `@work` decorator forms (_work_decorator.py:74-158)

```python
@work                                   # async worker, own asyncio.Task on the app loop (default)
@work(thread=True)                      # REQUIRED for a non-async def, else WorkerDeclarationError
@work(exclusive=True, group="gather")   # cancels other workers sharing (group, node) before start
@work(exit_on_error=False)              # failure does NOT crash the app; observe via wait()/StateChanged
```
Returns a `Worker[ReturnType]` immediately. Cross-thread → `post_message` / `call_from_thread`
(Part I §5.4). Determinism hazards for a tick driver → **Part I §9.5**.

### 0.7 TCSS `$variables`

`$primary`, `$secondary`, `$accent`, `$background`, `$surface`, `$panel`, `$boost`, `$warning`,
`$error`, `$success`, `$foreground`, plus per-name `-darken-{1,2,3}` / `-lighten-{1,2,3}` / `-muted`,
and `$text`, `$text-muted`, `$text-disabled`, `$border`, `$scrollbar*`, `$link-*`, `$footer-*`,
`$input-cursor-*`, `$block-cursor-*`, `$markdown-h{1-6}-*`, `$button-*`. Full derivation:
`ColorSystem.generate()` (`design.py:105-113`, body ~`280-520`), consumed via `App.get_css_variables()`
(`app.py:1419-1435`). Variables are substituted **before parse** (Part II §3), so a theme change
forces a full `reparse()`. Babylon's KSBC token overrides → **Part II §5**, **Part IV §4.7**.

### 0.8 Determinism one-liners (load-bearing for the sim clock)

- Every tick must hash deterministically; `set_timer`/`set_interval` are **wall-clock, skip-under-load**
  — never the source of sim-time (Part I §8, §9.5).
- Thread-worker cancellation is **cooperative only** — poll `worker.is_cancelled` (Part I §9.4).
- Graph round-trip drops computed fields; systems mutate the shared graph in-place (root `CLAUDE.md`).
- SVG snapshot bytes are deterministic by Rich's Adler-32 id + the plugin's `normalize_svg` (Part III §3.3.4).

---

# Part I — Core model & event system

Source: installed `textual/` tree, every claim `module.py:line`-cited.

## I.1 App lifecycle

### I.1.1 Entry points

| Method | File:line | Nature | Use |
|---|---|---|---|
| `App.run(...)` | `app.py:2308` | sync, blocks | wraps `asyncio.run(run_app())` → `run_async()`; `asyncio.run` unless a legacy `loop=` is passed (pre-3.10 fallback gated by `_ASYNCIO_GET_EVENT_LOOP_IS_DEPRECATED`, `app.py:157,2345-2361`) |
| `App.run_async(headless, inline, inline_no_clear, mouse, size, auto_pilot)` | `app.py:2220` | `async def`, returns `ReturnType \| None` | sets `app._loop = asyncio.get_running_loop()`; installs `asyncio.eager_task_factory` if present (`app.py:2282-2283`); calls `_process_messages`, awaits `auto_pilot_task`, then `asyncio.shield(app._shutdown())` |
| `App.run_test(*, headless=True, size=(80,24), tooltips=False, notifications=False, message_hook=None)` | `app.py:2133` | `async` **context manager**, yields `Pilot[ReturnType]` | launches `_process_messages` as a background task, awaits `app_ready_event`, then `pilot._wait_for_screen()` before yielding; on exit awaits `_shutdown()` and **re-raises `self._exception`** (`app.py:2216-2218`). Full contract → **Part III §3.1** |

`headless` (`App.is_headless`, `app.py:1188`) derives from `self._driver.is_headless`; it is not a
separate `run_test` code path — the same `_process_messages` startup runs regardless, only the
`Driver` differs.

### I.1.2 Startup ordering (`App._process_messages`, app.py:3364-3517)

Exact once-each call sequence:

1. `app_prelude()` (`app.py:3376`) — devtools init, stylesheet reads (`CSS_PATH`, default CSS, inline
   `CSS`), starts the CSS-watcher timer if `watch_css`. Returns `False` (aborting) on any CSS error.
2. `self._running = True` (`app.py:3475`).
3. `events.Load()` dispatched via `_dispatch_message` (`app.py:3477-3478`) **before** the driver
   starts application mode. `Load.__doc__` (`events.py:66-75`): setup "that doesn't require any
   visuals such as loading configuration and binding keys." Handler `on_load`.
4. `driver.start_application_mode()` (`app.py:3489`) — alt screen / raw mode engaged only now.
5. Under `self.batch_update()` (`app.py:3429`):
   a. `events.Compose()` → `App._on_compose` (`app.py:3544`) sets `self._compose_screen = self.screen`
      and mounts `[*screen._nodes, *compose(self)]`. `App.on_event` special-cases `Compose` to first
      `await self._init_mode(self._current_mode)` (`app.py:4063-4065`), which creates/pushes the first
      Screen for the default mode (§I.2.4).
   b. `events.Resize.from_dimensions(self.size, None)` (`app.py:3433-3435`).
   c. `self.stylesheet.apply(self)` (`app.py:3437`).
   d. `events.Mount()` (`app.py:3438`) → `on_mount`.
   e. `self.check_idle()`; `finally:` sets `_mounted_event`/`_is_mounted=True` (`app.py:3441-3442`)
      even on raise.
   f. `Reactive._initialize_object(self)` (`app.py:3444`) — App's reactives initialize **here**, after
      Mount, not in `__init__` (§I.4.3).
   g. If `on_mount` pushed a different screen, `stylesheet.apply(default_screen)` runs again
      (`app.py:3446-3447`).
   h. `await self.animator.start()` (`app.py:3449`).
   i. `finally:` → `_running=True` (idempotent) → `await self._ready()` (`app.py:3457`, override point;
      default logs startup time and, if `SCREENSHOT_DELAY>=0`, schedules an auto-screenshot,
      `app.py:3522-3542`) → `await invoke_ready_callback()` (`app.py:3458`), which unblocks
      `run_test`'s `app_ready_event` / `run_async`'s autopilot.
6. `await self._process_messages_loop()` (`app.py:3461`) — the queue-draining loop starts only now;
   anything `post_message`d during 3–5 sits queued and drains here.

**Net order** (plain subclass with `on_load`/`on_mount`/`on_ready` + a child widget with `on_mount`):
`App.on_load` → (driver starts) → child `Compose`/`Mount` cascade (bottom-up, §I.1.3) →
`App.on_mount` → `App.on_ready` (first frame composited, §I.1.4).

### I.1.3 Mount order is bottom-up (leaf-first)

`App._register` (`app.py:3621-3672`) recurses immediately into a widget's existing `_nodes`
(`app.py:3667-3669`) in a **separate stack frame** with its own `new_widgets`, running that frame's
`apply_stylesheet` + `_start_messages()` to completion **before** returning. `_start_messages`
creates the widget's `asyncio.Task` (`message_pump.py:550-557`) whose `_pre_process`
(`message_pump.py:588-613`) dispatches that widget's `Compose` then `Mount`. Consequence: a
container's children fire `on_mount` before the container itself. A parent's `on_mount` may assume a
child's `Mount` was **dispatched** (task creation is synchronous, deterministic bottom-up) but not
that it **completed** (coroutine completion order is scheduler-dependent).

### I.1.4 `Ready` fires exactly once per run

`events.Ready` (`events.py:206-211`, `on_ready`) is posted by `Screen._on_screen_layout`
(`screen.py:1392-1396`) the first time a screen lays out **and** `app._dom_ready` is still `False`.
`_dom_ready` inits `False` (`app.py:788`), sets `True` once (`screen.py:1396`), is **never reset** —
a true one-shot "first frame on screen" hook, not re-fired on `switch_mode`/`push_screen`. Subsequent
layouts publish `Screen.screen_layout_refresh_signal` instead.

### I.1.5 Shutdown

Message-loop `finally` (`app.py:3464-3470`): `workers.cancel_all()` → `_running=False` →
`animator.stop()` → `Timer._stop_all(self._timers)`. `App.exit(result=None, return_code=0,
message=None)` (`app.py:1270-1288`) does **not** tear down synchronously — it sets `_exit=True`,
records `_return_value`/`_return_code`, and **posts** `messages.ExitApp()`; teardown happens when
that message is processed.

## I.2 Screen, ModalScreen, screen stack, MODES

### I.2.1 `Screen` (screen.py:148, `Generic[ScreenResultType]`, subclasses `Widget`)

| Name | Kind | Default | Note |
|---|---|---|---|
| `AUTO_FOCUS` | `ClassVar[str\|None]` | `None` | `None` ⇒ inherit `App.AUTO_FOCUS` (defaults `"*"`, app.py:396-401); `""` disables |
| `focused` | `Reactive[Widget\|None]` | `None` | mutate only via `set_focus()` (screen.py:229-231) |
| `title`/`sub_title` | `Reactive[str\|None]` | `None` | override `App.title`/`sub_title` when non-`None` |
| `maximized` | `Reactive[Widget\|None]`, `layout=True` | `None` | maximize/minimize state |
| `BINDINGS` | list | `tab`→`app.focus_next`, `shift+tab`→`app.focus_previous`, `ctrl+c,super+c`→`screen.copy_text` (`show=False`) | screen.py:269-273 |

Key properties: `is_modal` (`screen.py:336-338`, returns `_modal`), `is_active` (`screen.py:557-562`,
`app.screen is self` — top of the current mode's stack), `is_current` (`screen.py:341-348`,
top-or-in-`_background_screens` — true for a screen partially visible under a translucent modal).
`_background_screens` (`app.py:1644-1652`) walks the stack top-down, stopping at the first fully-opaque
screen — how translucent `ModalScreen`s (60% bg, `screen.py:2164-2172`) let underlying screens keep
rendering.

### I.2.2 `ModalScreen` / `SystemModalScreen` (screen.py:2157-2195)

`ModalScreen.__init__` sets `self._modal = True`; there are no special methods — "takes precedence"
comes entirely from `is_modal=True` being consulted by `Screen._modal_binding_chain` (§I.6.2) and by
`App.children` (`app.py:993-1012`, which skips `SystemModalScreen`s in the visible-child report).
`SystemModalScreen(ModalScreen, inherit_css=False)` is isolated from app CSS. `CommandPalette`
(`command.py:532`) is a `SystemModalScreen[None]`.

### I.2.3 Screen stack API (all on `App`, operate on `self._screen_stacks[mode]`)

| Method | Signature | File:line | Semantics |
|---|---|---|---|
| `push_screen` | `(screen: Screen\|str, callback=None, wait_for_dismiss=False, *, mode=None) -> AwaitMount \| Future` | `app.py:2895` | suspends top (`ScreenSuspend`+`refresh()`), appends, posts `ScreenResume` if now active. `wait_for_dismiss=True` **requires an active worker** — else `NoActiveWorker` (`app.py:2960-2964`); returns a `Future` resolved by the pushed screen's `dismiss()` |
| `push_screen_wait` | `async (screen, *, mode=None) -> ScreenResultType` | `app.py:2981` | `await asyncio.shield(push_screen(..., wait_for_dismiss=True))`; worker-only |
| `pop_screen` | `() -> AwaitComplete` | `app.py:3096` | raises `ScreenStackError` if ≤1 screen; pops, posts `ScreenResume` to new top, `_replace_screen` removes it from DOM unless still installed / on another mode's stack |
| `switch_screen` | `(screen: Screen\|str) -> AwaitComplete` | `app.py:3001` | replaces the **top** in place; no-op if already current |
| `install_screen` | `(screen: Screen, name: str) -> None` | `app.py:3036` | registers under a name so it survives popping; `ScreenError` on duplicate |
| `uninstall_screen` | `(screen: Screen\|str) -> str\|None` | `app.py:3062` | `ScreenStackError` if still on **any** mode's stack |
| `get_screen` | `(screen: Screen\|str, screen_class=None) -> Screen` | `app.py:2761` | resolves a name→instance (instantiates a factory once, caches into `_installed_screens`) |
| `is_screen_installed` | `(screen: Screen\|str) -> bool` | `app.py:2731` | — |

`dismiss(result=None)` is a **`Screen`** method (`screen.py:2048-2081`), not on `App`: invokes the
most recent `ResultCallback` (LIFO, `_result_callbacks[-1]`, pushed at push-time `screen.py:1285`)
then `app.pop_screen()`. **`ScreenError` if you `await` a `dismiss()` from inside a message handler
on the screen being dismissed** — call it without `await` in that position (`screen.py:2053-2079`,
enforced by a `pre_await` check on `active_message_pump.get() is self`).

### I.2.4 MODES (App.MODES, app.py:361-390)

```python
MODES: ClassVar[dict[str, str | Callable[[], Screen]]] = {}
DEFAULT_MODE: ClassVar[str] = "_default"
```

Each mode maps to a **base screen** (bottom of that mode's own independent stack) — a `SCREENS`
name, a `Screen` subclass/callable, but **never an instance** (`__init_subclass__` raises `ValueError`
at class-definition time, `app.py:912-928`). `__init__` seeds `_screen_stacks = {DEFAULT_MODE: []}`
(`app.py:638`) and `_modes = self.MODES.copy()` (`app.py:753`) — per-instance copies, so
`add_mode`/`remove_mode` never mutate the class dict.

| Method | File:line | Notes |
|---|---|---|
| `switch_mode(mode) -> AwaitMount` | `app.py:2630` | no-op if already there; `UnknownModeError` if unknown. Suspends current (`ScreenSuspend`), lazily `_init_mode(mode)` on first empty stack, sets `_current_mode`, publishes `mode_change_signal`/`screen_change_signal`, resumes new top |
| `_init_mode(mode) -> AwaitMount` | `app.py:2584` | no-op if the mode already has a stack; else instantiates the registered base screen (`TypeError` on a smuggled instance); if the mode is unknown to `MODES` (incl. `DEFAULT_MODE` on first boot) falls back to `get_default_screen()` (`app.py:1380-1391`, default `Screen(id="_default")`) |
| `add_mode(mode, base_screen)` | `app.py:2677` | `InvalidModeError` for `"_default"`/duplicate; `TypeError` for an instance |
| `remove_mode(mode) -> AwaitComplete` | `app.py:2699` | `ActiveModeError` if current; dismisses every screen on that stack |

`App.screen` (`app.py:1627-1641`) is always `self._screen_stack[-1]` for the **current** mode;
`App.screen_stack` (`app.py:1206-1212`) returns a **copy** (`_screen_stack`, singular, is the live
reference). The Babylon lobby→briefing→campaign shell uses exactly this: named `SCREENS`, same-mode
transitions via `push_screen`/`switch_screen`, independent stacks via `switch_mode` (Part IV §4.1).

## I.3 `compose()`, `mount()`, the DOM

### I.3.1 `compose()` contract

`App.compose(self) -> ComposeResult` (`app.py:1393-1398`, default `yield from ()`); identical on
`Widget`. `ComposeResult = Iterable[Widget]` (`app.py:159`). Shared engine:
`textual.compose.compose(node, compose_result=None)` (`compose.py:12-99`), used for both the initial
`Compose` handler and `recompose()`.

- Validates each child `isinstance(child, Widget)` with a real `.id`; on failure raises a `MountError`
  **thrown back into the generator** via `.throw()` so the traceback points at your `yield`
  (`compose.py:58-76`).
- `with SomeContainer():` blocks push the container onto `app._compose_stacks[-1]`
  (`app.py:756-757`) and route each yielded widget through `compose_stack[-1].compose_add_child(child)`
  (`compose.py:81-90`, `dom.py:393`/`widget.py:894`) — nesting children under the container.

### I.3.2 Mounting

`App.mount(*widgets, before=None, after=None) -> AwaitMount` (`app.py:2525`) and `Widget.mount(...)`
(`widget.py:1424`) bottom out in `App._register` (§I.1.3). `before`/`after` accept an `int` index or a
`query_one`-style `str`; both together raise `AppError` (`app.py:3593-3594`). `mount_all(...)`
(`app.py:2555`) is sugar. `recompose()` (`app.py:3562-3574`, `widget.py:1704`) removes all children
(`query("*").exclude(".-textual-system").remove()` at App level) and re-runs `compose()` inside
`async with self.screen.batch():`. A `reactive(..., recompose=True)` change defers via
`_recompose_required=True; call_next(_check_recompose)` (`widget.py:4358-4360`) — next tick, not
synchronous. Refresh cost comparison → **Part II §8**.

### I.3.3 Query API (DOMNode, dom.py)

| Method | Signature | File:line | Behavior |
|---|---|---|---|
| `query` | `(selector=None) -> DOMQuery` | `dom.py:1397` | deep, rooted at `_get_dom_base()` (App → `default_screen`; else `self`) |
| `query_children` | same, `deep=False` | `dom.py:1425` | immediate children only |
| `query_one` | `(selector, expect_type=None) -> Widget` | `dom.py:1462` | `NoMatches`/`WrongType`. ID/simple selectors **cached** on `(base._nodes._updates, selector, expect_type)`; complex selectors not (`dom.py:1519-1520`) |
| `query_one_optional` | same, returns `None` on no match | `dom.py:1548` | still raises `WrongType` |
| `query_exactly_one` | as `query_one`, `TooManyMatches` if >1 | `dom.py:1585` | — |

## I.4 Reactive attributes (reactive.py)

### I.4.1 The `Reactive` descriptor — full parameter table (reactive.py:142-164)

| Param | `Reactive` | `reactive` | `var` | Effect |
|---|---|---|---|---|
| `default` | *required* | *required* | *required* | value, 0-arg callable, or `Initialize(fn)` (lazy `fn(obj)` first access, reactive.py:59-79,210-223) |
| `layout` | `False` | `False` | `False` forced | on change `refresh(layout=True)` |
| `repaint` | `True` | `True` | `False` forced | on change `refresh(repaint=True)` |
| `init` | `False` | `True` | `True` | run watchers immediately at first init (after mount) rather than staying silent until first explicit set (reactive.py:230-239) |
| `always_update` | `False` | `False` | `False` | run watchers even if new `==` old |
| `compute` | `True` | `True` | *(inherits `True`)* | after set, re-run `compute_<name>`/`_compute_<name>` |
| `recompose` | `False` | `False` | *(base `False`)* | on change, full `recompose()` (deferred, §I.3.2) |
| `bindings` | `False` | `False` | present | on change, `refresh_bindings()` |
| `toggle_class` | `None` | `None` | present | space-separated class(es) toggled by truthiness, at init and every set |

`reactive` (`reactive.py:437-472`) and `var` (`reactive.py:475-502`) are thin `Reactive` subclasses
that only change defaults. `var`'s sole documented difference is "no auto-refresh" (`layout=False,
repaint=False` forced); it does **not** disable `compute` (it never passes `compute=` through, so
inherits base `True`, reactive.py:486-502).

### I.4.2 Method-name resolution — not symmetric across validate/watch/compute

Verified in `Reactive._set` (`reactive.py:316-369`) and `_check_watchers` (`reactive.py:376-410`):

| Hook | Private | Public | Both present |
|---|---|---|---|
| validate | `_validate_<name>` | `validate_<name>` | **both run, chained** — private output feeds public input (reactive.py:338-343) |
| watch | `_watch_<name>` | `watch_<name>` | **both run** (reactive.py:390-396) |
| compute | `_compute_<name>` | `compute_<name>` | **mutually exclusive** — `__set_name__` picks private else public (reactive.py:251-258); a class defining both **and** the reactive raises `TooManyComputesError` at class-body eval (message_pump.py:96-109) |

Module-level `_watch(node, obj, attr, callback, init=True)` (`reactive.py:505-533`, what
cross-object `Widget.watch(other, "value", cb)` bottoms out in) stores in a separate `obj.__watchers`
dict-of-lists, **also** invoked alongside `_watch_`/`watch_` (`reactive.py:398-410`), each wrapped in
`reactable.prevent(*obj._prevent_message_types_stack[-1])`.

### I.4.3 Initialization & change flow

- `_initialize_object(obj)` (`reactive.py:230-239`) iterates `obj._reactives`, calling
  `_initialize_reactive` for each — invoked once from App startup (`app.py:3444`) and the widget mount
  path; **not** automatic from `__init__`.
- `_initialize_reactive` (`reactive.py:196-228`) is idempotent (`if hasattr(obj, internal_name):
  return`); `__get__` also lazily inits on first read (`reactive.py:303-304`).
- `__set__` → `_set` (`reactive.py:316-369`): validate (chained) → toggle-class → **only if** `always
  or _always_update or current != value`: store, watchers, compute (if `_run_compute`),
  `refresh_bindings()` (if `bindings`), `refresh(repaint=, layout=, recompose=)` (if any set).
- A watcher/validator may return an awaitable; `invoke_watcher` (`reactive.py:90-121`) schedules it via
  `call_next(partial(await_watcher, ...))` rather than awaiting inline — async watchers don't block the
  `__set__`; effects land next tick. `await_watcher` (`reactive.py:82-87`) re-runs `_compute(obj)`
  after the awaited watcher.
- `_reset_object`/`_clear_watchers` (`reactive.py:242-249,179-188`) run from
  `MessagePump._close_messages`/`_process_messages` `finally` (message_pump.py:536,580) to break cycles.

Reactive-driven refresh is the normal repaint path (default `repaint=True`) — see **Part II §8**.

## I.5 The message class hierarchy

### I.5.1 `Message` (message.py:23) vs `Event` (events.py:39)

`Event(Message)` adds no fields — a marker so `_dispatch_message` routes `Event`s through the
overridable `on_event` hook first (`message_pump.py:707-741`, both paths converge on `_on_message`;
`on_event` at `message_pump.py:802-808` just calls `_on_message`). Built-in input events (`Key`,
`MouseEvent`, …) pass through `on_event`; widget `Message`s (`Button.Pressed`) skip it.

`Message.__slots__` (`message.py:26-33`): `_sender, time, _forwarded, _no_default_action,
_stop_propagation, _prevent`. Class config via `__init_subclass__` kwargs (`message.py:62-86`):

```python
class Foo(Message, bubble=True, verbose=False, no_dispatch=False, namespace=None): ...
```

| ClassVar | Meaning |
|---|---|
| `bubble` | if `True` (default), an unhandled/unstopped message propagates to `_parent` after local dispatch (message_pump.py:833-839) |
| `verbose` | excluded from `textual console` log unless `-v` |
| `no_dispatch` | dropped at `_dispatch_message`, reaches no handler (message_pump.py:713-715) |
| `namespace` | overrides `handler_name` derivation |
| `handler_name` | computed, not settable — the `on_<...>` string looked up |

`handler_name` (`message.py:75-86`): with `namespace`, `f"{namespace}_{camel_to_snake(cls.__name__)}"`;
else last two `.`-segments of the **qualified name** (`Button.Pressed` → `button_pressed` →
`on_button_pressed`; top-level `Mount` → `on_mount`). `Message.control` (`message.py:88-91`) defaults
`None`; override as a `@property` to expose the widget the message is about, else `@on(YourMessage,
"#foo")` raises `OnDecoratorError` (`_on.py:67-71`).

### I.5.2 `@on` vs `on_<namespace>_<name>` — dispatch is both, always

`@on(message_type, selector=None, **kwargs)` (`_on.py:24-93`) stashes `(message_type, selectors)` on
`method._textual_on` — it does not rename the method. Collection into per-class
`_decorated_handlers` happens in `_MessagePumpMeta` (`message_pump.py:70-112`).

Dispatch, `_get_dispatch_methods(method_name, message)` (`message_pump.py:743-800`), for **every**
`cls` in `self.__class__.__mro__` (most-derived first):

1. **Decorated handlers first**: for each ancestor of the *message's* runtime type
   (`message.__class__.__mro__`), read `cls.__dict__["_decorated_handlers"]` (direct dict, not
   inherited) and yield the bound method if selectors empty/all match (`match(selector,
   getattr(message, attr))`; attr must be a `Widget` else `OnNoWidget`, message_pump.py:783-786).
2. **Naming fallback**: `cls.__dict__.get(f"_{method_name}") or cls.__dict__.get(method_name)`
   (`message_pump.py:796-798`) — the **private** `_on_mount` wins over `on_mount` on the same class —
   only if not already decorated (`not getattr(method, "_textual_on", None)`).

**Consequence:** because the loop walks `cls.__dict__` for every MRO class rather than normal
single-dispatch, **if a base and a subclass both define `on_click` as plain methods, BOTH run** —
most-derived to least, `await invoke(method, message)` for each (`message_pump.py:810-831`). A
subclass handler does **not** implicitly shadow a base handler of the same name. `methods_dispatched`
(a `set`) only dedupes identical inherited method objects reached twice, not distinct functions from
different classes.

### I.5.3 Bubbling, `prevent()`, `stop()`, `prevent_default()`

- `message.stop(stop=True)` (`message.py:142-149`) sets `_stop_propagation`, checked in the bubble
  step: `if message.bubble and self._parent and not message._stop_propagation:` (message_pump.py:834).
- `message.prevent_default(prevent=True)` (`message.py:131-140`) sets `_no_default_action`; inside
  `_get_dispatch_methods` `if message._no_default_action: break` **stops the whole per-class MRO loop**
  (`message_pump.py:759-760`) — suppresses all remaining handlers for this dispatch, not just a "soft"
  default. `_bubble_to` (`message.py:151-158`) resets `_no_default_action=False` before re-posting, so
  it suppresses only at the current node.
- `MessagePump.prevent(*message_types)` (`message_pump.py:198-217`) is a **context manager**, pushing
  a `ContextVar` stack (`_prevent_message_types_stack`); messages of those types posted inside the
  `with` are marked (`message._prevent`, unioned at post-time message_pump.py:879-881) and dropped.
  `_dispatch_message` wraps dispatch in `with self.prevent(*message._prevent):` (message_pump.py:724),
  re-establishing the sender's preventions as the receiver's ambient preventions — how "prevent
  `Input.Changed` while I mutate `.value`" suppresses handlers down the bubble chain too.
- **Sender==parent halt** (`message_pump.py:834-839`): if `message._sender is self._parent`, bubbling
  calls `message.stop()` after bubbling to that parent once — prevents infinite re-post bounce.

### I.5.4 Custom `Message` + `post_message` from a thread

Idiomatic (e.g. `command.py:689-698`):
```python
@dataclass
class Closed(Message):
    option_selected: bool
```
Dataclass fields work because `Message.__post_init__` (`message.py:47-57`) is named so "to allow
dataclasses to initialize the object."

`MessagePump.post_message(message) -> bool` (`message_pump.py:860-888`) is the thread-safe primitive:
```python
if self._thread_id != threading.get_ident() and self.app._loop is not None:
    self.app._loop.call_soon_threadsafe(self._message_queue.put_nowait, message)
else:
    self._message_queue.put_nowait(message)
```
Calling `.post_message(...)` from any OS thread is safe with no ceremony — Textual detects the foreign
thread id and marshals via `call_soon_threadsafe`. This is the **recommended** cross-thread path
(the `call_from_thread` docstring points back to it, `app.py:1801-1803`). Directly mutating a reactive
or calling most APIs from a foreign thread is **not** safe (`app.py:1796-1797`); use `post_message` or
`App.call_from_thread` (`app.py:1788-1838`, via `asyncio.run_coroutine_threadsafe` + blocking
`future.result()`).

## I.6 Key handling chain

### I.6.1 `Binding` (binding.py:54-168, frozen dataclass)

Fields: `key, action, description="", show=True, key_display=None, priority=False, tooltip="",
id=None, system=False, group=None`. `BINDINGS: ClassVar[list[BindingType]]` where `BindingType =
Binding | tuple[str,str] | tuple[str,str,str]` (`binding.py:23`). `Binding.make_bindings`
(`binding.py:121-168`) expands comma keys (`"j,down"`) and single chars via `_character_to_key`.
`BindingsMap` (`binding.py:184-393`) is the per-node runtime container (`key_to_bindings: dict[str,
list[Binding]]`); `apply_keymap(Keymap)` supports user remaps via `App.set_keymap`/`update_keymap`
(`app.py:4011-4038`) with clash detection.

### I.6.2 Priority vs normal bindings — two passes over two chains

`App._check_bindings(key, priority=False) -> bool` (`app.py:3966-3988`):
```python
for namespace, bindings in (
    reversed(self.screen._binding_chain) if priority else self.screen._modal_binding_chain
):
    for binding in bindings.key_to_bindings.get(key, ()):
        if binding.priority == priority:
            if await self.run_action(binding.action, namespace):
                return True
return False
```
`Screen._binding_chain` (`screen.py:407-446`) = `[focused, *focused.ancestors_with_self]` paired with
each node's `BindingsMap.copy()` if focused, else `[(screen,…),(app,…)]`. `_modal_binding_chain`
(`screen.py:449-455`) is `_binding_chain` **truncated at and including the first modal node** — never
reaches App-level bindings when a `ModalScreen` is in the chain.

Full flow at `App.on_event` for `events.Key` (`app.py:4121-4138`):

1. If a widget is maximized and `key=="escape"` and `escape_to_minimize` → minimize, **return**
   (`app.py:4124-4130`).
2. **Priority pass**: `_check_bindings(event.key, priority=True)` walks `reversed(_binding_chain)`
   (**App→focused**), only `priority=True` bindings (default `ctrl+q`→quit, app.py:454-464). If handled,
   stop — the raw key never reaches the DOM.
3. Else **forward** the raw `Key`: `(self.focused or self.screen)._forward_event(event)`
   (`app.py:4137-4138`) — injects into DOM dispatch at the focused widget.
4. At every ancestor the event bubbles (`Key(InputEvent)` bubbles, events.py:261), `Widget._on_key`
   (`widget.py:4705-4709`) → `dispatch_key` (`_dispatch_key.py:12-63`) tries
   `key_<name>`/`_key_<name>` **on that node only**, using `event.name_aliases`; raises
   `DuplicateKeyHandlers` if >1 alias resolves to distinct handlers. None of this stops propagation.
5. Reaching `App`, `App._on_key` (`app.py:4341-4343`) runs the **normal** pass: `if not (await
   _check_bindings(event.key)): await dispatch_key(self, event)` — `priority=False`, walking
   `_modal_binding_chain` focused→app, matching `priority==False` entries; then App's own
   `key_<name>`/`_key_<name>`.

**Summary:** priority bindings — checked once, top-down, before the key enters the DOM. Normal
`BINDINGS` — checked once at App level after the key bubbled all the way up (every `key_<name>` got
first refusal), walking focused→app, stopping at the first modal boundary. Closest-to-focused entries
win inside a single `_check_bindings` call — ordering, not bubbling.

### I.6.3 Actions: `check_action` / `action_<name>` / `_action_<name>`

`App.run_action(action, default_namespace=None, namespaces=None) -> bool` (`app.py:4223-4248`): parse
via `textual.actions.parse` (`actions.py:26-59` — `"namespace.method(args)"`, regex +
`ast.literal_eval`, `@lru_cache(1024)`), resolve namespace via `_parse_action` (`app.py:4167-4204`;
namespace ∈ `_action_targets = {"app","screen","focused"}`, app.py:652), call
`action_target.check_action(name, params)` (`dom.py:1909-1923`; default `True`; return `False` to
hide+disable, `None` to grey-out-but-show), then `_dispatch_action` (`app.py:4250-4289`):
```python
private = getattr(namespace, f"_action_{name}", None)
if callable(private): await invoke(private, *params); return True
public = getattr(namespace, f"action_{name}", None)
if callable(public): await invoke(public, *params); return True
```
Private-before-public, but a plain `getattr`/`or` fallback (not per-MRO, not "both run"). Raising
`actions.SkipAction` inside an action method means "not handled, fall through" (`app.py:4285-4287`) —
the only documented way an `action_*` declines after being called.

### I.6.4 Focus chain and focus events

`Screen.focus_chain` (`screen.py:772-826`) — DFS over `displayed_children`, sorted by
`_focus_sort_key`, honoring `trap_focus` and inherited `visibility`. `Screen.set_focus` posts
non-bubbling `Blur()`/`Focus(from_app_focus=…)` (`screen.py:1122,1130,1135`). `Widget._on_focus`/
`_on_blur` (`widget.py:4765-4775`) **always** post a **bubbling** `DescendantFocus(self)` /
`DescendantBlur(self)` to `self.parent` — the supported way for a container to react to any descendant
focus change without handling `Focus`/`Blur` on every descendant. `App.action_focus_next`/
`action_focus_previous` (`app.py:4554-4561`) drive `Screen._move_focus(direction, selector)`
(`screen.py:828-891`).

## I.7 Command palette (command.py)

| Class | Role |
|---|---|
| `Provider` (ABC, `command.py:179-326`) | one instance per palette invocation (`provider_class(calling_screen, match_style)`); implement `async def search(query) -> Hits`; optional `discover()`/`startup()`/`shutdown()`. `self.app`/`self.screen`/`self.focused` resolve against the screen active at invocation (command.py:201-217) |
| `Hit` (`command.py:72-116`) | `score: float` (0–1), `match_display`, `command`, optional `text`/`help` |
| `DiscoveryHit` (`command.py:120-169`) | always `score==0.0`; provider-yield order |
| `SimpleProvider` (`command.py:344-397`) | wraps a flat `list[(name, callback, help?)]` |
| `CommandPalette(SystemModalScreen[None])` (`command.py:532`) | the UI; `App.COMMAND_PALETTE_BINDING="ctrl+p"` (app.py:441), auto-registered `priority=True`→`action_command_palette` in `__init__` if unbound and `ENABLE_COMMAND_PALETTE` (app.py:874-889) |

Command gathering is a **worker** — `_gather_commands` is `@work(exclusive=True,
group=_GATHER_COMMANDS_GROUP)` (`command.py:1049-1053`): every keystroke cancels the in-flight gather
and starts fresh (exclusive + `cancel_group`, `command.py:1173-1187`), fanning one `asyncio.Task` per
`Provider._search` (`command.py:936-1021`) into a shared queue, batching UI every `_RESULT_BATCH_TIME`
(0.25s). `App.get_system_commands(screen)` (`app.py:1318-1379`, override point) supplies
Theme/Quit/Keys/Maximize/Screenshot via `SystemCommandsProvider` in `App.COMMANDS` (app.py:433-439).

## I.8 Timers — `set_timer` / `set_interval` (a real scheduling asymmetry)

Both are `MessagePump` methods (on App/Screen/Widget):
```python
set_timer(delay, callback=None, *, name=None, pause=False) -> Timer            # message_pump.py:378
set_interval(interval, callback=None, *, name=None, repeat=0, pause=False) -> Timer  # message_pump.py:418
```
**Load-bearing wiring difference** (`message_pump.py:406-413` vs `439-446`):
```python
# set_timer:    callback=None if callback is None else partial(self.call_next, callback)
# set_interval: callback=callback
```
`Timer._tick` (`timer.py:180-203`) does `await invoke(self._callback)` if present, else posts
`events.Timer` to `self.target`. Because `set_timer` wraps in `call_next`, its callback runs only when
the owning pump next flushes `_next_callbacks` (`_flush_next_callbacks`, message_pump.py:695-705, after
every dispatch and each `Idle`). **`set_interval`'s callback is invoked directly in the `Timer`'s own
`asyncio.Task`** (`Timer._run`/`_start`, timer.py:89-91) the instant the interval elapses — it does
not funnel through the target's queue.

Both tasks share the **single-threaded** asyncio loop (no data race), but ordering differs: a
`set_timer` callback is serialized behind the current message + prior `_next_callbacks`; a
`set_interval` callback preempts at its own task's next scheduling point. **Neither is
wall-clock-deterministic**: `Timer._run` (`timer.py:150-178`) uses `_time.get_time()`/`_time.sleep`
with `skip=True` default that **drops missed ticks** when the clock runs ahead
(`next_timer < now` ⇒ fast-forward `count`, timer.py:162-164). **For a paced/deterministic tick driver
(Babylon `SimulationEngine.run_tick`), neither timer should be the source of sim-time** — model the
sim clock as its own counter, advanced from whichever primitive drives it, never inferred from
elapsed wall-clock. `Timer.pause()`/`resume()`/`reset()`/`stop()` (`timer.py:127-141,93-100`) gate an
internal `asyncio.Event` (`_active`) — pause/resume do not cancel the task (a paused interval timer's
task stays alive, blocked on `await self._active.wait()`).

## I.9 `@work` — workers: thread vs async, exclusivity, lifecycle, determinism

### I.9.1 Declaration (_work_decorator.py:74-158)

```python
@work(name="", group="default", exit_on_error=True, exclusive=False, description=None, thread=False)
def method(self, ...): ...
```
A non-`async def` **MUST** pass `thread=True` — else `WorkerDeclarationError` at decoration
(`_work_decorator.py:110-115`). Calling a decorated method returns a `Worker[ReturnType]` immediately
(non-blocking); it calls `self.run_worker(partial(method, *a, **kw), ...)` (`dom.py:496-542`).

### I.9.2 `run_worker` → `WorkerManager` → `Worker`

`run_worker` (`dom.py:496-542`) detects a foreign-thread call (`app._thread_id != get_ident()`) and
marshals worker **creation** through `app.call_from_thread(workers._new_worker, ...)` — even starting
a worker from another worker's thread is thread-safe. `WorkerManager.add_worker(worker, start=True,
exclusive=True)` (`worker_manager.py:65-79`): if `exclusive` (the `add_worker` default `True`, distinct
from `@work`'s own `exclusive=False` threaded through `run_worker`'s param, dom.py:504,539 — so
**`@work`'s default is non-exclusive**), it first `cancel_group(worker.node, worker.group)` —
cancelling workers sharing **both** `group` **and** originating `node` (groups are node-scoped,
worker_manager.py:139-156).

### I.9.3 Thread vs async — genuinely different runtimes

`Worker.run()` (`worker.py:346-356`): `await (self._run_threaded() if self._thread_worker else
self._run_async())`.

| | Async (`thread=False`, default) | Thread (`thread=True`) |
|---|---|---|
| Runs on | `asyncio.create_task(self._run(app))` (worker.py:389-401) — ordinary Task on the app loop, same thread | `loop.run_in_executor(None, runner, self._work)` (worker.py:324-326) — default `ThreadPoolExecutor`, separate OS thread |
| Event loop | same loop, interleaved at `await`s | **brand-new loop** via `asyncio.run(do_work())` in the thread (worker.py:294-304) |
| Touch Textual state directly? | Yes (same thread) but **interleaved**, not serialized through the queue (§I.9.5) | **No** — must use `post_message`/`call_from_thread` (§I.5.4); direct writes are a data race |
| `get_current_worker()` | via `active_worker` ContextVar set in `Worker._run` under `app._context()` (worker.py:358-370) | same ContextVar, set inside the thread's own context (worker.py:291-309); contextvars propagate per-thread |

A non-coroutine passed `thread=False` raises `WorkerError("Request to run a non-async function as an
async worker")` at run time (`worker.py:342-344`) — runtime mirror of the decoration-time error.

### I.9.4 Lifecycle, state, cancellation

`WorkerState` (`worker.py:82-95`): `PENDING → RUNNING → {CANCELLED, ERROR, SUCCESS}`.
`Worker.StateChanged` (`worker.py:123-139`, `Message`, `bubble=False`, `namespace="worker"` ⇒
`on_worker_state_changed`) is posted to the owning node on **every** transition, including the initial
`PENDING` in `__init__` (worker.py:183) — the supported no-poll progress hook.

`Worker.cancel()` (`worker.py:416-421`): sets `_cancelled=True`, `self._task.cancel()` (delivers
`CancelledError` at the next `await` in the task), sets `cancelled_event` (a `threading.Event`).
**For a thread worker, `_task` wraps `run_in_executor`'s future — cancelling it cannot interrupt a
blocking call already running in the OS thread.** `is_cancelled`/`cancelled_event` exist so a
thread-worker body **cooperatively** polls and exits (pattern: `worker = get_current_worker(); if
worker.is_cancelled: break`, command.py:1084,1135-1136). A thread worker that ignores them and does a
long blocking call (`time.sleep`, blocking socket read) **runs to completion in its OS thread** after
`.cancel()` — a determinism/resource-leak hazard for any threaded tick driver.

`Worker.wait() -> ResultType` (`worker.py:423-455`): raises `DeadlockError` if called from inside the
worker's own task (`active_worker.get() is self`, worker.py:433-437); raises `WorkerFailed`/
`WorkerCancelled` to surface terminal state rather than returning `None`.

`exit_on_error: bool = True` (constructor default, worker.py:149,160,170): an uncaught exception in
`Worker._run` (worker.py:358-387) with `exit_on_error` wraps as `WorkerFailed` and calls
`app._handle_exception(...)` — **crashes the whole app**. Set `exit_on_error=False` to suppress and
observe via `Worker.error`/`StateChanged`/`await worker.wait()` raising `WorkerFailed`.

### I.9.5 Determinism summary for a paced tick driver

| Mechanism | Thread | Loop | Serialized through queue? | Wall-clock-driven? |
|---|---|---|---|---|
| `set_timer(delay, cb)` | main | main | Yes — via `call_next`/`_next_callbacks` (message_pump.py:406-413,695-705) | Yes (timer.py:150-178) |
| `set_interval(interval, cb)` | main | main | **No** — direct from the Timer's Task (message_pump.py:439-446) | Yes, **skips missed ticks** (skip=True, timer.py:162-164) |
| `@work` async | main | main (own Task) | No — interleaved at `await`s | No (unless the work awaits something wall-clock) |
| `@work(thread=True)` | **separate OS thread** | **separate, fresh** (worker.py:294-298) | No — touch state only via `post_message`/`call_from_thread` | No, cancellation **cooperative-only** (§I.9.4) |

None of the four provides a strictly-ordered, tick-exact primitive; a deterministic sim clock must be
its own counter/state, advanced from whichever drives it, never inferred from elapsed time or from
`Timer`'s tick-skip.

---

# Part II — Styling and rendering

## II.1 TCSS loading — `CSS_PATH` / `CSS` / `DEFAULT_CSS` / `SCOPED_CSS`

Four class-var sources feed one `Stylesheet` (`css/stylesheet.py`):

| Class var | Declared on | Scope | `is_default_css` |
|---|---|---|---|
| `DEFAULT_CSS: ClassVar[str]` | `DOMNode` (dom.py:138), per-widget | auto-scoped to the type unless `SCOPED_CSS=False` | `True` |
| `SCOPED_CSS: ClassVar[bool] = True` | `DOMNode` (dom.py:154) | toggles the above | N/A |
| `CSS_PATH: ClassVar[CSSPathType\|None]` | `App` (app.py:410), `Screen` (screen.py:165) | file(s) via `Stylesheet.read_all` | `False` (user) |
| `CSS: ClassVar[str] = ""` | `App` (app.py:299) | inline string | `False` (user) |

**Load order** (`app_prelude`, app.py:3388-3407): 1) `read_all(css_path)` (list order) → 2) every
mounted class's `DEFAULT_CSS` via `_get_default_css()` walking `_css_bases()` (MRO, base-first,
dom.py:707-745; also re-triggered per instance in `Widget._post_register`, widget.py:1721-1733) →
3) `add_source(self.CSS, is_default_css=False)` **last**.

**Precedence is not pure specificity.** `Styles.extract_rules()` (`css/styles.py:980-1009`) builds
`Specificity6 = (0 if is_default_rules else 1, 1 if important else 0, *specificity3, tie_breaker)` =
`(user_css_flag, important_flag, id, class, type, tie_breaker)`. Any `is_default_css=True` rule
(widget `DEFAULT_CSS`) is **strictly outranked** by any user rule — a bare `Widget { color: red; }` in
`App.CSS` beats `#my-id { color: blue; }` in a widget's `DEFAULT_CSS`. This is "component defaults are
a floor, not a ceiling."

**`CSS` beats `CSS_PATH` on a true tie** (docstring app.py:300-301): `Stylesheet.apply()`
(`css/stylesheet.py:470+`) does `rules = list(filter(limit_rules.__contains__, reversed(self.rules)))`
— reversed insertion order (CSS_PATH→DEFAULT_CSS→CSS) processes `CSS` first, and `max(...)` keeps the
first element on an equal key, so `CSS` wins identical-specificity ties. [Verified by reading `apply()`
and `extract_rules()`; only the outcome is documented.]

**Live reload**: `watch_css=True` / `textual run --dev` starts `FileMonitor(css_path, _on_css_change)`
(app.py:783); `_on_css_change` (app.py:2364) re-reads `CSS_PATH` into a stylesheet **copy**, swaps only
on success — a bad edit keeps the last-good stylesheet and logs+bells.

**Per-Screen CSS**: `Screen.CSS_PATH`/`CSS` register when the screen is first pushed
(app.py:2826-2849), not at boot — `css_monitor.add_paths(...)` + `stylesheet.reparse()` per push.

**Babylon usage**: `repo:src/babylon/tui/app.py`'s `ArchiveApp` uses only inline `CSS`; widgets use
`DEFAULT_CSS` — so `ArchiveApp.CSS` always wins over equal/lower-specificity widget `DEFAULT_CSS`.

## II.2 Selectors and specificity

`css/model.py` selector AST: `SelectorType` = `UNIVERSAL(*)`, `TYPE`, `CLASS(.foo)`, `ID(#foo)`,
`NESTED(&)`. `SELECTOR_MAP` (`css/parse.py:31-40`) base `Specificity3=(id,class,type)`:
`type→(0,0,1)`, `class→(0,1,0)`, `id→(1,0,0)`, `universal→(0,0,0)`. A pseudo-class (`:hover`,`:focus`)
adds `+1` to the **class** slot (`Selector._add_pseudo_class`, model.py:151-153).
`SelectorSet._total_specificity()` (`model.py:210-222`) sums the triple across a compound selector.
`CombinatorType`: `SAME` (no space), `DESCENDENT` (space), `CHILD` (`>`).

**Matching** is an explicit backtracking stack machine, `css/match.py:_check_selectors` over
`(selector_index, node_index)` against `node.css_path_nodes` — not a regex. `DOMQuery`/`app.query()`
(`css/query.py`) reuses the same `match()` as the stylesheet — one engine for both.
`SelectorSet.is_simple` (`model.py:198-203`) flags ID/TYPE-only sets (no pseudo) used by
`Stylesheet.apply()`'s cache-key (`:first-child`-style pseudo excluded via
`_EXCLUDE_PSEUDO_CLASSES_FROM_CACHE`).

## II.3 CSS variables (`$`-tokens) — substituted before parse

`VARIABLE_REF = r"\$[a-zA-Z0-9_\-]+"` (`css/tokenize.py:30`). Resolution is **token substitution
before parsing**:

- `Stylesheet.__init__(variables=...)` stores `dict[str,str]`; `_variable_tokens` lazily
  `tokenize_values(...)` each value once (`css/tokenize.py:311-323`).
- `css/parse.py::substitute_references(tokens, css_variables)` (`parse.py:367-436`): on a
  `variable_name` token (`$foo:`) captures the definition (supports `$x: $y;` chains); on a
  `variable_ref` token (`$foo` used) splices the tokenized value (`.with_reference(...)`); an
  unresolved ref raises `UnresolvedVariableError` (`parse.py:343-364`) with a "did you mean" suggestion.
- **Source dict**: `App.get_css_variables()` (`app.py:1419-1435`) =
  `{**theme.to_color_system().generate(), **theme.variables, **get_theme_variable_defaults()}` layered
  `{**theme_variables, **variables}` (theme wins over app defaults). Passed to `Stylesheet(variables=…)`
  at construction (app.py:727); refreshed via `stylesheet.set_variables()`/`reparse()` on theme change
  (`_watch_theme`→`refresh_css`, app.py:3800-3806).

So `$primary`/`$background`/etc. are **not** render-time custom properties — they are gone by
`Stylesheet.parse()`; a theme change forces a full `reparse()` of every CSS source. Variable list →
**Quick ref §0.7**.

## II.4 Component classes & `get_component_rich_style`

`COMPONENT_CLASSES: ClassVar[set[str]] = set()` (`dom.py:144`) declares virtual selector names for a
widget's internal parts (line-API widgets with no child DOM). Example `OptionList`
(`widgets/_option_list.py:179-193`): `{"option-list--option", …, "option-list--separator"}`. CSS
targets them `OptionList > .option-list--option-hover { background: $accent; }`. Retrieval:

- `DOMNode.get_component_styles(*names) -> RenderStyles` (`dom.py:601-624`) — raw merge; `KeyError` if
  a name isn't declared.
- `Widget.get_component_rich_style(*names, partial=False, default=None) -> rich.style.Style`
  (`widget.py:1175-1213`) — caches by names tuple, applies `text_opacity` blending; `partial=True`
  excludes ancestor styles.
- `Widget.get_visual_style(*component_classes, partial=False) -> VisualStyle` (`widget.py:1216-1275`)
  — newer, richer (Textual `VisualStyle`); walks `ancestors_with_self` reversed accumulating
  bg/color/opacity + `auto_color` contrast text. Prefer for manual `Strip`/`Segment` work against
  Textual `Visual`/`Content`, since it understands opacity/tint compositing a bare Rich `Style` cannot.

## II.5 The theme system (theme.py, design.py)

`Theme` dataclass (`theme.py:12-67`): `name, primary, secondary?, warning?, error?, success?, accent?,
foreground?, background?, surface?, panel?, boost?, dark=True, luminosity_spread=0.15, text_alpha=0.95,
variables={}, ansi=False`. `Theme.to_color_system()` is a 1:1 copy into `design.ColorSystem`
(`design.py:24-89`) which derives everything.

**21 built-in themes** in `BUILTIN_THEMES` (`theme.py:70-513`): `textual-dark`, `textual-light`, `nord`,
`gruvbox`, `catppuccin-{mocha,latte,frappe,macchiato}`, `dracula`, `tokyo-night`, `monokai`, `flexoki`,
`solarized-{light,dark}`, `rose-pine{,-moon,-dawn}`, `atom-one-{dark,light}`, `ansi-{dark,light}` — all
registered in `__init__` (`for theme in BUILTIN_THEMES.values(): self.register_theme(theme)`,
app.py:601-602).

`ColorSystem.generate()` (`design.py:105-113`, body `_generate`/`_generate_ansi`) produces the **full
`$`-variable dict** (§II.3, Quick ref §0.7). When `ansi=True` (the two `ansi-*` themes),
`_generate_ansi()` maps every color to a literal `ansi_<name>` token — the terminal's own 16/256
palette, no RGB math.

`App.theme` reactive + registration (app.py:560-561, 1439-1520):
```python
theme: Reactive[str] = Reactive(constants.DEFAULT_THEME)   # env TEXTUAL_THEME or "textual-dark"
```
- `register_theme(theme)` — `_registered_themes[theme.name] = theme` (silent overwrite).
- `available_themes -> dict[str, Theme]` — `{**_registered_themes}`.
- `get_theme(name) -> Theme | None` (`app.py:1439-1453`) — accepts a **comma-separated fallback list**
  (`"ksbc,textual-dark"`), splits/strips, returns the first in `available_themes` ([resolved](#verification-ledger)).
- `current_theme -> Theme` (`app.py:1487`) — `get_theme(self.theme)`, defaulting `"textual-dark"`.
- **Setting `app.theme` before registering it raises `InvalidThemeError`** (`_validate_theme`,
  app.py:1494-1500) — registration must precede activation.
- `_watch_theme` (`app.py:1503-1520`) toggles `-theme-<name>`, sets `-dark-mode`/`-light-mode` from
  `theme.dark`, refreshes the ANSI→truecolor filter, `_invalidate_css()`, and schedules
  `refresh_css(animate=False)` next tick — a full reparse + app-wide restyle.

**No `App.dark: bool`** in 8.2. `dark`/`light` are theme-derived pseudo-class predicates
(`app.py:541-542`): `"dark": lambda app: app.current_theme.dark`, `"light": lambda app: not
app.current_theme.dark` — usable as `App:dark { … }`/`:light`. `App.action_toggle_dark()`
(`app.py:4545-4552`) flips the two `textual-*` builtins — but it is **not wired to any default key**:
`App.BINDINGS` (app.py:454-464) contains only `ctrl+q`→quit (priority) and `ctrl+c`→help_quit (system);
`action_toggle_dark` exists solely as a named action ([resolved](#verification-ledger)).

**KSBC plug-in** (`repo:src/babylon/tui/theme.py:40-61`): a plain `Theme` instance (no subclassing),
`name="ksbc"`, `primary=CRIMSON`, `accent=GOLD`, etc., with a `variables={}` override dict
(`block-cursor-background=GOLD`, `footer-key-foreground=GOLD`, `link-color=GOLD`, `text-muted=DIM`,
`autopause-amber=AMBER`, …). `ArchiveApp.on_mount` (`repo:src/babylon/tui/app.py:242-244`) does
`register_theme(KSBC); self.theme = "ksbc"` (that order — registration first). `autopause-amber` is a
**Babylon-invented** variable name with no Textual meaning; it round-trips into `$autopause-amber`
purely because `get_css_variables()` merges `theme.variables` unconditionally — a supported pattern
(`get_theme_variable_defaults()` exists for exactly this), not a hack. Design-bible drift → **Part IV
§4.7**.

## II.6 Layout systems

`Styles.layout = LayoutProperty()` (`css/styles.py:288`), `VALID_LAYOUT = {"vertical","horizontal",
"grid","stream"}` (`css/constants.py:32`), resolved via `layouts/factory.py::get_layout(name)` →
`LAYOUT_MAP` → `VerticalLayout`/`HorizontalLayout`/`GridLayout`/`StreamLayout`. Every
`Layout.arrange(parent, children, size, greedy)` returns an `ArrangeResult` (list of
`WidgetPlacement`), computed via `textual._resolve.resolve_box_models`. `VerticalLayout.arrange`
(`layouts/vertical.py:19-101`) walks top-to-bottom, resolving heights, folding adjacent margins
(`max(margin_bottom, margin_top)` — margin-collapse, not sum), skipping the `y` advance for
`overlay=="screen"` or `position=="absolute"` children.

**Docking, layers, overlay** are a separate pass, `textual/_arrange.py::arrange()`, **before** the
named layout: `_build_layers(widgets)` (`_arrange.py:19-30`) groups by `layer`; `partition(_get_dock,
…)` (`_arrange.py:79-80`, `_get_dock=attrgetter("styles.is_docked")`) splits `layout_widgets` vs
`dock_widgets` (edges from `dock_widget.styles.dock`, `_arrange.py:153-176`) — a docked child skips the
named `Layout` entirely; `overlay: screen|none` (`css/styles.py:491-493`, `VALID_OVERLAY`,
`css/constants.py:86`) is checked by each `Layout.arrange` to exempt a widget from flow space while
still placing it.

**`Container` classes** (`textual/containers.py`, read in full):

| Class | `DEFAULT_CSS` layout | Sizing | Scrollbars |
|---|---|---|---|
| `Container` | vertical | `1fr × 1fr` | `overflow: hidden hidden` |
| `ScrollableContainer` | vertical | `1fr × 1fr` | `auto auto` + arrow/page/home/end BINDINGS |
| `Vertical` | vertical | `1fr × 1fr` | hidden |
| `VerticalGroup` | vertical | `1fr w × auto h` | hidden |
| `VerticalScroll` | vertical | inherited | `overflow-x: hidden; overflow-y: auto` |
| `Horizontal` | horizontal | `1fr × 1fr` | hidden |
| `HorizontalGroup` | horizontal | `1fr w × auto h`\* | hidden |
| `HorizontalScroll` | horizontal | inherited | `overflow-y: hidden; overflow-x: auto` |
| `Center` | (vertical) | `align-horizontal: center` | — |
| `Right` | — | `align-horizontal: right` | — |
| `Middle` | — | `align-vertical: middle` | — |
| `CenterMiddle` | — | `align: center middle` | — |
| `Grid` | grid | `1fr × 1fr` | — |
| `ItemGrid` | grid | `1fr w × auto h` | reactive `min/max_column_width`/`stretch_height`/`regular` |

\* `HorizontalGroup`'s literal CSS is `width: 1fr; height: auto;` (auto is the *height*, mirroring
`VerticalGroup`'s auto *width* — `containers.py:175-182` vs `139-145`).
`ScrollableContainer.BINDINGS` (`containers.py:52-62`) wires up/down/left/right/home/end/pageup/
pagedown/ctrl+pageup/ctrl+pagedown to `action_scroll_*`, all `show=False`.

## II.7 The render pipeline

**Path A — `render()` (common case):**
```
Widget.render(self) -> RenderResult          # widget.py:4432 — str | Content | rich renderable | Visual
       ▼
Widget._render(self) -> Visual               # widget.py:4460, cached in _layout_cache["_render.visual"]
       │  textual.visual.visualize(self, self.render(), markup=...):
       │    str → Content.from_markup (or Content(...) if markup=False); Rich Text → Content.from_rich_text;
       │    Rich renderable (__rich_console__) → RichVisual; Visual → passthrough;
       │    obj.visualize() → SupportsVisual protocol (visual.py:52-55)
       ▼
Visual.to_strips(widget, visual, width, height, style, ...) -> list[Strip]   # visual.py:191-266
       │  text-selection style, auto-link styling, pad to width, content_align (Strip.align)
       ▼
Widget._render_content(self)                 # caches list[Strip] + dirty size in _render_cache
       ▼
Widget.render_line(y) -> Strip               # widget.py:4250-4263, from _render_cache, recompute only if dirty
```
`Static.render()` (`widgets/_static.py:77-83`) returns `self.visual`; the `content` setter
(`widgets/_static.py:70-76`) builds the `Visual` eagerly so renders don't re-parse markup.

**Path B — `render_line(y)` override (the line API):** widgets with large/virtualized content override
`render_line(y)` directly, computing a `Strip` for row `y` from internal state, bypassing
`render()`/`Visual`. Verified overrides in this install: `RichLog` (`_rich_log.py:301`), `Input`
(`_input.py:616`), `SelectionList` (`_selection_list.py:506`), `Log` (`_log.py:281`), `DataTable`
(`_data_table.py:2478`), `TextArea` (`_text_area.py:1336`), `OptionList` (`_option_list.py:884`),
`Tree` (`_tree.py:1303`), `MaskedInput` (`_masked_input.py:554`). Use Path A for ordinary content;
override `render_line()` only for huge scrollable content (maintain a row cache, slice it).

**Both paths funnel through the same decoration step.** `Widget.render_lines(crop) -> list[Strip]`
(`widget.py:4271-4283`) calls `self._styles_cache.render_widget(self, crop)` (`_styles_cache.py:99`,
`StylesCache`), which applies **border, outline, padding, scrollbar gutters, hatch** universally —
never the individual widget's concern.

**Rich renderables vs `Strip`/`Segment`:** Rich renderables (`__rich_console__`) are *input* to
`Visual`/`RichVisual` — fine for content without per-frame updates. `Strip` (`textual/strip.py:69-`)
is Textual's **immutable list-of-`Segment`** for one terminal row; supports `.crop()`,
`.extend_cell_length()`, `.apply_style()`, per-instance FIFO caches (strip.py:83-92) — the only type
the compositor blits. `StripRenderable` (`strip.py:43-67`) is the reverse adapter (wrap `list[Strip]`
for Rich's `Console.print`, e.g. `export_screenshot`).

## II.8 Refresh semantics — `refresh()` / `recompose()` / reactive-driven

`Widget.refresh(*regions, repaint=True, layout=False, recompose=False) -> Self`
(`widget.py:4324-4368`):

| Call | Effect |
|---|---|
| `refresh()` | `repaint=True` default: clears `_rich_style_cache`, marks dirty, schedules content-only re-render on next idle — no re-layout |
| `refresh(layout=True)` | also `_layout_required=True` — parent box-model layout re-runs (size/position may change) |
| `refresh(recompose=True)` | `_recompose_required=True`, schedules `_check_recompose()` next tick; **short-circuits** the repaint/layout bookkeeping |

`Widget.recompose()` (`widget.py:1704-1716`): `await query_children("*").exclude(".-textual-system")
.remove()` then re-run `compose(self)` + `mount_all(...)` — **destroys and rebuilds the entire child
DOM**. Categorically more expensive than `refresh(layout=True)` (which keeps instances, only
re-measures) — use only when the *set* of children must change.

`Static.update(content, *, layout=True)` (`widgets/_static.py:82-92`) — rebuilds `__visual` via
`visualize(...)`, then `refresh(layout=layout)`; pass `layout=False` when size won't change.

**Reactive-driven refresh is the normal path** (`reactive.Reactive.__init__`, reactive.py:142-153):
`layout=False, repaint=True` (**default true**), `recompose=False`, `always_update=False`. Any
`reactive(...)` attribute triggers a repaint-only refresh on change — no explicit `self.refresh()` in
`watch_*` for the common case. Set `reactive(x, layout=True)` if `x` changes measured size,
`reactive(x, recompose=True)` if it changes which children exist. Backs the docstring "it is rarely
necessary to call `refresh()` explicitly" (widget.py:4342). Full reactive descriptor → **Part I §4**.

## II.9 Images in the terminal — `textual-image` 0.13.2

Separate PyPI package (`textual_image`, pinned via the `textual` extra, ADR099). Detection is
**module-level, at import time — before Textual boots** (`textual_image/renderable/__init__.py`):
```python
is_tty = sys.__stdout__ and sys.__stdout__.isatty()
if is_tty and sixel.query_terminal_support():  Image = SixelImage
elif is_tty and tgp.query_terminal_support():   Image = TGPImage
elif is_tty:                                     Image = HalfcellImage
else:                                            Image = UnicodeImage
```
Package auto-detect order: **Sixel → Kitty TGP → half-cell → unicode**. Both
`query_terminal_support()` (`renderable/tgp.py:195-225`, `renderable/sixel.py:108-138`) write a real
escape-sequence probe to stdout (TGP `a=q` query APC; Sixel `ESC[c` DA) and block-read the reply from
stdin with a 0.1s timeout via `_terminal.py::capture_terminal_response` — a live terminal probe, not a
`TERM` heuristic. **It must run before Textual claims stdin** ("Textual runs a thread to read stdin and
will grab the response" — verbatim docstring). `AutoImage`/`Image` is a **module-level variable, not a
class** (mypy: "Variable … is not valid as a type").

**Babylon does not use the package's auto-detect** (`repo:src/babylon/tui/map_room.py`,
`repo:src/babylon/render/capability.py`). Two documented deviations:
1. **Sixel is excluded as a target** (ADR099: "sixel is not a target"). Babylon's
   `TextualImageQuerier.detect_pixel_protocol()` (`repo:src/babylon/render/capability.py:125-139`)
   checks `tgp.query_terminal_support()` **first**, falls back to `sixel.query_terminal_support()`
   (returns `"sixel"` if hit) — inverted from upstream — and never imports/constructs `SixelImage`
   (`map_room.py` imports only `HalfcellImage, TGPImage`).
2. **Detection is decoupled from widget construction** — `babylon.render.capability.probe()` is a pure
   function over `(env, queries)` run once by a `babylon doctor` step and persisted to a
   `CapabilityReport` frozen model; `render_map_room(cells, *, render_tier)`
   (`repo:src/babylon/tui/map_room.py:120-142`) takes `render_tier` as a **caller-supplied parameter**,
   never re-probing — an explicit rejection of the "discover dependencies at runtime" anti-pattern.

**Glyph techniques (verified: no sextants in this install):**
- **`HalfcellImage`** (`renderable/halfcell.py`): two source pixel-rows per cell, one glyph `▀`
  (U+2580 UPPER HALF BLOCK), upper pixel RGB → `Style.color`, lower → `Style.bgcolor` — full 24-bit
  per half-cell, doubling vertical resolution. No sextant glyphs (U+1FB00–1FB3B) anywhere in the
  package (`rg -i sextant` = 0).
- **`UnicodeImage`** (`renderable/unicode.py`): last-resort, **grayscale only**, one glyph per pixel
  (1:1), 5-level ramp `_CHARACTERS = ["█","▓","▒","░"," "]` via `_CHARACTER_LOOKUP` (256/5 bins). No
  color survives — a legibility floor.
- **`SixelImage`/`TGPImage`** bypass glyphs — raw escape sequence (DCS Sixel / Kitty APC) with actual
  pixel data. Exact Sixel-supporting terminal list is **`[UNVERIFIED]`** — not enumerated in installed
  source (external terminal capability).

Babylon's map-room intent (`repo:src/babylon/tui/map_room.py:14`): "Cell-art is the design target, not
the fallback (ADR097 D1)" — `HalfcellImage` is primary, `TGPImage` an opt-in raster upgrade behind
`render_tier`, `UnicodeImage`/`SixelImage` not wired in. Babylon usage → **Part IV §4.6**.

## II.10 Text export surfaces

**There is no `App.export_text()` / `Screen.export_text()` in 8.2.8** — `rg -n "export_text"` over
`textual/` = 0 (only Rich's `Console.export_text`). First-class capture is **SVG only**, all in
`app.py`:

| Method | Signature | Behavior |
|---|---|---|
| `App.export_screenshot` | `(*, title=None, simplify=False) -> str` (app.py:1855-1878) | builds `Console(width, height, file=StringIO(), force_terminal=True, color_system="truecolor", record=True, legacy_windows=False, safe_box=False)`, prints `screen._compositor.render_update(full=True, …, simplify=simplify)`, returns `console.export_svg(title=...)` |
| `App.save_screenshot` | `(filename=None, path=None, time_format=None) -> str` (app.py:1889-1911) | `export_screenshot()` → write SVG → return path |
| `App.deliver_screenshot` | `(filename=None, path=None, time_format=None) -> str\|None` (app.py:1913-1938) | same SVG via `deliver_text(...)` (local-save-or-browser-serve) |
| `App.action_screenshot` | `(filename=None, path=None) -> None` (app.py:1844-1853) | action wrapper around `deliver_screenshot`, bindable |

**Plain-text splice**: `export_screenshot`'s `Console` uses `record=True` so `export_svg()` can replay
segments. Rich also exposes `export_text(*, clear=True, styles=False) -> str` (`rich/console.py:2192`,
present in the installed dep) against the same recording — Textual never calls it, but replicating the
`Console(record=True)` + `console.print(screen._compositor.render_update(...))` setup yields
`console.export_text()` for byte-for-byte full-screen text (borders/padding/decorations included). See
**Quick ref §0.5** for the snippet.

**Per-widget text primitives (narrower BDD-tier assertions, no screenshot):**
- `Strip.text` (`strip.py:118-121`): `"".join(seg.text for seg in self._segments)` — plain text of one
  rendered row, style-stripped.
- `Widget.render_line(y) -> Strip` / `render_lines(crop: Region) -> list[Strip]` (`widget.py:4250-4283`)
  — callable against a mounted widget under `run_test()`'s Pilot, exact rendered rows, no screenshot.
- Pre-render: `Static.content` / `Static.visual` (`widgets/_static.py:52-76`) — a `Content` object's
  plain text without the render pipeline.

**Repo precedent**: `repo:tests/unit/tui/snapshots/*` uses `pytest-textual-snapshot`'s `snap_compare`
(SVG-based only, matching Textual's only first-class surface). No plain-text capture helper exists in
the repo today — a BDD text tier would be new code on the `Strip.text`/`render_lines` primitives or the
`Console(record=True)` + `export_text()` pattern. Snapshot mechanics → **Part III §3.3**.

---

# Part III — Testing and config/env

## III.1 `App.run_test()` — signature and semantics

Source: `app.py:2133-2219` (an `@asynccontextmanager` async generator).
```python
async def run_test(self, *, headless=True, size=(80,24), tooltips=False,
                   notifications=False, message_hook=None) -> AsyncGenerator[Pilot[ReturnType], None]:
```

| Param | Default | Effect (verified) |
|---|---|---|
| `headless` | `True` | selects `HeadlessDriver` (app.py:3334-3335); no terminal I/O; `write()` no-op (drivers/headless_driver.py:36-42) |
| `size` | `(80,24)` | forces reported terminal size. Not `None` by default — contrast `HeadlessDriver._get_terminal_size()`'s own `(80,25)` fallback (drivers/headless_driver.py:18-34), which only matters if you pass `size=None` |
| `tooltips` | `False` | `app._disable_tooltips = not tooltips` (app.py:2170) |
| `notifications` | `False` | `app._disable_notifications = not notifications` (app.py:2171) |
| `message_hook` | `None` | bound into `message_hook_context_var` (app.py:2188); `MessagePump` reads it per-message (message_pump.py:718) |

Lifecycle (app.py:2199-2218): 1) message loop launched as background Task (`create_task(run_app(app),
name=f"run_test {app}")`); 2) awaits `app_ready_event` before yielding — blocks until mounted/composed;
3) constructs `Pilot(app)`, `await pilot._wait_for_screen()`, then `yield`s the Pilot; 4) on exit:
`asyncio.sleep(0)` → `app._shutdown()` → `await app_task`; 5) **`if self._exception: raise
self._exception`** — a background-task crash surfaces as a real pytest failure.

Usage (docstring, app.py:2150-2156):
```python
async with app.run_test() as pilot:
    await pilot.click("#Button.ok")
    assert ...
```

### III.1.1 Exception propagation and return codes

- `App._handle_exception(error)` (`app.py:3263-3276`): `_return_code=1`; if `_exception is None`,
  stashes `_exception=error` and sets `_exception_event` — **only the first** exception is captured.
- `App.return_code` (`app.py:975-990`): `int|None`; `None` until exit, `0` on graceful `exit()`, `1`
  auto on any unhandled exception, or an explicit `exit(return_code=…)`. Docstring recommends
  `sys.exit(my_app.return_code)` — **Textual never calls `sys.exit` for you**.
- `App.exit(result=None, return_code=0, message=None)` (`app.py:1270-1288`): sets `_return_value`/
  `_return_code`, posts `ExitApp`; `message` (a `RenderableType`) is printed after teardown.
- `Pilot.exit(result)` (`pilot.py:559-570`): `_wait_for_screen()` + `wait_for_idle()` then
  `app.exit(result)`.

### III.1.2 `Pilot` API — exhaustive (pilot.py, 570 lines; all methods `async`)

| Method | Signature | Behavior |
|---|---|---|
| `press` | `press(*keys: str) -> None` | `App._press_keys(keys)` then `_wait_for_screen()`. Grammar → §III.1.3 |
| `resize_terminal` | `(width, height) -> None` | if `HeadlessDriver`, mutates `driver._size`; always posts `Resize`, then `pause()` |
| `mouse_down` | `(widget=None, offset=(0,0), shift=False, meta=False, control=False, button=1) -> bool` | posts `[MouseMove, MouseDown]`; `OutOfBounds` if offset outside `screen.size.region` |
| `mouse_up` | `(widget=None, offset=(0,0), shift=False, meta=False, control=False) -> bool` | posts `[MouseMove, MouseUp]`, `button=1` fixed |
| `click` | `(widget=None, offset=(0,0), shift=False, meta=False, control=False, times=1, button=1) -> bool` | posts `[MouseDown, MouseUp, Click]` — **bypasses `App.on_event`** (docstring); patches `mouse_position`/tooltip manually in `_post_mouse_events` |
| `double_click` | same minus `times` | `self.click(..., times=2)` (pilot.py:296-298) |
| `triple_click` | same, `times=3` | pilot.py:345-347 |
| `hover` | `(widget=None, offset=(0,0)) -> bool` | `pause()` first, then `[MouseMove]` |
| `_wait_for_screen` | `(timeout=30.0) -> bool` | walks `[app, *screen.walk_children(with_self=True)]`, registers `call_later(decrement)` on each, waits on an Event set when the counter hits 0, racing `app._exception_event`; raises `WaitForScreenTimeout` on neither-in-`timeout` ("some kind of deadlock", pilot.py:59-64) |
| `pause` | `(delay=None) -> None` | `_wait_for_screen()`; if `delay is None`: `wait_for_idle(0)` (§III.1.4), else `asyncio.sleep(delay)`; then `screen._on_timer_update()` |
| `wait_for_animation` | `() -> None` | `animator.wait_for_idle()` + `_on_timer_update()` |
| `wait_for_scheduled_animations` | `() -> None` | `_wait_for_screen()` → `animator.wait_until_complete()` → `_wait_for_screen()` → `wait_for_idle()` → `_on_timer_update()` — waits for not-yet-started animations too |
| `exit` | `(result) -> None` | `_wait_for_screen()` → `wait_for_idle()` → `app.exit(result)` |

Pilot exceptions: `OutOfBounds` (pilot.py:56-58), `WaitForScreenTimeout` (pilot.py:61-65).
`_post_mouse_events` (pilot.py:388-451): for `times>1` loops `chain in range(1, times+1)` stamping
`chain` onto `Click` only; the widget-under-cursor (`app.get_widget_at`) is captured once, before the
first event, and reused.

### III.1.3 `press()` key-string grammar (App._press_keys, app.py:2050-2068)

- `"wait:<ms>"` is **not** a keypress — `await asyncio.sleep(ms/1000)` then
  `app._animator.wait_until_complete()`. The **only** engine-level special token.
- any other single non-alnum char → `_character_to_key(key)`.
- `REPLACED_KEYS.get(key, key)` normalizes aliases before Unicode lookup.
- dispatched as `events.Key(key, char)`, `char` via `unicodedata.lookup(_get_unicode_name_from_key(...))`
  falling back to the raw char or `None`.
- each keypress: `driver.send_message(key_event)` then `wait_for_idle(0)` **twice**, sandwiching
  `app._animator.wait_until_complete()`.
- **`"_"` is not a pause here** — it is length-1 and `"_".isalnum()` is `False`, so it routes through
  `_character_to_key("_")` as an ordinary character key. The "short pause" meaning of `"_"` documented
  by `snap_compare`/`take_svg_screenshot` is **snapshot-plugin convention only**, not `_press_keys`
  behavior ([resolved](#verification-ledger)).

### III.1.4 `wait_for_idle` — the CPU-idle heuristic (textual/_wait.py, 39 lines)

```python
async def wait_for_idle(min_sleep: float = SLEEP_GRANULARITY, max_sleep: float = 1) -> None
```
`SLEEP_GRANULARITY = 1/50` (20ms), `SLEEP_IDLE = SLEEP_GRANULARITY/20.0` (1ms). Loops `await
sleep(SLEEP_GRANULARITY)`, comparing wall-clock (`monotonic()`) against process CPU delta
(`time.process_time()`); breaks once `elapsed >= max_sleep` (hard cap) **or** (`elapsed > min_sleep`
and `cpu_elapsed < SLEEP_IDLE`) — detects that the process stopped burning CPU. Why `pilot.pause()`
with no delay beats a hardcoded `asyncio.sleep(n)`: it adapts to actual settle time, bounded at 1s.

## III.2 Headless mode and terminal-size forcing

- `HeadlessDriver` (`drivers/headless_driver.py`, 67 lines) — `is_headless` returns `True` (overriding
  `Driver.is_headless`=`False`, driver.py:47-50); `write()` no-op; `start_application_mode()` still
  posts one synthetic `Resize` from `_get_terminal_size()`.
- `App.is_headless` (`app.py:1187-1193`): `False if _driver is None else _driver.is_headless`.
- `App._print` (`app.py:2081-2103`): under headless, printed text also goes to
  `_original_stderr`/`_original_stdout` — so `print()` under `run_test` reaches captured stdout.
- `Pilot.resize_terminal` (`pilot.py:82-93`): mutates `driver._size` only for a `HeadlessDriver`; a
  real driver just gets the `Resize` (real terminals report size via SIGWINCH).

## III.3 `pytest-textual-snapshot` mechanics

Installed: `pytest_textual_snapshot.py` (single-file, 395 lines), registered via `[pytest11]
textual-snapshot = pytest_textual_snapshot`.

### III.3.1 The `snap_compare` fixture

```python
@pytest.fixture
def snap_compare(snapshot: SnapshotAssertion, request) -> Callable[[str | PurePath], bool]
```
Returned `compare(app, press=(), terminal_size=(80,24), run_before=None) -> bool` (lines 139-224):
- `app` = an `App` instance or a path (absolute or relative to the **test file's** directory, "NOT
  where pytest was invoked", line 174) via `textual._import_app.import_app`.
- renders via `textual._doc.take_svg_screenshot(app, press, terminal_size, run_before)` (§III.3.3).
- `snapshot = snapshot.use_extension(SVGImageExtension)` — a syrupy `SingleFileSnapshotExtension`
  (`_file_extension="svg"`, `WriteMode.TEXT`) normalizing SVGs on read and write via `normalize_svg()`
  (§III.3.4). One `.svg`/snapshot at `__snapshots__/<test_file>/<test_name>.svg`.
- every call (pass or fail) pickles a tuple `(result, expected_svg, actual, PseudoApp(...), full_path,
  line, name, docstring, app_path, snapshot_exists)` to a per-node file in a shared tempdir,
  consumed by `pytest_sessionfinish` — **unconditional**, so `pass_count` is accurate.
- `--snapshot-update` is a **syrupy** flag (not this plugin's) — overwrites the on-disk `.svg`.

### III.3.2 xdist support

`pytest_sessionstart`/`_sessionfinish` gate on `os.environ.get("PYTEST_XDIST_WORKER") is None` — only
the xdist **master** creates the `TemporaryDirectory`, writes `TEXTUAL_SNAPSHOT_TEMPDIR` (inherited by
workers), aggregates the HTML report (`save_svg_diffs`), and prints the summary
(`pytest_terminal_summary`). `repo:.mise.toml:456-457` runs `pytest tests/unit -n 4 --dist loadscope`,
so `repo:tests/unit/tui/snapshots/` runs inside that xdist run — the gating is load-bearing.

### III.3.3 `take_svg_screenshot` — how the render happens (_doc.py:63-146)

```python
def take_svg_screenshot(app=None, app_path=None, press=(), hover="", title=None,
    terminal_size=(80,24), run_before=None, wait_for_animation=True, simplify=True) -> str
```
- If `app_path is not None` **and** `run_before is None`: results cache to
  `.screenshot_cache/<md5>.svg`, keyed by MD5 of every CSS file's bytes + `f"{press}-{hover}-{title}-
  {terminal_size}"` (lines 100-108). Applies to the doc-fence renderer (`format_svg`), **not** to
  `snap_compare` given an `App` instance (`app_path=""`, pytest_textual_snapshot.py:158) — but does
  apply when `snap_compare` gets a file path (`import_app` yields a real `app_path`).
- inner `auto_pilot` (lines 122-138): `pilot.pause()` → `press(*press)` → (if `hover`) `hover(hover)` +
  `pause(0.5)` → (if `wait_for_animation`) `wait_for_scheduled_animations()` + `pause()` →
  unconditional `pause()` → unconditional `wait_for_scheduled_animations()` →
  `app.export_screenshot(title, simplify)` → `app.exit(svg)`.
- driven via `app.run(headless=True, auto_pilot=auto_pilot, size=terminal_size)` — **not** `run_test`;
  a distinct code path from §III.1.

### III.3.4 SVG snapshot determinism — what makes bakes sha-identical

1. **Rich's `Console.export_svg()` unique-id is content-derived** (`rich/console.py:2475-2480`):
   ```python
   if unique_id is None:
       unique_id = "terminal-" + str(zlib.adler32(("".join(repr(s) for s in segments)).encode("utf-8","ignore")))
   ```
   the `terminal-<N>-...` CSS-class prefix is an **Adler-32 of every segment's `repr()`** — identical
   content ⇒ identical id. This is what "byte-identical across delete-and-regenerate (sha256
   double-bake)" (ADR099) rests on.
2. **The plugin strips it anyway** (`pytest_textual_snapshot.py:92-94`):
   ```python
   def normalize_svg(svg): return re.sub(r"\bterminal-\d+-([\w-]+)", r"terminal-\1", svg)
   ```
   applied on both serialize and read — comparisons are id-insensitive. `individualize_svg` (a
   `random()`-based id) exists only for the HTML failure report, never in the pass/fail path.
3. **Repo-local wrinkle** (`repo:tests/unit/tui/snapshots/conftest.py`): Rich's SVG-export template
   bakes one line of trailing whitespace into every SVG; the repo's `trailing-whitespace` hook would
   strip it from staged goldens, desyncing them from a live re-render. Fix: an `autouse` fixture
   monkeypatching `textual._doc.take_svg_screenshot` to `.rstrip()` every line **before** the syrupy
   comparison, scoped to `tests/unit/tui/snapshots/` — a documented, load-bearing local workaround.

### III.3.5 The ADR099 syrupy/pytest pin gotcha

`pytest-textual-snapshot-1.1.0.dist-info/METADATA`:
```
Requires-Dist: pytest (>=8.0.0)
Requires-Dist: syrupy (==4.8.0)
```
`syrupy==4.8.0` declares `pytest<9` (per `repo:ai/decisions/ADR099_archive_stack.yaml`) — incompatible
with the repo's `pytest>=9.0.3,<10` (`repo:pyproject.toml:99`, resolved `9.1.1`). Resolution:
- `repo:pyproject.toml:140-141`: `"pytest-textual-snapshot==1.1.0"`, `"syrupy>=5.5,<6.0.0"`.
- `repo:pyproject.toml:156-157`: `[tool.uv] override-dependencies = ["syrupy>=5.5,<6.0.0"]` — forces
  the resolver to ignore the plugin's `syrupy==4.8.0` and substitute `>=5.5,<6.0.0` graph-wide.
- `repo:uv.lock`: `pytest-textual-snapshot==1.1.0` (unchanged), `syrupy==5.5.3` (not 4.8.0) — the
  installed syrupy genuinely violates the plugin's declared pin, by design.
- `ADR099` decision point 7 records this "REAL FINDING" and the resolution.

## III.4 Textual dev tools (`textual console` / `textual run --dev`)

**Not installed in this venv** — no `textual_dev` package, no `textual` console-script in `.venv/bin/`.
The `textual` CLI ships in the separate `textual-dev` PyPI package (not a repo dep). Devtools is
**conditionally, gracefully absent** (`app.py:762-771`):
```python
self.devtools: DevtoolsClient | None = None
if "devtools" in self.features:
    try:    from textual_dev.client import DevtoolsClient; from textual_dev.redirect_output import StdoutRedirector
    except ImportError: pass
    else:   self.devtools = DevtoolsClient(constants.DEVTOOLS_HOST); ...
```
- `"devtools" in self.features` needs `TEXTUAL` env to contain `devtools` (§III.5), normally set by the
  absent `textual run --dev`.
- `App._init_devtools` (`app.py:3353-3362`) `await self.devtools.connect()`, logging either way — a
  failed connection is non-fatal.
- `App._log` (`app.py:1703-1752`) is a no-op when `devtools is None or not is_connected` — with
  `textual-dev` absent, `self.log(...)` only reaches `TEXTUAL_LOG`.
- **`TEXTUAL_LOG` is the devtools-independent fallback** (`textual/__init__.py:84-94`):
  `Logger.__call__` appends to `open(constants.LOG_FILE, "a")` if `TEXTUAL_LOG` is set, regardless of
  devtools — the one dev-tooling path that works with only core `textual`.

## III.5 Every environment variable Textual (core) reads in 8.2.8

Primary registry `textual/constants.py` (all `Final`, readable via `textual.constants.*`); five more
are read outside it.

### III.5.1 `textual/constants.py`

| Env var | Constant | Parser | Default | Effect |
|---|---|---|---|---|
| `TEXTUAL` | (inline, app.py:596) | `parse_features()` | `""` | comma flags ∩ `{"devtools","debug","headless"}`; unknown dropped → `App.features: frozenset` |
| `TEXTUAL_DEBUG` | `DEBUG` | `_get_environ_bool` (`=="1"`) | `False` | at import: `warnings.simplefilter("always", ResourceWarning)`; OR'd into `App.debug`; passed `debug=` to driver |
| `TEXTUAL_DRIVER` | `DRIVER` | raw str | `None` | `"module:Class"`; `get_driver_class()` (app.py:1573-1608) imports + `issubclass(Driver)` check else `RuntimeError`; overrides OS auto-select |
| `TEXTUAL_DISABLE_KITTY_KEY` | `DISABLE_KITTY_KEY` | `_get_environ_bool` | `False` | gates the Kitty-keyboard enable sequence (drivers/linux_driver.py:285-292) |
| `TEXTUAL_FILTERS` | `FILTERS` | comma-split, lowercase | `""` | only `"dim"` → appends `DimFilter()` (app.py:618-621) |
| `TEXTUAL_LOG` | `LOG_FILE` | raw str | `None` | path appended by every `Logger.__call__`, devtools-independent (§III.4) |
| `TEXTUAL_DEVTOOLS_HOST` | `DEVTOOLS_HOST` | raw str | `"127.0.0.1"` | `DevtoolsClient(...)` — only if `textual-dev` installed |
| `TEXTUAL_DEVTOOLS_PORT` | `DEVTOOLS_PORT` | `_get_environ_port` | `8081` | defined but **unreferenced** in core 8.2.8 (`rg` matches only its definition); consumed inside `textual_dev` — **`[UNVERIFIED]`**, not installed |
| `TEXTUAL_SCREENSHOT` | `SCREENSHOT_DELAY` | `_get_environ_int` (min -1) | `-1` | if `>=0`, `App._ready()` schedules a one-shot save+exit timer (app.py:3522-3542) |
| `TEXTUAL_SCREENSHOT_LOCATION` | `SCREENSHOT_LOCATION` | raw str | `None` | `save_screenshot(path=...)` |
| `TEXTUAL_SCREENSHOT_FILENAME` | `SCREENSHOT_FILENAME` | raw str | `None` | `save_screenshot(filename=...)` |
| `TEXTUAL_PRESS` | `PRESS` | comma-split | `""` | **only `run_async`** auto-presses these (app.py:2249-2256); `run_test` does not read it |
| `TEXTUAL_SHOW_RETURN` | `SHOW_RETURN` | `_get_environ_bool` | `False` | on shutdown prints `"The app returned:"` + `Pretty(_return_value)` (app.py:3758-3764) |
| `TEXTUAL_FPS` | `MAX_FPS` | `_get_environ_int` (min 1) | `60` | compositor refresh cap |
| `TEXTUAL_COLOR_SYSTEM` | `COLOR_SYSTEM` | raw str | `"auto"` | `Console(color_system=...)` (app.py:624) |
| `TEXTUAL_ANIMATIONS` | `TEXTUAL_ANIMATIONS` | `_get_textual_animations()` → `"none"/"basic"/"full"` | `"full"` | global animation level |
| `ESCDELAY` (no prefix) | `ESCAPE_DELAY` | `_get_environ_int`(min 1)`/1000` | `100`→`0.1`s | escape-report delay; superseded by Kitty protocol |
| `TEXTUAL_SLOW_THRESHOLD` | `SLOW_THRESHOLD` | `_get_environ_int`(min 100) | `500`ms | slow-message-processing warning threshold |
| `TEXTUAL_THEME` | `DEFAULT_THEME` | raw str | `"textual-dark"` | comma-list "first that exists" — resolved by `get_theme` (app.py:1449, [resolved](#verification-ledger)) |
| `TEXTUAL_SMOOTH_SCROLL` | `SMOOTH_SCROLL` | `_get_environ_int(...,1)==1` | `True` | `=0` disables smooth scroll |
| `TEXTUAL_DIM_FACTOR` | `DIM_FACTOR` | `_get_environ_int`(0–100)`/100` | `66`→`0.66` | ANSI-dim→RGB opacity |

Helper parsers (all in `constants.py`, defensive — invalid/missing → default): `_get_environ_bool`
(`=="1"`), `_get_environ_int(name, default, minimum=None, maximum=None)`, `_get_environ_port`
(range-checks 0–65535).

### III.5.2 Env vars read outside `constants.py`

| Env var | File:line | Effect |
|---|---|---|
| `NO_COLOR` | `app.py:614` | sets `no_color`; appends `NoColor()`/`Monochrome()` filter. **Popped from the dict before the internal `Console`** (app.py:613,623-634) — Rich's own NO_COLOR never fires; Textual reimplements it via filters |
| `TERM_PROGRAM` | `_xterm_parser.py:51`; `drivers/linux_driver.py:322`; `drivers/linux_inline_driver.py:251` | `IS_ITERM` flag; drivers skip the sync-mode query for `"Apple_Terminal"` (echoes a stray `p`) |
| `LC_TERMINAL` | `_xterm_parser.py:50` | `IS_ITERM` (iTerm2) |
| `TEXTUAL_ALLOW_SIGNALS` | `drivers/linux_driver.py:338`; `linux_inline_driver.py:267` | `ISIG = 0 if set else termios.ISIG` — **by default Textual disables ISIG**, so Ctrl+C/Ctrl+\ arrive as `Key` events, not `SIGINT`/`SIGQUIT`. Any truthy value restores signal behavior |
| `TEXTUAL_SPEEDUPS` | `geometry.py:1483` | opt-**out** compiled geometry (`textual_speedups`), default `"1"` (enabled); `=0` forces pure-Python. **Not installed here** — no-op regardless |
| `COLUMNS`, `ROWS` | `drivers/web_driver.py:54-55` | `WebDriver`-only size; irrelevant to terminal/SSH |
| `PYTEST_XDIST_WORKER` | `pytest_textual_snapshot.py:210,229,299,376` | xdist's own; gates the snapshot plugin to master (§III.3.2) |
| `TEXTUAL_SNAPSHOT_TEMPDIR` | `pytest_textual_snapshot.py:214,222` | plugin-internal shared tempdir path |
| `TEXTUAL_SNAPSHOT_FILE_OPEN_PREFIX` | `pytest_textual_snapshot.py:341` | `file://` prefix override for the HTML report |

### III.5.3 Vars intentionally absent — Rich delegation boundary

Textual does **not** read `FORCE_COLOR` or `COLORTERM` (0 matches in `textual/`). `Console(
force_terminal=True, ...)` is passed unconditionally (`app.py:631`), overriding Rich's terminal
detection — so `FORCE_COLOR` can't matter. `COLORTERM` detection lives entirely in vendored Rich's
`Console._detect_color_system` (`rich/console.py:805`, `self._environ.get("COLORTERM","")`). It fires
in `Console.__init__` when `color_system=="auto"` (`rich/console.py:707-712`: `elif color_system ==
"auto": self._color_system = self._detect_color_system()`) — and since Textual passes
`color_system=constants.COLOR_SYSTEM` (default `"auto"`), detection runs at Console construction
([resolved](#verification-ledger); note: at construction, not "first render").

## III.6 Terminal capability detection

- **Color system**: `constants.COLOR_SYSTEM` (`"auto"`) → Rich's `Console(color_system=...)`
  (app.py:624); Rich owns truecolor/256/standard negotiation once triggered (§III.5.3).
  `force_terminal=True` always set — Rich's isatty checks never gate *whether* to render color, only
  the depth.
- **ANSI vs truecolor**: `App.native_ansi_color` (`app.py:1550-1558`) = `ansi_color if not None else
  current_theme.ansi`; controls whether an `ANSIToTruecolor` filter is inserted (`enabled=not
  native_ansi_color`, app.py:611).
- **NO_COLOR**: Textual's own filter chain, not Rich (§III.5.2).
- **Kitty keyboard protocol**: **unconditionally requested** (unless `TEXTUAL_DISABLE_KITTY_KEY=1`)
  by writing `\x1b[>{flags}u`, `flags = DISAMBIGUATE | REPORT_ALL_KEYS | REPORT_ASSOCIATED_TEXT =
  25` (drivers/linux_driver.py:285-292) — **no capability probe before enabling**; unsupporting
  terminals ignore it per spec; disabled again (`\x1b[<u`) before leaving the alt screen.
  `windows_driver.py:99` / `linux_inline_driver.py:211` send the simpler `\x1b[>1u`.
- **Sync-update mode** (`\033[?2026$p`, drivers/linux_driver.py:314-322): sent unless Apple Terminal
  (§III.5.2) or `not self.input_tty` (stdin not a real TTY).
- **In-band window resize** / **bracketed paste**: queried/enabled unconditionally at start; support is
  inferred from whether the terminal emits the response sequences (parsed by `_xterm_parser.py`), not
  from a `TERM` lookup.
- **What breaks over plain SSH/tmux** — **`[UNVERIFIED]`**: Textual issues all probe/enable sequences
  regardless of transport; a terminal/multiplexer that doesn't forward or understand one simply never
  sends the confirming response, so the feature degrades to legacy fallback rather than crashing — no
  `TERM` allowlist/denylist exists in source. Specific `screen`/older-`tmux` Kitty/sync-mode breakage
  is outside what source can verify; flag to the owner, do not assert.

## III.7 Exit codes and signals

- **No automatic `sys.exit`.** Neither `run()` nor `run_async()` calls it (`return_code` docstring
  app.py:983-987 tells callers to).
- **`return_code`**: `None` (not exited) · `0` (graceful `exit()`) · `1` (auto via `_handle_exception`,
  app.py:3271) · any explicit `exit(return_code=...)`.
- **Ctrl+C is not `SIGINT` by default** — `_patch_lflag` clears termios `ISIG` unless
  `TEXTUAL_ALLOW_SIGNALS` (§III.5.2).
- **`SIGWINCH`** hooked (`signal.signal(SIGWINCH, on_terminal_resize)`, drivers/linux_driver.py) to
  re-emit `Resize`, unless in-band resize is active (then a no-op).
- **`SIGTSTP`/`SIGCONT`** (Ctrl+Z suspend/resume): `App.action_suspend_process` (`app.py:4770-4788`)
  sends `os.kill(getpid(), SIGTSTP)`, gated on `not WINDOWS and _driver.can_suspend`.
  `LinuxDriver.start_application_mode` installs **both** handlers (`signal.signal(SIGTSTP,
  self._sigtstp_application)` / `signal.signal(SIGCONT, self._sigcont_application)`,
  drivers/linux_driver.py:78-79); on resume the driver posts `self.SignalResume()`
  (drivers/linux_driver.py:311), consumed by `App`'s `@on(Driver.SignalResume)` handler (app.py:4712)
  ([resolved](#verification-ledger) — there *is* an explicit SIGCONT handler).
- **`run_test`'s exception re-raise** (§III.1) is the test-relevant exit path — a background-task crash
  surfaces from the `async with app.run_test()` block on unwind.

## III.8 Repo config estate

### III.8.1 Exact pins (`repo:pyproject.toml` + `repo:uv.lock`)

| Package | `pyproject.toml` | Resolved (`uv.lock`) | Location |
|---|---|---|---|
| `textual` | `>=8.2,<8.3` | `8.2.8` | deps, line 69 |
| `textual-image` | `>=0.13,<0.14` | `0.13.2` | line 70 |
| `dulwich` | `>=1.2,<2` | `1.2.12` | line 71 |
| `jinja2` | `>=3.1.6,<4.0.0` | `3.1.6` | line 72 |
| `markdown-it-py` | `>=4.2.0,<5.0.0` | `4.2.0` | line 73 |
| `mdit-py-plugins` | `>=0.6.1,<0.7.0` | `0.6.1` | line 74 |
| `textual-plotext` | `>=1.0.1` | `1.0.1` | line 75 |
| `pytest` | `>=9.0.3,<10.0.0` | `9.1.1` | dev, line 99 |
| `pytest-textual-snapshot` | `==1.1.0` | `1.1.0` | line 140 |
| `syrupy` | `>=5.5,<6.0.0` | `5.5.3` | line 141 |

`[tool.uv] override-dependencies = ["syrupy>=5.5,<6.0.0"]` (`repo:pyproject.toml:156-157`) makes the
`pytest-textual-snapshot==1.1.0` / `syrupy>=5.5` combo installable despite the plugin's
`syrupy==4.8.0` metadata (§III.3.5). Rationale in-file comments (`repo:pyproject.toml:66-68,137-139,
152-155`); full evidence `repo:ai/decisions/ADR099_archive_stack.yaml`.

### III.8.2 TUI-relevant `mise` tasks

No dedicated `tui:*`/`snapshot:*` namespace exists. `repo:tests/unit/tui/` (incl. `snapshots/`) is swept
by the general `test:unit` (`repo:.mise.toml:452-462`): `uv run pytest tests/unit -m 'not red_phase and
not slow and not requires_ollama' -n 4 --dist loadscope --max-worker-restart=2 ...` — so the xdist-master
gating (§III.3.2) is live on every `mise run check`/`test:unit`. Adjacent non-Textual tasks:
`test:render` (`repo:.mise.toml:511-521`, the ADR097 render-seam suite, **no** `snap_compare` usage),
`sim:archive`/`sim:archived` (Parquet archival), `archive:record-fixtures{,-community}` (Program 24
fixture harvesting).

---

# Part IV — The Babylon TUI estate (how our code uses Textual)

Scope: cites **this repo** (`repo:`), verified on `feature/archive-wo52b` (HEAD `0f370552`) plus, where
noted, the in-progress `t4` worktree (`.claude/worktrees/t4`, HEAD `6cf0d50f`). Nothing here depends on
a Textual API not in Parts I–III.

## IV.0 Module inventory (`repo:src/babylon/tui/`)

| Module | LOC | What it is |
|---|---|---|
| `app.py` | 326 (keel) / ~430 (t4) | `ArchiveApp` composition shell; `BabylonMarkdown` dialect |
| `directives.py` | 569 | `BabylonFence(MarkdownFence)` fenced-directive dispatcher (6 directives) |
| `wikilinks.py` | 332 | `[[target]]` inline rule + `WikilinkContentMixin` |
| `router.py` | 126 | `babylon://` URI parser → frozen `BabylonTarget` |
| `nav.py` | 313 | `NavShell`/`JumplistState`/`BreadcrumbTrail` — vim `Ctrl-O`/`Ctrl-I` |
| `palette.py` | 152 | `EntityNavigatorProvider(Provider)` — command-palette fuzzy switcher |
| `peek.py` | 302 | `peek(view, depth)` — stat-plate renderer, 4 depths |
| `watchlist.py` | 332 | `WatchlistState` + `render_watchlist` — pinned-entity transclusion |
| `chronicle.py` | 366 | `ChronicleEvent`/`TickBulletin` — tick-bulletin event stream |
| `chronicle_salience.py` | 439 | severity classification, dedup, AMBER autopause, volume floors |
| `campaign_menu.py` | 415 | `CampaignMenu`/`LobbyScreen` — load/new/archive/delete lobby |
| `verb_plate.py` | 186 | `render_verb_plate` — Article V's nine verbs |
| `map_room.py` | 142 | `render_map_room` — choropleth cell-art / TGP-raster (ADR097) |
| `dispatch.py` | 218 | kind-dispatch statblock registry (WO-45 seam) |
| `theme.py` | 63 | `KSBC` Textual `Theme` + ksbc hex tokens |
| `topology/egotree.py` | 109 | Levi ego-tree fence parse + box-drawing render |
| `topology/matrix.py` | 121 | incidence/adjacency grid cell-art render |

Import-linter binds all as `babylon.tui` (§IV.9) — none may import `babylon.engine`,
`babylon.persistence`, or `django`.

## IV.1 `ArchiveApp` — current single-page reality, and what T4 adds

**As shipped (`repo:src/babylon/tui/app.py:164-326`):** `ArchiveApp(App[None])` is **single-page** —
`compose()` (251-262) mounts one `#breadcrumbs` `Label`, one `VerticalScroll(id="page")` wrapping one
`BabylonMarkdown(id="dossier")`, one `#status` `Label`, and a `Footer`. No `Screen`/mode-switching;
`on_mount` (242-249) only registers `KSBC` and, if boot-paged onto the sample, seeds the jumplist.
Construction is fully DI (Constitution: "inject dependencies explicitly"):
```python
def __init__(self, *, page=None, resolver=None, statblocks=None,
             known_entities=None, pages=None, nav=None) -> None: ...
```
`BINDINGS`: `ctrl+o`→`action_jump_back`, `ctrl+i`→`action_jump_forward`. `COMMANDS = App.COMMANDS |
{EntityNavigatorProvider}`. The module-level `app = ArchiveApp()` singleton (line 321) exists only
because `snap_compare` resolves an app path relative to the calling test file — every
`repo:tests/unit/tui/snapshots/*_app.py` constructs its own fresh `ArchiveApp(page=...)`, never the
singleton (`repo:tests/unit/tui/snapshot_app.py:10-16`: a cached `import` hands back a stale mounted
instance — an order-dependent flake). `repo:src/babylon/cli/play.py` still boots the old two-node demo
(`from babylon.__main__ import main as run_demo`). `LobbyScreen` (`repo:src/babylon/tui/
campaign_menu.py:322`) is built + unit-tested but **never pushed** by `ArchiveApp` on this branch.

**What T4 adds (verified in the `t4` worktree, commits `02550de9`/`6cf0d50f`):** Program v1.0.0
(`repo:ai/_inbox/PROGRAM_v1_0_0_playable_archive.md`) charters T4 as "Campaign runtime — ONE new
composition root": `repo:src/babylon/game/session.py` (**outside** `babylon.tui`) plus the named
`tui/app.py` change: "add Screen modes (lobby→briefing→campaign) + advance-tick binding." Two units:
- **Unit C1** (`02550de9`): `GameSession` — real 30-system `SimulationEngine` tick loop,
  `PostgresRuntime` lifecycle, crash-resume (`persist_tick`/`hydrate_graph` + `tick_commit`),
  `ArchiveTickBaker` as `TickCommitObserver`, per-tick event-bus collection. `cli/play.py` boots a real
  Wayne County campaign; the old demo survives as `play_demo()`.
- **Unit C2** (`6cf0d50f`): two new **optional** ctor params `campaign_menu: CampaignMenu | None`,
  `campaign_loader: CampaignLoader | None`; two new structural Protocols `CampaignHandle`
  (`session_id`, `tick`, `read_page(subject)`, `advance_tick() -> TickOutcome`) and `TickOutcome`
  (`tick`, `paused`) satisfied by `GameSession`/`TickAdvanceResult` without cross-import. `on_mount`
  branches: if `campaign_menu` given, `push_screen(LobbyScreen(...), callback=self._on_campaign_chosen)`;
  the callback calls `campaign_loader(campaign_id)`, sets `self._pages = campaign.read_page`, pushes a
  `BriefingScreen` (`Screen[bool]` rendering `VaultMaterializer.bake_briefing`'s previously-orphaned
  output through `BabylonMarkdown`); dismissing reveals the dossier shell reading the live vault. A new
  `Binding("t", "advance_tick", "Advance Tick", show=False)` calls `campaign.advance_tick()`.
  **Backward-compat (load-bearing for §IV.11):** `campaign_menu` defaults `None`, and with no
  `campaign_menu` `ArchiveApp()` is byte-identical to before; the new binding is `show=False` so the
  `Footer` pixels don't change either. `campaign_menu is not None and campaign_loader is None` raises
  `ValueError` at construction (Constitution III.11). **Not yet on `dev`** — treat as in-flight.

## IV.2 The fenced-directives-ONLY Markdown pipeline (`directives.py`)

`BabylonFence(MarkdownFence)` (`repo:src/babylon/tui/directives.py:369-554`) is the **one** subclass
Textual's `Markdown.BLOCKS` routes both `"fence"` and `"code_block"` tokens through (`app.py:127-130`).
Dispatch is on the fence's info string (`token.info`, exposed as `self.lexer`), never on paired
open/close tokens:
```python
def _directive(self) -> tuple[str,str] | None:
    match = DIRECTIVE_RE.match((self.lexer or "").strip())   # r"^\{(\w+)\}\s*(.*)$"
def compose(self) -> ComposeResult:
    directive = self._directive()
    if directive is None: yield from super().compose(); return   # ordinary fences highlight
    name, arg = directive
    method = getattr(self, f"_directive_{name}", None)
    if method is None:
        yield Label(f"▌ UNKNOWN DIRECTIVE {{{name}}} — refusing to render silently", classes="absence")
        return
    yield from method(arg)
```
**Paired container tokens are banned**: ADR099 (`repo:ai/decisions/ADR099_archive_stack.yaml:27-29`):
"the 8.2.8 token walker's generic close-pop breaks paired container tokens — re-confirmed in source." So
every directive is **one** fence dispatched inside `compose()` — a hard architectural constraint on any
future directive. **The loud unknown-directive law** (`method is None`) renders a crimson `.absence`
line, never a blank pane (Constitution III.11).

### The six directives

| Fence | Handler | Baked-body (III.13) | Live-provider fallback |
|---|---|---|---|
| `{statblock} <subject>` | `_directive_statblock` (395-436) | `key: value` lines, first-colon partition | `StatblockProvider(subject) -> Sequence[(str,str)]\|None` (`_StatblockHost`) |
| `{absence} <field> — <remedy>` | `_directive_absence` (438-446) | body/arg; both empty → diagnostic, never a bare dash | none |
| `{narrative} [cached:<tick>:<model_pin>]` | `_directive_narrative` (448-479) | body = cached prose; `NARRATIVE_CACHE_KEY_RE` byline | empty → `▌ no narration cached...` |
| `{paoh}` | `_directive_paoh` (481-487) | `parse_paoh_body`: `nodes:` + `tick: a,b,c` edges | n/a |
| `{maproom}` | `_directive_maproom` (489-505) | `parse_maproom_body`: `tier:` + `region:value` | always `render_tier="glyph"` (§IV.6) |
| `{egotree}` | `_directive_egotree` (507-518) | `topology/egotree.py::parse_egotree_body` | n/a |
| `{matrix}` | `_directive_matrix` (520-538) | `topology/matrix.py` incidence/adjacency parser | n/a |

Every dynamic string in a `Label(markup=True)` body is run through `textual.markup.escape` (imported
line 22): a lowercase-initiated bracket span (`"[unclear]"`) parses as a `Content` markup tag and is
silently dropped if unescaped — **because** `Label(markup=True)` builds a `textual.content.Content` via
`textual.markup.to_content`, not a Rich `Text` (Rich's escaping doesn't apply). Pinned by
`repo:tests/unit/tui/test_directives_hardening.py`. `DirectiveHover` (`Message`, lines 67-77) posts on
`on_enter`/`on_leave` (public names — the underscore forms are reserved by Textual and no base defines a
private hover hook at 8.2.8).

## IV.3 `wikilinks.py` — the inline rule

`wikilink_plugin(md, resolver)` (`repo:src/babylon/tui/wikilinks.py:72-106`) registers an inline rule
**before** `link` (`md.inline.ruler.before("link","wikilink",rule)`) matching `WIKILINK_RE =
r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]"`. A known target (per `resolver: Callable[[str], bool]`) emits
`wikilink_open/text/close` with `href = f"babylon://{target}"`; an unknown one emits `redlink_*` with
`href = f"babylon://redlink/{target}"` — the **redlink register**: a valid wikilink to an unknown entity
still parses and renders (in `REDLINK_COLOR = Color.parse(CRIMSON)`), pointing at `redlink/` so
`router.py::parse_babylon_uri` marks `redlink=True` and callers report `[REDLINK]` instead of
navigating (`app.py`'s `on_markdown_link_clicked`, lines 304-318).

Because Textual builds inline content via one monolithic `MarkdownBlock._token_to_content`
(verified: `textual/widgets/_markdown.py:281`, a single method called once from `compose` at line 278,
[resolved](#verification-ledger)) with "no finer extension seam," `WikilinkContentMixin._token_to_content`
(lines 250-277) is a **full reimplementation** — table-driven over `_INLINE_HANDLERS` (line 232) —
adding two entries: `wikilink_open`/`redlink_open` → `_wikilink_handler(color)` (211-225), which combines
a foreground `Style(foreground=color)` with the **same** `_click_style(href)` (`Style.from_meta({"@click":
f"link({href!r})"})`) upstream's `link_open` uses, so `Markdown.LinkClicked` fires natively for a
wikilink/redlink exactly as for a standard link — only the color register differs. Eight mixins are
pre-built for every inline-carrying block: `BabylonParagraph`, `BabylonH1`–`H6`,
`BabylonTableHeaderCell`/`TableDataCell` — wired into `BabylonMarkdown.BLOCKS` (`app.py:127-140`).
`make_parser_factory(resolver)` (109-126) matches Textual's default `MarkdownIt("gfm-like")` +
`front_matter_plugin` + the wikilink rule.

## IV.4 `chronicle.py` + `chronicle_salience.py`

`chronicle.py` renders `WorldState.events`-shaped `ChronicleEvent` records (frozen Pydantic:
`tick`/`event_type`/`summary`/`data`/`class_names`/`org_names`) into newest-first `TickBulletin`s
(`chronicle_stream`, 251-288; capped `CHRONICLE_ROW_CEILING=200`, mirroring
`PostgresRuntime.query_session_events(limit=200)`, spec-092). `resolve_actor` (211-243) is a **port,
not a re-derivation** of `web/game/narrator.py::_subject_from_class_id`/`_subject_from_org_id` — the
docstring (14-22) states `narrator.py` is never imported (Constitution III boundary; `web/` is legacy);
the canonical `_CLASS_ID_NAMES` (C001–C006) and per-event field tables are copied verbatim. An event
with no attributable actor resolves `None`, never a fabricated placeholder. A quiet tick renders
`"the wire is quiet"` (`_WIRE_QUIET`, line 72).

`chronicle_salience.py` is WO-48's **net-new** layer (pure functions run *before*
`chronicle_stream`/`bulletin_for_tick`):
- **`EVENT_SEVERITY`** (104-155): `Final[dict[str, SeverityTier]]` ported verbatim from
  `web/game/engine_bridge.py::_EVENT_SEVERITY` (spec-061 FR-012) — 47/84 `EventType` values (14 critical
  / 20 warning / 13 informational). `classify_event_salience` (180-195) resolves an unmapped `EventType`
  to **`"warning"`**, `unclassified=True` — the docstring (27-35) documents this as the deliberate fix
  of a ported bug (legacy defaulted unrecognized types to "informational," which Constitution III.11
  forbids; `src/frontend/src/lib/eventClassifier.ts` had a second instance via UPPERCASE keys matching
  nothing). Both halves fixed at root.
- **`dedupe_consecutive`** (264-292) + **`dedup_key`**/`chronicle_subject` (198-261) port
  `src/frontend/src/lib/eventDedup.ts` (spec-116 FR-116-2): collapse only *consecutive*
  `f"{event_type}:{subject}"` runs, keep the first, tick-independent.
- **Autopause**: `compute_autopause_state` (311-331) fires `AutopauseState(active=True, token=AMBER)`
  iff any event classifies `"critical"` — a deliberately **partial** port (the frontend's
  session-scoped once-per-`(event_type, subject)` ack set is NOT ported, left for WO-46's `babylon_meta`).
  `render_autopause_indicator` (334-345) returns `None` (never a dimmed placeholder) when inactive.
- **Volume floors** (ADR086): `cap_narrative_events` (357-379) caps only the **informational** tier to
  `NARRATIVE_EVENT_CEILING_PER_TICK=1`; `aggregate_organizational_actions` (382-420) collapses every
  `ORGANIZATIONAL_ACTION` in a tick into one count-bearing rollup (idempotent). The two floors touch
  disjoint event types (`ORGANIZATIONAL_ACTION` isn't an `EVENT_SEVERITY` key → unclassified-`warning`,
  never `NARRATIVE_TIER`), so `apply_volume_floors` (423-439) composes them in either order.

## IV.5 `verb_plate.py`

`render_verb_plate(view: VerbPlateView) -> RenderableType` (150-186) renders Constitution Article V's
nine verbs as one bordered Rich `Panel`, Investigate **expanded** to its three named sub-verbs
(`INVESTIGATE_SUB_VERBS = ("Territory","Org","Edge")`, line 67) — the docstring (28-42) is explicit the
view-model doesn't yet carry per-sub-verb eligibility/cost/preview, so all three inherit the parent
row's signal (honest description, not a fabricated split). `_status_text` (77-91) enforces "the UI
disables on `eligible` only, never on `can_afford`" — an eligible-but-unaffordable verb renders `✓
legal` with a dim affordability note, never hidden. A canonical verb absent from `view.verbs` renders
`_missing_verb_line` (137-147), a bold-crimson `▌ {verb} — missing from plate view` (Article V's "always
available" ⇒ a missing row is a caller bug). Fixture-fed today (WO-26); the live provider
`babylon.projection.verbs.plate.build_verb_plate` (WO-38) is a drop-in `VerbPlateView` source —
`verb_plate.py` never imports the graph/engine/DB.

## IV.6 `map_room.py` + the ADR097 three-tier render seam

`render_map_room(cells, *, render_tier: RenderTier) -> MapRoomWidget` (120-142) builds **one**
deterministic `PIL.Image.Image` via `build_choropleth_image` (90-117; pure, no wall-clock/randomness)
and wraps that **same** bitmap in one of two `textual-image` widgets:
```python
MapRoomWidget = HalfcellImage | TGPImage        # type-alias union; module-level Image var NOT an annotation
widget: MapRoomWidget = TGPImage(image) if render_tier == "pixel" else HalfcellImage(image)
```
Implements ADR097 (`repo:ai/decisions/ADR097_render_seam.yaml`) D1+D2: **Tier 0 (glyph, `HalfcellImage`)
is the design target, not the fallback** — the default Debian terminal (GNOME/VTE) has no pixel
protocol, so "everything the game ever communicates is communicated in this tier." **Tier 1 (pixel,
`TGPImage`, Kitty-graphics) never carries unique information — it re-renders Tier 0**, true *by
construction* (one bitmap, two wrappers). `render_tier` is caller-supplied, never runtime-probed here
(D4's `babylon doctor` probe is separate future work); the `{maproom}` directive always calls with
`render_tier="glyph"` today. `_band_color` (70-87) maps `s/v` to a **hard-cut** band color
(`DIM`/`GOLD`/`CRIMSON` at 1.0/2.0), never a gradient — DESIGN_BIBLE §11 "qualitative change is a hard
cut, never tweened." Tier 2 (SVG export) is Textual's native mechanism (Constitution III.13, ADR097 D3)
and is how the snapshot-golden estate (§IV.11) works. Textual-image mechanism → **Part II §9**.

## IV.7 `theme.py` — KSBC tokens vs `DESIGN_BIBLE` §9b (verified drift)

`theme.py` is the **single source of truth other `babylon.tui` modules import from** (docstring 1-10,
citing owner ruling 2026-07-11: the Archive's chrome follows the Guix installer anatomy in the owner's
Kitty `ksbc-new` palette; full table `repo:project/research/16-living-map/DESIGN_BIBLE.md` §9b 292-331).

**Verified drift (not previously flagged, no test catches it):**

| Role | DESIGN_BIBLE §9b | `theme.py` constant | Match? |
|---|---|---|---|
| field (dead space) | `#1a0000` | `FIELD = "#1a0000"` | yes |
| text | `#e8e8e8` | `BONE = "#e8e2d4"` | **no** |
| accent-crimson | `#dc143c` | `CRIMSON = "#dc143c"` | yes |
| accent-gold | `#ffd700` | `GOLD = "#ffd700"` | yes |
| selection text | `#000000` | `"block-cursor-foreground": "#000000"` | yes |
| muted | `#404040`/`#c0c0c0`/`#202020` | `DIM = "#8f8778"` | **no** (matches none) |

For contrast, the ADR097-era `repo:src/babylon/render/tiers.py` (`TRUECOLOR_PALETTE[RoleToken.TEXT] ==
"#e8e8e8"`) gets it right and is parity-tested against the design bible
(`repo:tests/unit/render/test_tiers.py:50`; also `repo:tests/unit/render/test_design_bible_parity.py`,
verified to exist — it asserts `rows == expected` where `expected` maps `RoleToken` →
`(TRUECOLOR_PALETTE[token], DEGRADED_256_PALETTE[token])` parsed from §9b, **covering the `RoleToken`
dict, NOT `tui/theme.py`'s `BONE`/`DIM`**, [resolved](#verification-ledger)). `tui/theme.py` is a
**second, independently-hardcoded** copy of the same design row (sole commit `0b1905db`, WO-5) that
nothing cross-checks — every `babylon.tui` module importing `BONE`/`DIM` (`chronicle.py`, `peek.py`,
`verb_plate.py`, `watchlist.py`, `chronicle_salience.py`) is one step from the ratified palette.
`AMBER = "#ff8c00"` (autopause, reserved) and `PANEL = "#200404"` have no §9b row — extensions, not drift.

## IV.8 The Protocol-persistence seams — the "WO-37 trick"

Three `tui/` modules each define a `@runtime_checkable` `Protocol` whose method names/signatures exactly
mirror a concrete `babylon.persistence` class — without either importing the other, so the import-linter
contract (§IV.9) never learns persistence exists:

| Seam (`tui/`) | Protocol | Satisfier | Status |
|---|---|---|---|
| `nav.py:169-209` | `NavPersistence` (`load_jumplist`/`save_jumplist`/`load_breadcrumbs`/`save_breadcrumbs`) | `babylon.persistence.babylon_meta.BabylonMetaStore` | **closed** — all four at `babylon_meta.py:260-293` |
| `campaign_menu.py:80-121` | `CampaignCatalog` (`list_campaigns`/`create_campaign`/`set_status`/`delete_campaign`) | `BabylonMetaStore` | **closed** — `babylon_meta.py:163-224` |
| `watchlist.py:177-207` | `WatchlistPersistence` (`load(session_id)`/`save(session_id, ids)`) | `BabylonMetaStore` | **closed** — `babylon_meta.py:230-256` |
| `app.py` (t4 only) | `CampaignHandle`/`TickOutcome` | `babylon.game.session.GameSession`/`TickAdvanceResult` | **closed in t4**, not on `dev` |

Each `tui/` module ships its own `InMemory*` fake (`InMemoryNavPersistence`, `InMemoryCampaignCatalog`,
`InMemoryWatchlistPersistence`) as **both** the honest no-DB default (state dies with the process) and
the test double — `repo:tests/unit/tui/test_nav_shell.py`/`test_campaign_menu.py` verify the seam via
`BabylonMetaStore.__new__(BabylonMetaStore)` (no `__init__`) + `isinstance(store, <Protocol>)`.

**Stale-docstring gotcha**: `watchlist.py`'s module docstring (15-20) still says, present tense, "that
store does not exist yet, and this WO is explicitly barred from creating any DB table." No longer true —
`babylon_meta.py`'s docstring (line 15) states it "structurally satisfies
`babylon.tui.watchlist.WatchlistPersistence`," and `_NAV_TABLES` includes `"watchlist"`
(babylon_meta.py:52); WO-46 has landed. Read the "does not exist yet" framing as historically accurate,
currently stale; the Protocol shape is unaffected.

`palette.py`'s `EntityNavigatorProvider` and `directives.py`'s `_StatblockHost` use the same
structural-duck-typing idiom one layer down (an `App`/`Markdown` exposing `known_entities`/`statblocks`,
checked via `isinstance(x, _KnownEntityHost)`) — same mechanism, not a persistence seam.

## IV.9 The import-linter contract and the composition-root pattern

`repo:pyproject.toml:637-645`:
```toml
[[tool.importlinter.contracts]]
name = "tui client reads projections only — no engine, persistence, or legacy web stack"
type = "forbidden"
source_modules = ["babylon.tui"]
forbidden_modules = ["babylon.engine", "babylon.persistence", "django"]
```
Enforced by `mise run lint:imports` (CI-gated). Every §IV.8 seam exists **because** of this: `babylon.tui`
can't import `BabylonMetaStore` to type-hint against, so it defines a matching-shape `Protocol`, and
something **outside** `babylon.tui` — the composition root — does the wiring. Pre-T4 that root didn't
exist; T4's `repo:src/babylon/game/session.py` (t4 worktree, not on `dev`) is chartered as "ONE new
composition root" outside both `babylon.tui` and `babylon.projection` — the module allowed to import
engine, persistence, *and* tui and glue them.

## IV.10 The golden-vault relationship

Two artifact families, not to be confused:
1. **Vault manifests = content goldens, ARE a ceremony.** `VaultMaterializer`
   (`repo:src/babylon/projection/vault/materializer.py`) bakes projection view-models to Markdown via
   per-kind `render_*` (`projection/vault/render*.py`) and commits through `git_backend.py`'s dulwich
   wrapper with a **sim-time-pinned** commit timestamp (ADR099 item 5: two builds → identical shas).
   The byte-gate (`repo:tests/integration/archive/test_vault_golden.py`, `tools/vault_regression.py`)
   compares a fresh bake's `manifest.json` (path→sha256) against the committed baseline under
   `single_county`; drift is a `Baselines: blessed(<slug>)` ceremony (root `CLAUDE.md` §6.5), never a
   silent update.
2. **Textual SVG snapshots = render goldens, NEVER a ceremony.** Every `*_app.py` under
   `repo:tests/unit/tui/snapshots/`/`tests/integration/archive/` produces an SVG via `snap_compare`,
   compared against a syrupy golden. Program v1.0.0 (`repo:ai/_inbox/PROGRAM_v1_0_0_playable_archive.md:
   253`): "re-bakes ALL Textual snapshots — snapshots are render SVGs, regenerate freely, NOT a
   ceremony." Delete-and-regenerate → byte-identical SVGs (sha256 double-bake) is itself the III.13
   determinism proof ADR099 cites.

The vault (content) is content-hash-gated like any economics baseline; the SVG (pixels) is a rendering
regression check that reuses the vault's baked pages as fixture input for ~16 of ~24 dossier goldens
(§IV.11).

## IV.11 Known gotchas (cross-referenced against source)

- **`ArchiveApp` layout change re-bakes ALL snapshot goldens whose launcher builds a real
  `ArchiveApp`.** 16 of `repo:tests/unit/tui/snapshots/*_app.py` (`app_dispatch_app.py`,
  `national_page_app.py`, `org_page_app.py`, `state_page_app.py`, `institution_snapshot_app.py`,
  `snapshot_sovereign_app.py`, `industry_snapshot_app.py`, `epilogue_snapshot_app.py`,
  `paoh_snapshot_app.py`, `egotree_snapshot_app.py`, `directives_hardening_app.py`,
  `community_snapshot_app.py`, `concept_card_snapshot_app.py`, `map_room_snapshot_app.py`,
  `matrix_snapshot_app.py`, `key_figure_snapshot_app.py`) construct a real `ArchiveApp(page=...)`,
  inheriting its full `CSS`/`compose()` — changing `ArchiveApp.CSS` or its chrome reflows every one. The
  other widget-level goldens (`chronicle_app.py`, `watchlist_app.py`, `verb_plate_app.py`,
  `peek_plate_app.py`, `lobby_snapshot_app.py`, `chronicle_salience_app.py`, `palette_snapshot_app.py`)
  build a bare `App[None]` host — insulated from `ArchiveApp` layout but **not** from `theme.py`
  hex-constant changes (they import `CRIMSON`/`GOLD`/`BONE` into Rich styling, not through CSS
  `$variables`). Exactly why T4 C2 keeps the no-`campaign_menu` path byte-identical and binds `t` with
  `show=False`.
- **Trailing-whitespace hook vs every SVG golden** — §III.3.4 item 3; a scoped `conftest.py` autouse
  monkeypatch strips it before comparison.
- **`NO_COLOR` vs snapshot goldens.** Repo-wide mise `[env]` exports `NO_COLOR=1`; Textual desaturates
  every theme color to grayscale even under the headless harness. `repo:tests/unit/tui/conftest.py`'s
  autouse `_truecolor_snapshots` fixture `monkeypatch.delenv("NO_COLOR")` so goldens stay sensitive to
  ksbc regressions and mise/CI and a direct venv pytest render the same bytes.
- **`MarkdownFence`'s walker forces fenced-directives-only** — §IV.2; the single reason there is one
  `BabylonFence` class instead of paired-token container types.
- **The Jinja asymmetry law** (`repo:ai/_inbox/PROGRAM_v1_0_0_playable_archive.md:159-161`): "sandboxed
  Jinja for deterministic pages ONLY; LLM prose via plain string building, never a template engine." The
  page (headers/statblocks/wikilinks) is Jinja-templated at bake time (ADR099 item 3,
  `ImmutableSandboxedEnvironment` + `StrictUndefined`), but a `{narrative}` fence body is written as a
  plain string dropped in verbatim — which is what makes `_directive_narrative`'s escaping discipline
  (§IV.2) load-bearing.
- **`theme.py` vs `DESIGN_BIBLE` §9b drift** — §IV.7; `BONE`/`DIM` don't match, no test catches it.
- **`watchlist.py` module docstring is stale** re: WO-46 — §IV.8.
- **`LobbyScreen` was orphaned prior to T4** — built + unit-tested (`campaign_menu.py:322`) but never
  `push_screen`'d on `dev`; T4 C2 is the fix (in the t4 worktree).

---

# Part V — Cross-map: Babylon concern → Textual mechanism → our file

Each row is a durable seam between a Babylon design requirement and the Textual primitive it rides on.

| Babylon concern | Textual mechanism (Part cite) | Our file(s) |
|---|---|---|
| **Pacing / tick driver** | `@work` workers + `set_timer`/`set_interval` — all wall-clock, skip-under-load; sim clock must be its own counter, advanced from (not represented by) a timer (Part I §8, §9.5) | `repo:src/babylon/game/session.py` (`GameSession.advance_tick`, t4); `ArchiveApp` `Binding("t","advance_tick")` (Part IV §4.1) |
| **BDD / golden assertions** | `App.run_test()` → `Pilot`; capture via `export_screenshot` (SVG) or `Console(record=True)`+`export_text()`; per-widget `Strip.text`/`render_lines` (Part III §3.1, Part II §10) | `repo:tests/unit/tui/snapshots/*_app.py`, `repo:tests/integration/archive/test_vault_golden.py`; `snap_compare` fixture |
| **Autopause on severity** | custom `Message` from a severity-classifying consumer; `post_message` (thread-safe); reactive-driven repaint (Part I §5.4, §4) | `repo:src/babylon/tui/chronicle_salience.py` (`compute_autopause_state`, `AMBER`); `render_autopause_indicator` |
| **Lobby → briefing → campaign flow** | `Screen` stack / `push_screen(callback=...)` + `Screen[bool]` dismiss; `MODES` for independent stacks (Part I §2.3, §2.4) | `repo:src/babylon/tui/campaign_menu.py` (`LobbyScreen`); `ArchiveApp.on_mount` branch + `BriefingScreen` (Part IV §4.1) |
| **KSBC aesthetic** | `textual.theme.Theme` dataclass + `ColorSystem.generate()` → `$`-variables + `theme.variables` overrides; register-before-activate (Part II §5) | `repo:src/babylon/tui/theme.py` (`KSBC`); `ArchiveApp.on_mount` `register_theme(KSBC); self.theme="ksbc"` |
| **Raster lane (map room)** | `textual-image` tiers: `HalfcellImage` (glyph, primary) / `TGPImage` (Kitty pixel); detect-before-boot; one bitmap, two wrappers (Part II §9) | `repo:src/babylon/tui/map_room.py` (`render_map_room`); `repo:src/babylon/render/capability.py` (`probe`) |
| **Wikilink / redlink navigation** | monolithic `MarkdownBlock._token_to_content` reimplemented; `Markdown.LinkClicked` via `Style.from_meta({"@click": ...})` (Part IV §4.3) | `repo:src/babylon/tui/wikilinks.py`; `router.py`; `ArchiveApp.on_markdown_link_clicked` |
| **Fenced directives (statblock/paoh/…)** | `MarkdownFence` subclass dispatched on `token.info`; paired container tokens banned at 8.2.8 (Part IV §4.2) | `repo:src/babylon/tui/directives.py` (`BabylonFence`) |
| **Loud unknown / absence rendering** | `Label(markup=True)` → `Content` via `textual.markup.to_content`; `textual.markup.escape` mandatory (Part IV §4.2) | `directives.py` unknown-directive law; `verb_plate.py` `_missing_verb_line`; `chronicle.py` `_WIRE_QUIET` |
| **Vim jumplist / nav** | `BINDINGS` (`ctrl+o`/`ctrl+i`) → `action_*`; `check_action` for conditional bindings (Part I §6.1, §6.3) | `repo:src/babylon/tui/nav.py`; `ArchiveApp.BINDINGS` |
| **Fuzzy entity switcher** | command palette `Provider.search` (a `@work(exclusive=True)` gather); `App.COMMANDS` (Part I §7) | `repo:src/babylon/tui/palette.py` (`EntityNavigatorProvider`) |
| **Persistence without import coupling** | `@runtime_checkable Protocol` shape-mirroring a concrete class; composition root does the wiring (import-linter contract) | `nav.py`/`campaign_menu.py`/`watchlist.py` Protocols; `repo:src/babylon/game/session.py` root (Part IV §4.8, §4.9) |
| **Cross-thread engine → UI** | `post_message` (auto-marshals foreign thread) / `App.call_from_thread`; never mutate reactives off-thread (Part I §5.4) | `GameSession` event-bus collection → `ArchiveTickBaker` (Part IV §4.1) |
| **Deterministic render golden** | Rich `export_svg` Adler-32 id + plugin `normalize_svg`; sha256 double-bake (Part III §3.3.4) | `repo:tests/unit/tui/snapshots/conftest.py`; `snap_compare` estate |
| **Text-only export tier (no `export_text`)** | reuse `Console(record=True)` + compositor `render_update` + `export_text()`; or `Strip.text` (Part II §10, Quick ref §0.5) | (new code — no repo helper exists yet) |

---

## Verification ledger

Every `[UNVERIFIED]` flag from the four source drafts and its disposition this pass. "RESOLVED" items
were checked against installed source on 2026-07-21 and their claims promoted to verified in the body;
"KEPT" items remain `[UNVERIFIED]` for the stated reason.

| # | Draft flag | Disposition | Evidence / reason |
|---|---|---|---|
| 1 | `App.action_toggle_dark` — is there a default key binding? (Part II §5) | **RESOLVED** — no. | `App.BINDINGS` (app.py:454-464) has only `ctrl+q`→quit (priority) and `ctrl+c`→help_quit (system); `action_toggle_dark` (app.py:4545) is an unbound named action. |
| 2 | `"_"` as a "short pause" token in `press`/`snap_compare` (Part III §3.1.3) | **RESOLVED** — not engine behavior. | `_press_keys` (app.py:2050-2068): `"_".isalnum()` is `False` → `_character_to_key("_")`, an ordinary char key. The pause meaning is snapshot-plugin convention only. |
| 3 | `TEXTUAL_THEME` comma-list "first that exists" resolution (Part III §3.5.1) | **RESOLVED.** | `App.get_theme` (app.py:1449-1453) splits on `,`, strips, returns first in `available_themes`; `current_theme` (app.py:1487) calls it. |
| 4 | Rich `color_system="auto"` / COLORTERM trigger (Part III §3.5.3) | **RESOLVED** (corrected). | `rich/console.py:707-712`: resolved in `Console.__init__` (not "first render"): `elif color_system=="auto": self._color_system=self._detect_color_system()`; `_detect_color_system` reads `COLORTERM` (console.py:805). Textual passes default `"auto"`, so it fires at construction. |
| 5 | `SIGCONT` / resume-from-`SIGTSTP` mechanism (Part III §3.7) | **RESOLVED** — explicit handler exists. | `drivers/linux_driver.py:78-79` installs `SIGTSTP`→`_sigtstp_application` and `SIGCONT`→`_sigcont_application`; resume posts `SignalResume()` (linux_driver.py:311) → `App` `@on(Driver.SignalResume)` (app.py:4712). |
| 6 | `_token_to_content` monolithic reimplementation target (Part IV §4.3) | **RESOLVED.** | `textual/widgets/_markdown.py:281` — `MarkdownBlock._token_to_content(self, token) -> Content`, a single method called once from `compose` (line 278). Confirms the full-reimplementation need. |
| 7 | `test_design_bible_parity.py` contents (Part IV §4.7) | **RESOLVED.** | `repo:tests/unit/render/test_design_bible_parity.py` exists; asserts `rows == expected` mapping `RoleToken` → `(TRUECOLOR_PALETTE[t], DEGRADED_256_PALETTE[t])` parsed from §9b — covers `render/tiers.py`'s dict, **not** `tui/theme.py`'s `BONE`/`DIM`, so the §4.7 drift is genuinely uncaught. |
| 8 | Exact Sixel-supporting terminal list (Part II §9) | **KEPT.** | External terminal capability, not enumerated in installed `textual-image` source. |
| 9 | `TEXTUAL_DEVTOOLS_PORT` actual consumption (Part III §3.4, §3.5.1) | **KEPT.** | Consumed inside `textual_dev.client.DevtoolsClient`; `textual-dev` not installed in this venv. Only the constant's existence and its non-use in core are verified. |
| 10 | Which `TERM`/multiplexer combos drop Kitty-protocol / sync-mode (Part III §3.6) | **KEPT.** | No allowlist/denylist in source; degradation is inferred ("unacknowledged escape = silent fallback"), not a verified compatibility matrix. Needs live-terminal testing. |
| 11 | Whether T4's `t4` worktree merges to `dev` as-is (Part IV §4.1, §4.12) | **KEPT.** | Future/process fact, unknowable from source; §4.1 is a snapshot of in-flight work. |
| 12 | Driver raw-input→`events.Key` path; eager-task-factory scheduling; `Pilot._wait_for_screen` internals (Part I open questions) | **KEPT (scope).** | Driver input internals not read this pass; `_wait_for_screen` is described to the depth Part III §3.1.2 needs. Out of scope for the sections as drafted. |

---

*End of manual. Source drafts: `01-core-events.md`, `02-style-render.md`, `03-testing-config.md`,
`04-our-idioms.md`. Composed and deduplicated 2026-07-21; not committed (controller commits).*
