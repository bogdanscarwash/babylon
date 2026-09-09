//! Keyboard focus for the existing observer widgets.
//!
//! Integration contract:
//! - Add only `ObserverFocusPlugin`; it installs Bevy's input/tab plugins.
//! - Put `TabGroup::new(order)` on each ordinary panel's ancestor, and
//!   `TabGroup::modal()` on the menu and comparison roots. Do not nest groups.
//!   A group root is not itself a focus target.
//! - Register the actual interactive entity with `ObserverFocusTarget::action`.
//!   This includes picking-only Archive chips: they need no `Button` component.
//!   Register a text reading region with `ObserverFocusTarget::reading`; its
//!   nearest scroll ancestor receives PageUp/PageDown/Home/End.
//! - In `PreUpdate`, `ObserverFocusSystems::Eligibility`, owners update
//!   `available` using their existing availability helpers, and update the
//!   policy's exact context and active modal root. Use `set_if_neq`. A target's
//!   optional context binds dynamic controls; `None` is for fixed shell controls.
//!   Snapshot-dependent targets remain unavailable until their owner has a
//!   validated snapshot. These components are presentation state, not authority.
//! - Handle `On<ObserverKeyboardActivate>` in each widget's own module. Query its
//!   private action component by `event.entity`, check `event.context`, and call
//!   the SAME local action helper as pointer input. That helper must revalidate
//!   current availability and scope. This is an activation attempt, including
//!   for a focused control that became disabled, so its normal denial can show.
//! - Shell world shortcuts must honor `blocks_world_shortcuts()`. Other raw key
//!   handlers (notably browser Enter) must honor `claimed(key)`. Stopping a
//!   `FocusedInput` event does not consume Bevy's raw `ButtonInput` resource.
//! - Owners retain existing Escape/close routes. Do not add a second action enum,
//!   synthesize `Interaction::Pressed`, or use displayed text to restore focus.
//! - After accepting WORLD/WORK navigation, even to the current view, trigger
//!   `ObserverFocusWorld` from the existing command handler. This returns keys
//!   to map/camera navigation on the next frame; Tab re-enters the controls.
//!   The activating frame stays claimed. Emit only after focus preparation
//!   (existing command handlers run in Update), never from eligibility sync.
//!   Rejected commands emit nothing; an active modal refuses focus release.
//! - This module owns `TabIndex` and `Outline` on registered entities. Owner
//!   painters may style hover/pressed states but must leave that outline alone.
//! - Existing app ordering keeps widget action handlers in `ObserverSet::Input`.
//!   All focus preparation and input dispatch finish in `PreUpdate` first:
//!   eligibility -> registration -> Bevy picking -> reconciliation -> Bevy
//!   focused-input dispatch -> focus capture. Deferred commands are flushed at
//!   these boundaries. Capture preserves claims already made in that frame.

use bevy::ecs::system::SystemParam;
use bevy::input::{keyboard::KeyboardInput, ButtonState, InputSystems};
use bevy::input_focus::tab_navigation::{
    NavAction, TabGroup, TabIndex, TabNavigation, TabNavigationPlugin,
};
use bevy::input_focus::{
    FocusedInput, InputDispatchPlugin, InputFocus, InputFocusSystems, InputFocusVisible,
};
use bevy::picking::PickingSystems;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::window::PrimaryWindow;

use crate::observer::ObservationContext;
use crate::observer_theme as theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObserverFocusKind {
    Action,
    Reading,
}

/// Owner-supplied presentation eligibility. Actions always revalidate on use.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObserverFocusTarget {
    pub available: bool,
    pub context: Option<ObservationContext>,
    pub kind: ObserverFocusKind,
}

impl ObserverFocusTarget {
    /// Starts unavailable; its owner admits it in the eligibility set.
    pub(crate) const fn action(context: Option<ObservationContext>) -> Self {
        Self {
            available: false,
            context,
            kind: ObserverFocusKind::Action,
        }
    }

    /// Starts unavailable, like an action. Enter/Space never activates readings.
    pub(crate) const fn reading(context: Option<ObservationContext>) -> Self {
        Self {
            available: false,
            context,
            kind: ObserverFocusKind::Reading,
        }
    }
}

/// A request to the existing local action handler, never an authorization.
#[derive(EntityEvent, Clone, Debug)]
pub(crate) struct ObserverKeyboardActivate {
    #[event_target]
    pub entity: Entity,
    pub context: Option<ObservationContext>,
}

/// Successful navigation hands keyboard ownership back to the primary world.
/// This event changes focus only; the existing action handler admits navigation.
#[derive(Event, Clone, Copy, Debug)]
pub(crate) struct ObserverFocusWorld;

/// The shell supplies exact session context and the single active modal root.
/// Comparison supersedes menu. `None` means no modal, not a hidden modal.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ObserverFocusPolicy {
    pub context: Option<ObservationContext>,
    pub modal: Option<Entity>,
}

/// Per-frame ownership, consulted by existing raw-key shortcut handlers.
#[derive(Resource, Debug, Default)]
pub(crate) struct ObserverKeyboardClaim {
    blocks_world: bool,
    suppress_activation: bool,
    keys: Vec<KeyCode>,
}

impl ObserverKeyboardClaim {
    pub(crate) const fn blocks_world_shortcuts(&self) -> bool {
        self.blocks_world
    }

    pub(crate) fn claimed(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
            || (self.blocks_world && matches!(key, KeyCode::Enter | KeyCode::Space))
    }

