# Textual Docs Digest — Babylon Archive TUI (Program 24)
Pin: textual==8.2.8. Docs-site claims cite textual.textualize.io URLs. Where the live docs
were silent/ambiguous, I cross-checked the pinned source at
github.com/Textualize/textual tag `v8.2.8` — those are marked **[SOURCE, not docs]** and are
NOT authoritative documentation, just verified fact about our exact pin.

## 1. Events & messages — bubbling, naming, @on, custom Message, private handlers

- Bubbling: "Messages with `bubble` attribute set to `True` propagate to parent widgets after
  processing... Event handlers may stop this bubble behavior by calling the `stop()` method on
  the event or message." — https://textual.textualize.io/guide/events/
- Naming convention: prefix `on_`, then namespace (parent class, snake_case) then message class
  snake_case, e.g. `ColorButton.Selected` → `on_color_button_selected`. Verify via
  `Input.Changed.handler_name == 'on_input_changed'`. — same page.
- `@on` decorator: "turns a method into a handler for the given message or event... you can
  specify which widget(s) you want to handle messages for" via CSS selector; "If multiple
  decorated handlers match the message, then they will _all_ be called in the order they are
  defined." — https://textual.textualize.io/guide/events/
- `Message.control` (property): "The widget associated with this message, or None by default."
  `stop(stop=True)`: "Stop propagation of the message to parent." `prevent_default(prevent=True)`:
  "Suppress the default action(s). This will prevent handlers in any base classes from being
  called." — https://textual.textualize.io/api/message/
- Verbosity/dispatch class vars **[SOURCE, not docs — not on the API page]**:
  `bubble: ClassVar[bool] = True`, `verbose: ClassVar[bool] = False`,
  `no_dispatch: ClassVar[bool] = False`, `namespace: ClassVar[str] = ""` —
  github.com/Textualize/textual/blob/v8.2.8/src/textual/message.py

### Private `_on_enter`/`_on_leave` vs public `on_enter`/`on_leave`
- The events guide documents **only two** sanctioned handler mechanisms: the bare
  `on_<namespace>_<message>` naming convention and the `@on` decorator. It says nothing about a
  leading-underscore variant as a user-facing API. — https://textual.textualize.io/guide/events/
- **[SOURCE, not docs]** Actual dispatch code (`MessagePump._get_dispatch_methods`,
  message_pump.py @ v8.2.8):
  ```python
  # Fall back to the naming convention
  # But avoid calling the handler if it was decorated
  method = cls.__dict__.get(f"_{method_name}") or cls.__dict__.get(method_name)
  ```
  For **each class in the MRO**, Textual looks up `_on_enter` before falling back to `on_enter`
  *at that class's own `__dict__`* — this is how Textual's own base classes (`Screen`, `App`)
  intercept a message internally (`_on_mount`, `_on_resize`, `_on_idle`, `_on_screen_resume`,
  `_on_css_change`, confirmed present in screen.py/app.py @ v8.2.8) while leaving the
  underscore-free name free for **user** subclasses to override without collision. Both a base
  class's private handler and a subclass's public handler fire (dispatch walks the whole MRO).
- **Critically: `_on_enter`/`_on_leave` are NOT defined anywhere in `Widget`, `DOMNode`,
  `MessagePump`, `Screen`, `App`, `Static`, `MarkdownBlock`, or `MarkdownFence` at v8.2.8**
  (all six source files checked directly). There is no Textual-declared base method at that name
  for a subclass to legitimately "override" today.
- **Confirmed live precedent**: `Markdown` itself defines a **private**
  `_on_markdown_link_clicked` (real work: auto-navigation when `open_links=True`) *and* a
  **public no-op** `on_markdown_link_clicked(self, event: LinkClicked) -> None` stub for users
  to override. **[SOURCE]** widgets/_markdown.py @ v8.2.8. This confirms
  `ArchiveApp.on_markdown_link_clicked` (public, no underscore) is exactly the sanctioned
  override point.
- **Recommendation**: switch `MarkdownFence`-subclass hover code from `_on_enter`/`_on_leave` to
  public `on_enter`/`on_leave`. Reasons: (1) only mechanism the docs describe as user-facing;
  (2) `@on`-decorator compatible, the private form is not; (3) removes reliance on an
  internal-only convention that could silently start double-firing if a future Textual release
  adds its OWN `_on_enter` to `Widget` for hover-delay/tooltip bookkeeping (plausible —
  `Widget.mouse_hover` reactive already exists); (4) resolves the Pyright override complaint.