    fn claim(&mut self, key: KeyCode) {
        if !self.keys.contains(&key) {
            self.keys.push(key);
        }
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ObserverFocusSystems {
    /// Owners synchronize their policy and canonical presentation availability.
    Eligibility,
    Registration,
    Prepare,
    Capture,
}

#[derive(Debug)]
struct ModalEntry {
    root: Entity,
    return_focus: Option<Entity>,
}

#[derive(Resource, Default)]
struct FocusMemory {
    context: Option<ObservationContext>,
    modals: Vec<ModalEntry>,
    last_registered: Option<Entity>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadingMovement {
    PreviousPage,
    NextPage,
    Start,
    End,
}

impl ReadingMovement {
    const fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::PageUp => Some(Self::PreviousPage),
            KeyCode::PageDown => Some(Self::NextPage),
            KeyCode::Home => Some(Self::Start),
            KeyCode::End => Some(Self::End),
            _ => None,
        }
    }
}

#[derive(Resource, Default)]
struct ReadingRequest(Option<(Entity, ReadingMovement)>);

#[derive(Resource, Default)]
struct RevealFocus {
    target: Option<Entity>,
    last_focus: Option<Entity>,
    viewport: Option<(Vec2, f32, f32)>,
}

pub(crate) struct ObserverFocusPlugin;

impl Plugin for ObserverFocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputDispatchPlugin, TabNavigationPlugin))
            .init_resource::<ObserverFocusPolicy>()
            .init_resource::<ObserverKeyboardClaim>()
            .init_resource::<FocusMemory>()
            .init_resource::<ReadingRequest>()
            .init_resource::<RevealFocus>()
            .init_resource::<UiScale>()
            .configure_sets(
                PreUpdate,
                (
                    ObserverFocusSystems::Eligibility.after(InputSystems),
                    ObserverFocusSystems::Registration
                        .after(ObserverFocusSystems::Eligibility)
                        .before(PickingSystems::ProcessInput),
                    ObserverFocusSystems::Prepare
                        .after(ObserverFocusSystems::Registration)
                        .after(PickingSystems::Last)
                        .before(InputFocusSystems::Dispatch),
                    ObserverFocusSystems::Capture.after(InputFocusSystems::Dispatch),
                ),
            )
            .add_systems(
                PreUpdate,
                (register_tabbables, ApplyDeferred)
                    .chain()
                    .in_set(ObserverFocusSystems::Registration),
            )
            .add_systems(
                PreUpdate,
                (ApplyDeferred, reconcile_focus)
                    .chain()
                    .in_set(ObserverFocusSystems::Prepare),
            )
            .add_systems(
                PreUpdate,
                (ApplyDeferred, capture_focus)
                    .chain()
                    .in_set(ObserverFocusSystems::Capture),
            )
            .add_observer(keyboard_input)
            .add_observer(focus_world)
            .add_systems(
                PostUpdate,
                (paint_outline, queue_reveal, move_reading, reveal_focus)
                    .chain()
                    .after(UiSystems::Layout),
            );
    }
}

fn focus_world(
    _event: On<ObserverFocusWorld>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut state: FocusState,
    mut reading: ResMut<ReadingRequest>,
) {
    if state.policy.modal.is_some() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    state.focus.set_if_neq(InputFocus(Some(window)));
    if state.visible.0 {
        state.visible.0 = false;
    }
    state.memory.last_registered = None;
    reading.0 = None;
    // Never make the successful action's Enter/Space (or a simultaneous arrow)
    // become a world shortcut later in this frame. Prepare resets this claim
    // next frame, after observing the new primary-window focus.
    state.claim.blocks_world = true;
    state.claim.suppress_activation = true;
}

#[derive(SystemParam)]
struct FocusTree<'w, 's> {
    targets: Query<'w, 's, &'static ObserverFocusTarget>,
    nodes: Query<'w, 's, &'static Node>,
    visibility: Query<'w, 's, &'static Visibility>,
    parents: Query<'w, 's, &'static ChildOf>,
    groups: Query<'w, 's, &'static TabGroup>,
}

impl FocusTree<'_, '_> {
    /// Ignores disabled state so a focused, newly disabled control can explain
    /// its refusal. Hidden, stale and inactive-modal controls cannot retain it.
    fn retainable(&self, entity: Entity, policy: &ObserverFocusPolicy) -> bool {
        let Ok(target) = self.targets.get(entity) else {
            return false;
        };
        if !self.nodes.contains(entity)
            || (target.context.is_some() && target.context != policy.context)
        {
            return false;
        }
        let mut cursor = Some(entity);
        let mut group = None;
        let mut inside_modal = false;
        while let Some(current) = cursor {
            if self
                .nodes
                .get(current)
                .is_ok_and(|node| node.display == Display::None)
                || self
                    .visibility
                    .get(current)
                    .is_ok_and(|value| *value == Visibility::Hidden)
            {
                return false;
            }
            if group.is_none() {
                group = self.groups.get(current).ok().map(|value| (current, value));
            }
            inside_modal |= Some(current) == policy.modal;
            cursor = self.parents.get(current).ok().map(ChildOf::parent);
        }
        let Some((group_entity, group)) = group else {
            return false;
        };
        // No nested group may escape the modal trap. A target is a child, never
        // the TabGroup root itself (Bevy excludes roots from its target query).
        group_entity != entity
            && match policy.modal {
                Some(root) => inside_modal && group_entity == root && group.modal,
                None => !group.modal,
            }
    }

    fn tabbable(&self, entity: Entity, policy: &ObserverFocusPolicy) -> bool {
        self.retainable(entity, policy)
            && self
                .targets
                .get(entity)
                .is_ok_and(|target| target.available)
    }
}

type RegistrationChangeFilter = Or<(
    Changed<Node>,
    Changed<Visibility>,
    Changed<ChildOf>,
    Changed<TabGroup>,
    Changed<ObserverFocusTarget>,
)>;

#[derive(SystemParam)]
struct RegistrationChanges<'w, 's> {
    changed: Query<'w, 's, (), RegistrationChangeFilter>,
    targets: RemovedComponents<'w, 's, ObserverFocusTarget>,
    nodes: RemovedComponents<'w, 's, Node>,
    visibility: RemovedComponents<'w, 's, Visibility>,
    parents: RemovedComponents<'w, 's, ChildOf>,
    groups: RemovedComponents<'w, 's, TabGroup>,
}

fn register_tabbables(
    mut commands: Commands,
    tree: FocusTree,
    policy: Res<ObserverFocusPolicy>,
    targets: Query<(Entity, Option<&TabIndex>), With<ObserverFocusTarget>>,
    mut changes: RegistrationChanges,
) {
    let mut removed_target = false;
    for entity in changes.targets.read() {
        removed_target = true;
        if !tree.targets.contains(entity) {
            if let Ok(mut entity) = commands.get_entity(entity) {
                entity.remove::<(TabIndex, Outline)>();
            }
        }
    }
    // Consume all removal streams even if an earlier one contains a change.
    let removed = removed_target
        | (changes.nodes.read().count() > 0)
        | (changes.visibility.read().count() > 0)
        | (changes.parents.read().count() > 0)
        | (changes.groups.read().count() > 0);
    if !policy.is_changed() && !removed && changes.changed.is_empty() {
        return; // No repeated ancestry walk over an idle event catalogue.
    }
    for (entity, index) in &targets {
        match (tree.tabbable(entity, &policy), index) {
            (true, None) => {
                commands.entity(entity).insert(TabIndex(0));
            }
            (false, Some(_)) => {
                commands.entity(entity).remove::<TabIndex>();
            }
            _ => {}
        }
    }
}

#[derive(SystemParam)]
struct FocusState<'w> {
    policy: Res<'w, ObserverFocusPolicy>,
    focus: ResMut<'w, InputFocus>,
    visible: ResMut<'w, InputFocusVisible>,
    claim: ResMut<'w, ObserverKeyboardClaim>,
    memory: ResMut<'w, FocusMemory>,
}

fn reconcile_focus(mut state: FocusState, tree: FocusTree, nav: TabNavigation) {
    let previous = state.focus.get();
    let previous_was_ui = previous.is_some()
        && (previous == state.memory.last_registered
            || previous.is_some_and(|entity| tree.targets.contains(entity)));
    let scope_changed = state.memory.context != state.policy.context;
    let old_modal = state.memory.modals.last().map(|entry| entry.root);
    let modal_changed = old_modal != state.policy.modal;
    state.claim.keys.clear();
    state.claim.suppress_activation = scope_changed || modal_changed;
    state.memory.context.clone_from(&state.policy.context);

    if modal_changed {
        let candidate = modal_return_target(&mut state, previous);
        let restored = candidate.filter(|entity| tree.tabbable(*entity, &state.policy));
        let next = restored.or_else(|| first_target(&nav, state.policy.modal));
        state.focus.set_if_neq(InputFocus(next));
        if next.is_some() && !state.visible.0 {
            state.visible.0 = true;
        }
    } else if (previous_was_ui || state.policy.modal.is_some())
        && !previous.is_some_and(|entity| tree.retainable(entity, &state.policy))
    {
        // Hidden/destroyed targets and pointer focus outside a modal must not
        // put this frame's activation onto a replacement or into the world.
        state.claim.suppress_activation = true;
        let next = first_target(&nav, state.policy.modal);
        state.focus.set_if_neq(InputFocus(next));
    }

    state.memory.last_registered = state
        .focus
        .get()
        .filter(|entity| tree.targets.contains(*entity));
    state.claim.blocks_world = state.policy.modal.is_some()
        || state.memory.last_registered.is_some()
        || state.claim.suppress_activation
        || (previous_was_ui && state.focus.get().is_none());
}

/// Bevy changes focus during Tab traversal and pointer acquisition. Remember
/// the final registered entity before Update can rebuild or despawn its panel.
/// Also capture Tab's world-shortcut ownership in this same frame, even when
/// the other key is a world navigation key rather than Enter/Space.
fn capture_focus(
    focus: Res<InputFocus>,
    policy: Res<ObserverFocusPolicy>,
    tree: FocusTree,
    mut claim: ResMut<ObserverKeyboardClaim>,
    mut memory: ResMut<FocusMemory>,
) {
    let registered = focus.get().filter(|entity| tree.targets.contains(*entity));
    if registered.is_some() || policy.modal.is_some() {
        claim.blocks_world = true;
    }
    memory.last_registered = registered;
}

fn modal_return_target(state: &mut FocusState, previous: Option<Entity>) -> Option<Entity> {
    let Some(root) = state.policy.modal else {
        let original = state
            .memory
            .modals
            .first()
            .and_then(|entry| entry.return_focus);
        state.memory.modals.clear();
        return original;
    };
    if let Some(index) = state
        .memory
        .modals
        .iter()
        .position(|entry| entry.root == root)
    {
        let restoring = state
            .memory
            .modals
            .get(index + 1)
            .and_then(|entry| entry.return_focus);
        state.memory.modals.truncate(index + 1);
        restoring
    } else {
        state.memory.modals.push(ModalEntry {
            root,
            return_focus: previous,
        });
        None
    }
}

fn first_target(nav: &TabNavigation, modal: Option<Entity>) -> Option<Entity> {
    match modal {
        Some(root) => nav.initialize(root, NavAction::First).ok(),
        None => nav.navigate(&InputFocus::default(), NavAction::First).ok(),
    }
}

fn keyboard_input(
    mut event: On<FocusedInput<KeyboardInput>>,
    mut commands: Commands,
    tree: FocusTree,
    policy: Res<ObserverFocusPolicy>,
    focus: Res<InputFocus>,
    mut claim: ResMut<ObserverKeyboardClaim>,
    mut reading: ResMut<ReadingRequest>,
) {
    if event.focused_entity != event.original_event_target() {
        return;
    }
    let key = event.input.key_code;
    if key == KeyCode::Tab {
        claim.claim(key);
        return; // Bevy's primary-window observer performs traversal.
    }
    let target = focus
        .get()
        .and_then(|entity| tree.targets.get(entity).ok().map(|value| (entity, value)));
    let owned = target.is_some() || policy.modal.is_some() || claim.suppress_activation;
    if !owned
        || (!matches!(key, KeyCode::Enter | KeyCode::Space)
            && ReadingMovement::from_key(key).is_none())
    {
        return;
    }
    claim.blocks_world = true;
    claim.claim(key);
    event.propagate(false);
    if event.input.state != ButtonState::Pressed || event.input.repeat || claim.suppress_activation
    {
        return;
    }
    let Some((entity, target)) = target else {
        return;
    };
    // Batched Tab + Enter may have been dispatched at the previous target.
    // Never redirect that activation onto a newly selected control.
    if entity != event.original_event_target() || !tree.retainable(entity, &policy) {
        return;
    }
    match target.kind {
        ObserverFocusKind::Action if matches!(key, KeyCode::Enter | KeyCode::Space) => {
            commands.trigger(ObserverKeyboardActivate {
                entity,
                context: target.context.clone(),
            });
        }
        ObserverFocusKind::Reading if target.available => {
            if let Some(movement) = ReadingMovement::from_key(key) {
                reading.0 = Some((entity, movement));
            }
        }
        _ => {}
    }
}