## 2. Enter/Leave semantics for hover

- "Textual will send a Enter event to a widget when the mouse cursor first moves over it, and a
  Leave event when the cursor moves off a widget. Both Enter and Leave bubble, so a widget may
  receive these events from a child widget." — https://textual.textualize.io/guide/events/
- `Enter`: bubbles; "Check the `node` attribute for the widget directly under the mouse." —
  https://textual.textualize.io/events/enter/
- `Leave`: "Sent when the mouse is moved away from a widget, or if a widget is programmatically
  disabled while hovered... Check the `node` parameter for the original widget that was
  previously under the mouse." Bubbles: Yes. — https://textual.textualize.io/events/leave/
- Documented hover idiom: `on_enter()` → set a reactive True, `on_leave()` → False, watcher
  updates appearance. — https://textual.textualize.io/guide/events/ (Hover example).
- **[SOURCE]** The actual reactive in v8.2.8's `Widget` is **`mouse_hover`** (`Reactive[bool]`,
  "Is the mouse over this widget? Read only."), plus property `is_mouse_over` (True even when a
  child widget is between pointer and self). — https://textual.textualize.io/api/widget/
  **Naming-drift caveat**: guide prose says `mouse_over`; the 8.2.8 API name is `mouse_hover`.
- `can_focus`: class attribute, default `False`. `allow_focus()` overridable. — /api/widget/
- Hit-testing: **[SOURCE]** `Screen._handle_mouse_move` → `get_hover_widgets_at(x, y)` manages
  Enter/Leave transitions — internal plumbing, do not hook.
- Gap: no documented delayed-hover ("peek plate after N ms") idiom — build on `Timer` +
  `on_enter`/`on_leave` yourself.

## 3. Markdown widget extension surface (8.x)

- `BLOCKS: dict[str, type[MarkdownBlock]]` maps token names ("h1".."h6", "hr", "paragraph_open",
  "blockquote_open", "bullet_list_open", "ordered_list_open", "table_open", "fence",
  "code_block", …) to widget classes; overriding it is the documented extension point. —
  https://textual.textualize.io/widgets/markdown/ (+ confirmed in source).
- Class hierarchy **[SOURCE]**: `MarkdownBlock(Static)`, `MarkdownFence(MarkdownBlock)`,
  `Markdown(Widget)`.
- `LinkClicked`: posted on link click; `href: str` (unquoted), `markdown` widget ref, `control`
  aliases `markdown` for the `@on` decorator. — /widgets/markdown/ (+ source).
- `Markdown.__init__` signature **[SOURCE, not on the docs page]**:
  `(self, markdown=None, *, name=None, id=None, classes=None, parser_factory=None,
  open_links=True)`.
  - `open_links: bool = True` — "Open links automatically. If you set this to `False`, you can
    handle the `LinkClicked` events."
  - `parser_factory` — returns a configured MarkdownIt; `None` → "gfm-like" parser.
  - **`code_dark_theme`/`code_light_theme` do NOT exist as constructor parameters in 8.2.8.**
    Version caveat — do not assume they exist on this pin.
- `open_links` decision for the `babylon://` router (P2): either keep `open_links=True` and only
  use `on_markdown_link_clicked` for non-URL (`babylon://`) hrefs (Textual's internal handler
  auto-opens real URLs), or set `open_links=False` and handle everything. Docs frame these as
  mutually exclusive modes. Make the choice explicit.

## 4. Themes

- "A theme is a simple Python object which maps variable names to colors." Register via
  `App.register_theme` in `on_mount`, then `self.theme = '<name>'`. —
  https://textual.textualize.io/guide/design/ **`on_mount` is the documented lifecycle point.**
- `Theme` fields: `name`, `primary`, `secondary`, `accent`, `foreground`, `background`,
  `surface`, `panel`, `boost`, `warning`, `error`, `success`, `dark` (bool), `variables` (dict).
  Only `primary` is required; Textual generates the rest if absent.