fn paint_outline(
    mut commands: Commands,
    tree: FocusTree,
    policy: Res<ObserverFocusPolicy>,
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    mut previous: Local<Option<Entity>>,
) {
    let next = focus
        .get()
        .filter(|entity| visible.0 && tree.retainable(*entity, &policy));
    if *previous == next {
        return;
    }
    if let Some(old) = previous.take() {
        if let Ok(mut entity) = commands.get_entity(old) {
            entity.remove::<Outline>();
        }
    }
    if let Some(entity) = next {
        commands.entity(entity).insert(Outline {
            width: px(2),
            offset: px(2),
            color: theme::YELLOW,
        });
    }
    *previous = next;
}

fn queue_reveal(
    focus: Res<InputFocus>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut reveal: ResMut<RevealFocus>,
) {
    let viewport = windows.single().ok().map(|window| {
        (
            Vec2::new(window.width(), window.height()),
            window.resolution.scale_factor(),
            scale.0,
        )
    });
    if focus.get() != reveal.last_focus || viewport != reveal.viewport {
        reveal.target = focus.get();
        reveal.last_focus = focus.get();
        reveal.viewport = viewport;
    }
}

fn move_reading(
    mut request: ResMut<ReadingRequest>,
    tree: FocusTree,
    policy: Res<ObserverFocusPolicy>,
    geometry: Query<(&Node, &ComputedNode)>,
    mut positions: Query<&mut ScrollPosition>,
) {
    let Some((entity, movement)) = request.0.take() else {
        return;
    };
    if !tree.tabbable(entity, &policy) {
        return;
    }
    let mut cursor = Some(entity);
    while let Some(current) = cursor {
        if let (Ok((node, computed)), Ok(mut position)) =
            (geometry.get(current), positions.get_mut(current))
        {
            if let Some(next) = reading_position(node, computed, position.0, movement) {
                if position.0 != next {
                    position.0 = next;
                }
                return;
            }
        }
        cursor = tree.parents.get(current).ok().map(ChildOf::parent);
    }
}

fn reading_position(
    node: &Node,
    computed: &ComputedNode,
    current: Vec2,
    movement: ReadingMovement,
) -> Option<Vec2> {
    let axis = if node.overflow.y == OverflowAxis::Scroll {
        1
    } else if node.overflow.x == OverflowAxis::Scroll {
        0
    } else {
        return None;
    };
    let maximum = scroll_max(computed)?;
    let inset = computed.content_inset();
    let page = ((computed.size - inset.min_inset - inset.max_inset)
        * computed.inverse_scale_factor)
        .max(Vec2::ZERO);
    let mut next = current;
    next[axis] = match movement {
        ReadingMovement::PreviousPage => current[axis] - page[axis],
        ReadingMovement::NextPage => current[axis] + page[axis],
        ReadingMovement::Start => 0.0,
        ReadingMovement::End => maximum[axis],
    }
    .clamp(0.0, maximum[axis]);
    next.is_finite().then_some(next)
}

/// Bevy's own layout limit includes the space reserved for scrollbars.
fn scroll_max(computed: &ComputedNode) -> Option<Vec2> {
    let scale = computed.inverse_scale_factor;
    let maximum =
        (computed.content_size - computed.size + computed.scrollbar_size).max(Vec2::ZERO) * scale;
    (scale.is_finite()
        && scale > 0.0
        && computed.size.is_finite()
        && computed.content_size.is_finite()
        && computed.scrollbar_size.is_finite()
        && maximum.is_finite())
    .then_some(maximum)
}

fn reveal_focus(
    mut reveal: ResMut<RevealFocus>,
    tree: FocusTree,
    policy: Res<ObserverFocusPolicy>,
    geometry: Query<(&Node, &ComputedNode, &UiGlobalTransform)>,
    mut positions: Query<&mut ScrollPosition>,
) {
    let Some(target) = reveal.target else { return };
    if !tree.retainable(target, &policy) {
        reveal.target = None;
        return;
    }
    let Ok((_, target_node, target_transform)) = geometry.get(target) else {
        reveal.target = None;
        return;
    };
    let mut cursor = tree.parents.get(target).ok().map(ChildOf::parent);
    while let Some(entity) = cursor {
        if let (Ok((node, computed, transform)), Ok(mut position)) =
            (geometry.get(entity), positions.get_mut(entity))
        {
            if let Some(next) = reveal_position(
                node,
                computed,
                transform,
                target_node,
                target_transform,
                position.0,
            ) {
                if next != position.0 {
                    position.0 = next;
                    // One ancestor per layout: outer geometry still reflects
                    // the old inner offset. Recheck after the next layout.
                    return;
                }
            }
        }
        cursor = tree.parents.get(entity).ok().map(ChildOf::parent);
    }
    reveal.target = None;
}

fn reveal_position(
    node: &Node,
    computed: &ComputedNode,
    transform: &UiGlobalTransform,
    target: &ComputedNode,
    target_transform: &UiGlobalTransform,
    current: Vec2,
) -> Option<Vec2> {
    let maximum = scroll_max(computed)?;
    if computed.is_empty() || target.is_empty() || !current.is_finite() {
        return None;
    }
    let matrix = transform.affine();
    if !matrix.is_finite() || matrix.matrix2.determinant().abs() <= f32::EPSILON {
        return None;
    }
    let relative = matrix.inverse() * target_transform.affine();
    let half = target.size * 0.5;
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        half,
        Vec2::new(-half.x, half.y),
    ];
    let mut minimum = Vec2::splat(f32::INFINITY);
    let mut end = Vec2::splat(f32::NEG_INFINITY);
    for corner in corners {
        let point = relative.transform_point2(corner);
        if !point.is_finite() {
            return None;
        }
        minimum = minimum.min(point);
        end = end.max(point);
    }
    let inset = computed.content_inset();
    let viewport_min = computed.size * -0.5 + inset.min_inset;
    let viewport_max = computed.size * 0.5 - inset.max_inset;
    let mut next = current;
    for (axis, overflow) in [node.overflow.x, node.overflow.y].into_iter().enumerate() {
        if overflow == OverflowAxis::Scroll && viewport_max[axis] > viewport_min[axis] {
            let delta = reveal_delta(
                minimum[axis],
                end[axis],
                viewport_min[axis],
                viewport_max[axis],
            );
            // Bevy floors physical scroll offsets. Round the correction away
            // from zero so a fractional clipping gap does not cause repeated
            // subpixel writes while waiting to cross a physical pixel.
            let delta = if delta > 0.0 {
                delta.ceil()
            } else {
                delta.floor()
            };
            next[axis] =
                (current[axis] + delta * computed.inverse_scale_factor).clamp(0.0, maximum[axis]);
        }
    }
    next.is_finite().then_some(next)
}