- Variables: 11 base colors → CSS vars `$primary` etc., plus `-lighten-1/2/3`/`-darken-1/2/3`
  and `-muted` variants. Custom app-level vars: override `App.get_theme_variable_defaults()`
  (theme's `variables` wins on collision).
- `https://textual.textualize.io/api/theme/` 404s — guide/design/ is the canonical source.
- ANSI fallback: only on the FAQ. `App.ansi_color: bool` (default False) — Textual REPLACES the
  16 ANSI colors with resolved truecolor; `ansi_color=True` trusts the terminal's palette but
  loses transparency/blending. **No ANSI-256-specific fallback story exists in the docs** —
  empirical testing required for target terminals. — https://textual.textualize.io/FAQ/

## 5. Key bindings, modes, command palette (design surface for Lane W / P2)

- `BINDINGS`: list of (key, action, description) tuples; comma-separate multiple keys. `Binding`
  class adds `show=False`, `priority=True` (checked before focused-widget bindings). —
  https://textual.textualize.io/guide/input/
- Precedence: **focused widget → ancestors up the DOM → App**, `priority=True` first at each
  level. Bind vim-ish j/k on the content-owning widget so focus determines who answers.
- Modes: `MODES` class var (name → screen/callable/installed-name), `DEFAULT_MODE`,
  `App.switch_mode()`; "Any calls to `App.push_screen` or `App.pop_screen` will affect only the
  active mode" — per-mode stack isolation. Natural fit for READ/PEEK/VERB. —
  https://textual.textualize.io/guide/screens/
- Command palette `Provider`: register via `COMMANDS = App.COMMANDS | {YourProvider}`. Four
  coroutines: `startup()` (once at palette open), `search(query) -> Hits` (**only required**;
  `self.matcher(query)` → score > 0 = match, 1 = exact), `discover()` (empty-input suggestions,
  must be fast), `shutdown()`. Provider errors are logged, not fatal. —
  https://textual.textualize.io/guide/command_palette/

## 6. Testing — run_test, snapshots, determinism

- `run_test()`: headless, returns async-context `Pilot`; default size 80×24, override
  `app.run_test(size=(100, 50))`. — https://textual.textualize.io/guide/testing/
- `pilot.pause()`: waits for all pending messages; optional `delay`. `pilot.click()`: CSS
  selector, `offset`, `times=2`, modifier flags. `pilot.hover()`: selector-driven, usable in
  `run_before`.
- Snapshot testing: `snap_compare(app_or_path, press=[...], run_before=..., terminal_size=...)`.
  First run generates the SVG baseline and fails; update via `--snapshot-update` "only ever...
  if you're happy with how the output looks."
- **Determinism caveat (plugin README only, not the guide)**: `run_before` example calls
  `await disable_blink_for_active_cursors(pilot)` — cursor blink is a known SVG-golden flakiness
  source. — https://github.com/Textualize/pytest-textual-snapshot
  **Gap**: no general animation/timer/worker determinism guidance — animated peek plates or mode
  indicators need custom `run_before` neutralization.
- Workers/threads: never call UI methods or set reactives from a threaded worker directly;
  `post_message` is thread-safe; use `App.call_from_thread` when you must. —
  https://textual.textualize.io/guide/workers/ Relevant to any async P2 feature racing a
  snapshot test.

## 7. Reactive attributes vs direct `.update()` in message handlers

- Watch methods fire on reactive modification — preferred idiom for state many parts of the UI
  derive from. — https://textual.textualize.io/guide/reactivity/
- Pre-mount hazard: watchers that query the DOM break if triggered from `__init__`; use
  `set_reactive` there instead.
- Mutable-collection gap: in-place list/dict mutation does NOT fire watchers — call
  `mutate_reactive` explicitly. Relevant to a future watchlist reactive.
- `always_update=True` re-fires watchers on same-value assignment (mode indicator re-entry);
  `recompose=True` is the alternative to manual watcher plumbing for structural changes.
- Plain `label.update(...)` inside a message handler is a normal, supported idiom — the docs
  never frame it as unsafe. The genuine documented hazard is the cross-thread case (§6).

## Sources consulted
- https://textual.textualize.io/guide/events/ , /api/events/ , /events/enter/ , /events/leave/
- https://textual.textualize.io/api/message/ , /api/widget/ , /widgets/markdown/
- https://textual.textualize.io/guide/design/ , /FAQ/ , /guide/input/ , /guide/screens/
- https://textual.textualize.io/guide/command_palette/ , /guide/testing/ , /guide/workers/
- https://textual.textualize.io/guide/reactivity/ , /api/style/ , /guide/content/
- https://github.com/Textualize/pytest-textual-snapshot (README)
- github.com/Textualize/textual @ v8.2.8 — message.py, message_pump.py, widget.py, dom.py,
  screen.py, app.py, widgets/_markdown.py (pin-verification only, not docs)