/// An oversized target already covering the viewport must not oscillate.
fn reveal_delta(start: f32, end: f32, viewport_start: f32, viewport_end: f32) -> f32 {
    if start <= viewport_start && end >= viewport_end {
        0.0
    } else if start < viewport_start {
        start - viewport_start
    } else if end > viewport_end {
        end - viewport_end
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::ObserverSession;
    use bevy::camera::RenderTarget;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::{keyboard::Key, InputPlugin};
    use bevy::picking::backend::{HitData, PointerHits};
    use bevy::picking::pointer::{
        Location, PointerAction, PointerButton, PointerId, PointerInput, PointerLocation,
    };
    use bevy::picking::{InteractionPlugin, PickingPlugin};
    use bevy::window::WindowRef;

    #[derive(Resource, Default)]
    struct Actions {
        accepted: Vec<Entity>,
        refused: Vec<Entity>,
        world_steps: usize,
        world_navigation: usize,
    }

    #[derive(Component)]
    struct WorldNavigationAction;

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct FrameChanges {
        tab_indices: usize,
        outlines: usize,
        scroll_positions: usize,
    }

    fn record_changes(
        tabs: Query<(), (With<ObserverFocusTarget>, Changed<TabIndex>)>,
        outlines: Query<(), (With<ObserverFocusTarget>, Changed<Outline>)>,
        scroll: Query<(), Changed<ScrollPosition>>,
        mut changes: ResMut<FrameChanges>,
    ) {
        // Sample inside the schedule: App::update clears change trackers
        // afterward, so EntityRef::is_changed outside it cannot prove idleness.
        *changes = FrameChanges {
            tab_indices: tabs.iter().count(),
            outlines: outlines.iter().count(),
            scroll_positions: scroll.iter().count(),
        };
    }

    // Models the adapter contract: the actual owner, not the focus layer,
    // decides whether an attempted action is currently admissible.
    fn existing_action(
        event: On<ObserverKeyboardActivate>,
        targets: Query<&ObserverFocusTarget>,
        world_navigation: Query<(), With<WorldNavigationAction>>,
        policy: Res<ObserverFocusPolicy>,
        mut actions: ResMut<Actions>,
        mut commands: Commands,
    ) {
        let target = targets.get(event.entity).expect("registered test action");
        if target.available
            && event.context == target.context
            && (target.context.is_none() || target.context == policy.context)
        {
            actions.accepted.push(event.entity);
            if world_navigation.contains(event.entity) {
                commands.trigger(ObserverFocusWorld);
            }
        } else {
            actions.refused.push(event.entity);
        }
    }

    fn world_shortcut(
        keys: Res<ButtonInput<KeyCode>>,
        claim: Res<ObserverKeyboardClaim>,
        mut actions: ResMut<Actions>,
    ) {
        if !claim.blocks_world_shortcuts()
            && !claim.claimed(KeyCode::Enter)
            && keys.just_pressed(KeyCode::Enter)
        {
            actions.world_steps += 1;
        }
        for key in [KeyCode::ArrowRight, KeyCode::BracketRight] {
            if !claim.blocks_world_shortcuts() && !claim.claimed(key) && keys.just_pressed(key) {
                actions.world_navigation += 1;
            }
        }
    }

    fn app() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin, ObserverFocusPlugin))
            .init_resource::<Actions>()
            .add_observer(existing_action)
            .add_systems(Update, world_shortcut);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        (app, window)
    }

    fn group(app: &mut App, modal: bool) -> Entity {
        app.world_mut()
            .spawn((
                Node::default(),
                if modal {
                    TabGroup::modal()
                } else {
                    TabGroup::new(0)
                },
            ))
            .id()
    }

    fn target(app: &mut App, parent: Entity, context: Option<ObservationContext>) -> Entity {
        let mut target = ObserverFocusTarget::action(context);
        target.available = true;
        let entity = app.world_mut().spawn((Node::default(), target)).id();
        app.world_mut().entity_mut(parent).add_child(entity);
        entity
    }

    fn queue_key(
        app: &mut App,
        window: Entity,
        key_code: KeyCode,
        state: ButtonState,
        repeat: bool,
    ) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
            state,
            text: None,
            repeat,
            window,
        });
    }

    fn key(app: &mut App, window: Entity, key_code: KeyCode, state: ButtonState, repeat: bool) {
        queue_key(app, window, key_code, state, repeat);
        app.update();
    }

    fn press(app: &mut App, window: Entity, code: KeyCode) {
        key(app, window, code, ButtonState::Pressed, false);
        key(app, window, code, ButtonState::Released, false);
    }

    #[test]
    fn picking_only_nodes_tab_in_hierarchy_order_and_shift_tab_reverses() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let first = target(&mut app, root, None);
        let second = target(&mut app, root, None);
        app.update();
        assert!(app.world().get::<Button>(first).is_none());
        press(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(first));
        press(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(second));
        key(
            &mut app,
            window,
            KeyCode::ShiftLeft,
            ButtonState::Pressed,
            false,
        );
        press(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(first));
        assert!(app.world().resource::<InputFocusVisible>().0);
    }

    #[test]
    fn activation_uses_existing_handler_once_and_never_advances_world() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Pressed,
            false,
        );
        key(&mut app, window, KeyCode::Enter, ButtonState::Pressed, true);
        key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Released,
            false,
        );
        press(&mut app, window, KeyCode::Space);
        let actions = app.world().resource::<Actions>();
        assert_eq!(actions.accepted, [button, button]);
        assert_eq!(actions.world_steps, 0);
        app.world_mut().resource_mut::<InputFocus>().set(window);
        press(&mut app, window, KeyCode::Enter);
        assert_eq!(app.world().resource::<Actions>().world_steps, 1);
    }

    #[test]
    fn accepted_world_navigation_releases_next_frame_without_shortcut_fallthrough() {
        for activation in [KeyCode::Enter, KeyCode::Space] {
            let (mut app, window) = app();
            let root = group(&mut app, false);
            let button = target(&mut app, root, None);
            app.world_mut()
                .entity_mut(button)
                .insert(WorldNavigationAction);
            app.update();
            press(&mut app, window, KeyCode::Tab);
            queue_key(&mut app, window, activation, ButtonState::Pressed, false);
            queue_key(
                &mut app,
                window,
                KeyCode::ArrowRight,
                ButtonState::Pressed,
                false,
            );
            app.update();

            assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));
            assert!(!app.world().resource::<InputFocusVisible>().0);
            assert!(app.world().get::<Outline>(button).is_none());
            let claim = app.world().resource::<ObserverKeyboardClaim>();
            assert!(claim.blocks_world_shortcuts());
            assert!(claim.claimed(activation));
            assert!(claim.suppress_activation);
            let actions = app.world().resource::<Actions>();
            assert_eq!(actions.accepted, [button]);
            assert_eq!(actions.world_steps, 0);
            assert_eq!(actions.world_navigation, 0);

            queue_key(&mut app, window, activation, ButtonState::Released, false);
            queue_key(
                &mut app,
                window,
                KeyCode::ArrowRight,
                ButtonState::Released,
                false,
            );
            app.update();
            assert!(!app
                .world()
                .resource::<ObserverKeyboardClaim>()
                .blocks_world_shortcuts());
            press(&mut app, window, KeyCode::ArrowRight);
            press(&mut app, window, KeyCode::BracketRight);
            press(&mut app, window, KeyCode::Enter);
            let actions = app.world().resource::<Actions>();
            assert_eq!(actions.world_navigation, 2);
            assert_eq!(actions.world_steps, 1);

            press(&mut app, window, KeyCode::Tab);
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        }
    }

    #[test]
    fn rejected_world_navigation_keeps_the_control_and_its_refusal() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        app.world_mut()
            .entity_mut(button)
            .insert(WorldNavigationAction);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        app.world_mut()
            .get_mut::<ObserverFocusTarget>(button)
            .unwrap()
            .available = false;
        press(&mut app, window, KeyCode::Enter);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        let actions = app.world().resource::<Actions>();
        assert!(actions.accepted.is_empty());
        assert_eq!(actions.refused, [button]);
        assert_eq!(actions.world_steps, 0);
    }

    #[test]
    fn world_focus_release_cannot_escape_a_modal_or_replace_menu_restoration() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        let modal = group(&mut app, true);
        let close = target(&mut app, modal, None);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = Some(modal);
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        app.world_mut().trigger(ObserverFocusWorld);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        assert!(app.world().resource::<InputFocusVisible>().0);

        // Existing Escape/menu handling changes this policy; release does not.
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = None;
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        assert_eq!(app.world().resource::<Actions>().world_steps, 0);
    }

    #[test]
    fn first_tab_owns_world_navigation_keys_received_in_that_same_frame() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        app.update();
        for code in [KeyCode::Tab, KeyCode::ArrowRight, KeyCode::BracketRight] {
            queue_key(&mut app, window, code, ButtonState::Pressed, false);
        }
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        assert!(app
            .world()
            .resource::<ObserverKeyboardClaim>()
            .blocks_world_shortcuts());
        assert_eq!(app.world().resource::<Actions>().world_navigation, 0);
    }

    #[test]
    fn newly_tabbed_target_destroyed_before_next_frame_cannot_turn_enter_into_step() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let removed = target(&mut app, root, None);
        let safe = target(&mut app, root, None);
        app.update();
        // No key-release update may intervene and accidentally let preparation
        // remember this target. The first Tab frame itself must capture it.
        key(&mut app, window, KeyCode::Tab, ButtonState::Pressed, false);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(removed));
        assert_eq!(
            app.world().resource::<FocusMemory>().last_registered,
            Some(removed)
        );
        app.world_mut().despawn(removed);
        key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Pressed,
            false,
        );
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(safe));
        let actions = app.world().resource::<Actions>();
        assert!(actions.accepted.is_empty());
        assert_eq!(actions.world_steps, 0);
    }

    #[test]
    fn native_picking_acquires_focus_before_same_frame_keyboard_dispatch() {
        let (mut app, window) = app();
        // Use Bevy's actual pointer reception, hover and Press event pipeline.
        // A headless backend supplies only the typed hit, not focus or action.
        app.add_plugins((PickingPlugin, InteractionPlugin));
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        let camera = app.world_mut().spawn_empty().id();
        let location = Location {
            target: RenderTarget::Window(WindowRef::Entity(window))
                .normalize(Some(window))
                .unwrap(),
            position: Vec2::new(20.0, 20.0),
        };
        app.world_mut()
            .spawn((PointerId::Mouse, PointerLocation::new(location.clone())));
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));
        app.world_mut().write_message(PointerHits::new(
            PointerId::Mouse,
            vec![(button, HitData::new(camera, 0.0, None, None))],
            1.0,
        ));
        app.world_mut().write_message(PointerInput::new(
            PointerId::Mouse,
            location,
            PointerAction::Press(PointerButton::Primary),
        ));
        queue_key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Pressed,
            false,
        );
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        assert_eq!(
            app.world().resource::<FocusMemory>().last_registered,
            Some(button)
        );
        let actions = app.world().resource::<Actions>();
        assert_eq!(actions.accepted, [button]);
        assert_eq!(actions.world_steps, 0);
    }

    #[test]
    fn modal_open_and_close_refuse_same_frame_enter_on_a_new_target() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let original = target(&mut app, root, None);
        let modal = group(&mut app, true);
        let close = target(&mut app, modal, None);
        app.update();
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = Some(modal);
        key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Pressed,
            false,
        );
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        assert!(app.world().resource::<Actions>().accepted.is_empty());
        key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Released,
            false,
        );
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = None;
        key(
            &mut app,
            window,
            KeyCode::Enter,
            ButtonState::Pressed,
            false,
        );
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(original));
        let actions = app.world().resource::<Actions>();
        assert!(actions.accepted.is_empty());
        assert_eq!(actions.world_steps, 0);
    }

    #[test]
    fn hidden_and_disabled_controls_are_skipped_but_clipped_controls_remain() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let hidden_parent = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        app.world_mut().entity_mut(root).add_child(hidden_parent);
        let hidden = target(&mut app, hidden_parent, None);
        let disabled = target(&mut app, root, None);
        app.world_mut()
            .get_mut::<ObserverFocusTarget>(disabled)
            .unwrap()
            .available = false;
        let clipped = target(&mut app, root, None);
        app.world_mut()
            .entity_mut(clipped)
            .insert(CalculatedClip { clip: Rect::EMPTY });
        app.update();
        assert!(app.world().get::<TabIndex>(hidden).is_none());
        assert!(app.world().get::<TabIndex>(disabled).is_none());
        assert!(app.world().get::<TabIndex>(clipped).is_some());
        press(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(clipped));
    }

    #[test]
    fn disabled_current_control_retains_focus_and_its_owner_explains_refusal() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        app.world_mut()
            .get_mut::<ObserverFocusTarget>(button)
            .unwrap()
            .available = false;
        press(&mut app, window, KeyCode::Enter);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        assert!(app.world().get::<TabIndex>(button).is_none());
        let actions = app.world().resource::<Actions>();
        assert!(actions.accepted.is_empty());
        assert_eq!(actions.refused, [button]);
        assert_eq!(actions.world_steps, 0);
    }

    #[test]
    fn modal_entry_traps_tabs_and_comparison_returns_to_menu_then_world_control() {
        let (mut app, window) = app();
        let main = group(&mut app, false);
        let original = target(&mut app, main, None);
        let menu = group(&mut app, true);
        let menu_first = target(&mut app, menu, None);
        let menu_second = target(&mut app, menu, None);
        let comparison = group(&mut app, true);
        let close = target(&mut app, comparison, None);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = Some(menu);
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(menu_first));
        press(&mut app, window, KeyCode::Tab);
        assert_eq!(
            app.world().resource::<InputFocus>().get(),
            Some(menu_second)
        );
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = Some(comparison);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        assert!(app.world().get::<TabIndex>(original).is_none());
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = Some(menu);
        app.update();
        assert_eq!(
            app.world().resource::<InputFocus>().get(),
            Some(menu_second)
        );
        app.world_mut().resource_mut::<ObserverFocusPolicy>().modal = None;
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(original));
    }

    #[test]
    fn stale_scope_and_destroyed_focus_never_redirect_enter_or_restore_facts() {
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let context =
            ObserverSession::new(babylon_persistence::CampaignId::from_uuid(uuid::Uuid::nil()))
                .context();
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(context.clone());
        let stale = target(&mut app, root, Some(context.clone()));
        let safe = target(&mut app, root, None);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        let mut changed = context;
        changed.perspective = crate::observer::Perspective::PlayerKnowledge;
        changed.generation += 1;
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(changed);
        press(&mut app, window, KeyCode::Enter);
        assert!(app.world().get::<TabIndex>(stale).is_none());
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(safe));
        assert!(app.world().resource::<Actions>().accepted.is_empty());
        app.world_mut().despawn(safe);
        press(&mut app, window, KeyCode::Enter);
        let actions = app.world().resource::<Actions>();
        assert!(actions.accepted.is_empty());
        assert_eq!(actions.world_steps, 0);
    }

    fn computed(size: Vec2, content_size: Vec2, scale: f32) -> ComputedNode {
        ComputedNode {
            size,
            content_size,
            inverse_scale_factor: scale.recip(),
            ..default()
        }
    }

    #[test]
    fn reveal_uses_physical_bounds_and_logical_offsets_at_both_viewports_and_scales() {
        for viewport in [Vec2::new(1366.0, 768.0), Vec2::new(1920.0, 1080.0)] {
            for scale in [1.0, 1.5] {
                let size = viewport * 0.25 * scale;
                let panel = computed(size, size * 3.0, scale);
                let target = computed(Vec2::splat(20.0 * scale), Vec2::splat(20.0 * scale), scale);
                let target_transform =
                    UiGlobalTransform::from_translation(size * 0.5 + Vec2::new(30.0, 40.0) * scale);
                let node = Node {
                    overflow: Overflow::scroll(),
                    ..default()
                };
                let next = reveal_position(
                    &node,
                    &panel,
                    &UiGlobalTransform::default(),
                    &target,
                    &target_transform,
                    Vec2::ZERO,
                )
                .unwrap();
                assert_eq!(next, Vec2::new(40.0, 50.0));
                let moved = UiGlobalTransform::from_translation(
                    target_transform.affine().translation - next * scale,
                );
                assert_eq!(
                    reveal_position(
                        &node,
                        &panel,
                        &UiGlobalTransform::default(),
                        &target,
                        &moved,
                        next
                    ),
                    Some(next)
                );
            }
        }
    }

    #[test]
    fn reading_paging_is_clamped_and_has_no_activation_action() {
        let panel = computed(Vec2::new(100.0, 200.0), Vec2::new(100.0, 800.0), 2.0);
        let node = Node {
            overflow: Overflow::scroll_y(),
            ..default()
        };
        assert_eq!(
            reading_position(&node, &panel, Vec2::ZERO, ReadingMovement::NextPage),
            Some(Vec2::new(0.0, 100.0))
        );
        assert_eq!(
            reading_position(&node, &panel, Vec2::ZERO, ReadingMovement::PreviousPage),
            Some(Vec2::ZERO)
        );
        assert_eq!(
            reading_position(&node, &panel, Vec2::ZERO, ReadingMovement::End),
            Some(Vec2::new(0.0, 300.0))
        );
        assert_eq!(
            reading_position(&node, &panel, Vec2::new(0.0, 300.0), ReadingMovement::Start),
            Some(Vec2::ZERO)
        );
        let (mut app, window) = app();
        let root = group(&mut app, false);
        let mut reading = ObserverFocusTarget::reading(None);
        reading.available = true;
        let entity = app
            .world_mut()
            .spawn((node, panel, reading, ScrollPosition::default()))
            .id();
        app.world_mut().entity_mut(root).add_child(entity);
        app.update();
        press(&mut app, window, KeyCode::Tab);
        press(&mut app, window, KeyCode::PageDown);
        assert_eq!(
            app.world().get::<ScrollPosition>(entity).unwrap().0,
            Vec2::new(0.0, 100.0)
        );
        press(&mut app, window, KeyCode::Enter);
        assert!(app.world().resource::<Actions>().accepted.is_empty());
        assert_eq!(app.world().resource::<Actions>().world_steps, 0);
    }

    #[test]
    fn nested_reveal_moves_one_scroll_ancestor_per_layout_then_stops() {
        let (mut app, _) = app();
        let root = group(&mut app, false);
        app.world_mut().entity_mut(root).insert((
            Node {
                overflow: Overflow::scroll_y(),
                ..default()
            },
            computed(Vec2::splat(100.0), Vec2::new(100.0, 400.0), 1.0),
            ScrollPosition::default(),
            UiGlobalTransform::default(),
        ));
        let inner = app
            .world_mut()
            .spawn((
                Node {
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                computed(Vec2::splat(100.0), Vec2::new(100.0, 300.0), 1.0),
                ScrollPosition::default(),
                UiGlobalTransform::from_xy(0.0, 60.0),
            ))
            .id();
        app.world_mut().entity_mut(root).add_child(inner);
        let button = target(&mut app, inner, None);
        app.world_mut().entity_mut(button).insert((
            computed(Vec2::splat(20.0), Vec2::splat(20.0), 1.0),
            UiGlobalTransform::from_xy(0.0, 140.0),
        ));
        app.world_mut().resource_mut::<RevealFocus>().target = Some(button);
        app.world_mut().run_system_once(reveal_focus).unwrap();
        // Integral pixel offsets are exact; rounding drift must not pass this test.
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(inner)
                .unwrap()
                .y
                .to_bits(),
            40.0_f32.to_bits()
        );
        assert_eq!(
            app.world().get::<ScrollPosition>(root).unwrap().y.to_bits(),
            0.0_f32.to_bits()
        );

        // The next real layout applies the inner offset to its descendant.
        app.world_mut()
            .entity_mut(button)
            .insert(UiGlobalTransform::from_xy(0.0, 100.0));
        app.world_mut().run_system_once(reveal_focus).unwrap();
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(inner)
                .unwrap()
                .y
                .to_bits(),
            40.0_f32.to_bits()
        );
        assert_eq!(
            app.world().get::<ScrollPosition>(root).unwrap().y.to_bits(),
            60.0_f32.to_bits()
        );

        // A subsequent layout applies the outer offset to the whole subtree.
        app.world_mut()
            .entity_mut(inner)
            .insert(UiGlobalTransform::from_xy(0.0, 0.0));
        app.world_mut()
            .entity_mut(button)
            .insert(UiGlobalTransform::from_xy(0.0, 40.0));
        app.world_mut().run_system_once(reveal_focus).unwrap();
        assert!(app.world().resource::<RevealFocus>().target.is_none());
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(inner)
                .unwrap()
                .y
                .to_bits(),
            40.0_f32.to_bits()
        );
        assert_eq!(
            app.world().get::<ScrollPosition>(root).unwrap().y.to_bits(),
            60.0_f32.to_bits()
        );
    }

    #[test]
    fn oversized_or_invalid_geometry_never_oscillates_or_writes_nan() {
        assert_eq!(
            reveal_delta(-200.0, 200.0, -100.0, 100.0).to_bits(),
            0.0_f32.to_bits()
        );
        let bad = computed(Vec2::splat(100.0), Vec2::splat(f32::NAN), 1.0);
        assert!(scroll_max(&bad).is_none());
        let panel = computed(Vec2::splat(100.0), Vec2::splat(300.0), 1.0);
        let target = computed(Vec2::splat(20.0), Vec2::splat(20.0), 1.0);
        assert!(reveal_position(
            &Node {
                overflow: Overflow::scroll(),
                ..default()
            },
            &panel,
            &UiGlobalTransform::from_scale(Vec2::ZERO),
            &target,
            &UiGlobalTransform::default(),
            Vec2::ZERO
        )
        .is_none());
    }

    #[test]
    fn unchanged_frame_does_not_rewrite_tab_index_outline_or_scroll_position() {
        let (mut app, window) = app();
        app.init_resource::<FrameChanges>()
            .add_systems(Last, record_changes);
        let root = group(&mut app, false);
        let button = target(&mut app, root, None);
        app.world_mut()
            .entity_mut(root)
            .insert(ScrollPosition::default());
        app.update();
        press(&mut app, window, KeyCode::Tab);
        app.update();
        app.update();
        assert!(app.world().get::<TabIndex>(button).is_some());
        assert!(app.world().get::<Outline>(button).is_some());
        assert_eq!(
            *app.world().resource::<FrameChanges>(),
            FrameChanges::default()
        );
    }
}
