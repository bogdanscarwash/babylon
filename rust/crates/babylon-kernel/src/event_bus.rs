//! Deterministic event bus: ports `kernel/event_bus.py`'s ordering
//! guarantees (spec §6.5), generic over the topic type — the 100-value
//! `EventType` domain enum lands with `babylon-domain`/`babylon-engine` in
//! Phase 2/3, not here.
//!
//! The four guarantees extracted from the Python source read end-to-end
//! (F1 discipline, 2026-07-30 — the fourth is one the Phase-1 plan's own
//! summary missed):
//!
//! 1. **Registration-order dispatch**: handlers for one topic fire in
//!    subscription order.
//! 2. **Append-before-emit**: the event enters the bus's history before
//!    any handler runs — so the event is never lost even when the fan-out
//!    fails.
//! 3. **Interceptor chain**: stable sort by priority DESCENDING (higher
//!    first), registration order as the tiebreak, sorted per publish.
//!    An interceptor may allow, BLOCK (the ORIGINAL event is recorded to a
//!    blocked-events audit channel with the interceptor's name and reason;
//!    nothing enters history, nothing dispatches), or MODIFY (the modified
//!    event continues down the chain, and it is the MODIFIED event that
//!    history records and handlers see).
//! 4. **Handler isolation** (`_emit_to_handlers`'s III.7 clause): every
//!    subscribed handler runs even when an earlier one fails; failures are
//!    collected and returned together AFTER the fan-out — isolation
//!    without silent swallowing. Rust surfaces Python's `ExceptionGroup`
//!    as an error list in `publish`'s `Result`.

/// One simulation event: a topic plus the tick it occurred on. The Python
/// `Event.payload` dict and its sim-time timestamp are domain-layer
/// concerns (Phase 2/3); the kernel bus pins the ordering machinery only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<T> {
    /// The event's topic (the Python `Event.type` string).
    pub topic: T,
    /// The tick the event occurred on.
    pub tick: u64,
}

/// One handler failure, reported after the full fan-out (guarantee 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerFailure {
    /// Index of the failing handler in registration order.
    pub handler_index: usize,
    /// The handler's own description of the failure.
    pub message: String,
}

/// What an interceptor decides for one event (the Python
/// `InterceptResult` surface).
#[derive(Debug, Clone)]
pub enum Intercept<T> {
    /// Pass the event through unchanged.
    Allow,
    /// Stop the event: it is recorded to the blocked-events audit channel
    /// and never reaches history or handlers.
    Block {
        /// Why the interceptor blocked it (audit-channel record).
        reason: String,
    },
    /// Replace the event; the replacement continues down the chain.
    Modify(Event<T>),
}

/// An interceptor: adversarial-mechanics middleware ahead of emission.
pub trait Interceptor<T> {
    /// Higher priority runs first; ties run in registration order.
    fn priority(&self) -> i32;
    /// The interceptor's audit-channel name.
    fn name(&self) -> &str;
    /// Decide this event's fate.
    fn intercept(&mut self, event: &Event<T>) -> Intercept<T>;
}

/// One blocked-event audit record: the ORIGINAL event (pre-modification —
/// the Python bus logs `event`, not `current_event`, for auditability),
/// plus who blocked it and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedEvent<T> {
    /// The original event as published.
    pub event: Event<T>,
    /// The blocking interceptor's name.
    pub interceptor_name: String,
    /// The blocking reason.
    pub reason: String,
}

/// A subscribed handler: returns `Err(message)` to report failure without
/// stopping the fan-out (guarantee 4).
pub type Handler<T> = Box<dyn FnMut(&Event<T>) -> Result<(), String>>;

/// The deterministic publish/subscribe bus.
pub struct EventBus<T> {
    subscribers: Vec<(T, Handler<T>)>,
    history: Vec<Event<T>>,
    interceptors: Vec<Box<dyn Interceptor<T>>>,
    blocked: Vec<BlockedEvent<T>>,
}

impl<T: Clone + PartialEq> EventBus<T> {
    /// An empty bus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            history: Vec::new(),
            interceptors: Vec::new(),
            blocked: Vec::new(),
        }
    }

    /// Subscribe a handler to a topic. Registration order is dispatch
    /// order (guarantee 1).
    pub fn subscribe(&mut self, topic: T, handler: Handler<T>) {
        self.subscribers.push((topic, handler));
    }

    /// Register an interceptor. Sorted per publish — priority descending,
    /// registration order as the stable tiebreak (guarantee 3).
    pub fn register_interceptor(&mut self, interceptor: Box<dyn Interceptor<T>>) {
        self.interceptors.push(interceptor);
    }

    /// Number of registered interceptors.
    #[must_use]
    pub fn interceptor_count(&self) -> usize {
        self.interceptors.len()
    }

    /// Publish one event.
    ///
    /// Fast path with no interceptors (the Python zero-overhead clause);
    /// otherwise the chain runs first and a blocked event goes to the
    /// audit channel instead of history.
    ///
    /// # Errors
    /// Returns every [`HandlerFailure`] the fan-out produced, AFTER every
    /// handler has run (guarantee 4) — the event is already in history
    /// regardless (guarantee 2).
    pub fn publish(&mut self, event: Event<T>) -> Result<(), Vec<HandlerFailure>> {
        let event = if self.interceptors.is_empty() {
            event
        } else {
            match self.run_interceptors(event) {
                Some(processed) => processed,
                None => return Ok(()), // blocked: audited, nothing dispatched
            }
        };
        self.history.push(event.clone());
        self.dispatch(&event)
    }

    /// The interceptor chain: returns the surviving (possibly modified)
    /// event, or `None` if blocked (audit record written).
    fn run_interceptors(&mut self, event: Event<T>) -> Option<Event<T>> {
        // Stable sort of INDICES by descending priority: equal priorities
        // keep registration order, and sorting per publish mirrors the
        // Python bus exactly.
        let mut order: Vec<usize> = (0..self.interceptors.len()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(self.interceptors[i].priority()));

        let original = event.clone();
        let mut current = event;
        for i in order {
            let interceptor = &mut self.interceptors[i];
            match interceptor.intercept(&current) {
                Intercept::Allow => {}
                Intercept::Block { reason } => {
                    let name = interceptor.name().to_owned();
                    self.blocked.push(BlockedEvent {
                        event: original,
                        interceptor_name: name,
                        reason,
                    });
                    return None;
                }
                Intercept::Modify(replacement) => current = replacement,
            }
        }
        Some(current)
    }

    /// The isolated fan-out (guarantee 4): every matching handler runs;
    /// failures collect and return together.
    fn dispatch(&mut self, event: &Event<T>) -> Result<(), Vec<HandlerFailure>> {
        let mut failures = Vec::new();
        for (index, (topic, handler)) in self.subscribers.iter_mut().enumerate() {
            if *topic == event.topic {
                if let Err(message) = handler(event) {
                    failures.push(HandlerFailure {
                        handler_index: index,
                        message,
                    });
                }
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }

    /// The published-event history, oldest first.
    #[must_use]
    pub fn history(&self) -> &[Event<T>] {
        &self.history
    }

    /// The blocked-events audit channel, oldest first.
    #[must_use]
    pub fn blocked_events(&self) -> &[BlockedEvent<T>] {
        &self.blocked
    }

    /// Clear the published-event history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl<T: Clone + PartialEq> Default for EventBus<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, EventBus, Intercept, Interceptor};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn event(topic: &'static str, tick: u64) -> Event<&'static str> {
        Event { topic, tick }
    }

    #[test]
    fn handlers_fire_in_registration_order() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        let order = Rc::new(RefCell::new(Vec::<u32>::new()));
        let o1 = Rc::clone(&order);
        let o2 = Rc::clone(&order);
        bus.subscribe(
            "t",
            Box::new(move |_| {
                o1.borrow_mut().push(1);
                Ok(())
            }),
        );
        bus.subscribe(
            "t",
            Box::new(move |_| {
                o2.borrow_mut().push(2);
                Ok(())
            }),
        );
        bus.publish(event("t", 0)).unwrap();
        assert_eq!(*order.borrow(), vec![1, 2]);
    }

    #[test]
    fn a_failing_handler_does_not_stop_later_handlers_and_all_failures_report() {
        // Guarantee 4 — the one the plan's summary missed. Python raises an
        // ExceptionGroup AFTER the fan-out; Rust returns the collected list.
        let mut bus: EventBus<&'static str> = EventBus::new();
        let ran = Rc::new(RefCell::new(Vec::<u32>::new()));
        let r1 = Rc::clone(&ran);
        let r3 = Rc::clone(&ran);
        bus.subscribe(
            "t",
            Box::new(move |_| {
                r1.borrow_mut().push(1);
                Ok(())
            }),
        );
        bus.subscribe("t", Box::new(|_| Err("second handler failed".into())));
        bus.subscribe(
            "t",
            Box::new(move |_| {
                r3.borrow_mut().push(3);
                Ok(())
            }),
        );
        let failures = bus.publish(event("t", 5)).unwrap_err();
        assert_eq!(*ran.borrow(), vec![1, 3], "later handlers must still run");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].handler_index, 1);
    }

    #[test]
    fn event_survives_in_history_even_when_every_handler_fails() {
        // Guarantee 2, in the form the Python docstring gives as its
        // REASON: append-before-emit means the event is never lost.
        let mut bus: EventBus<&'static str> = EventBus::new();
        bus.subscribe("t", Box::new(|_| Err("boom".into())));
        let _ = bus.publish(event("t", 9));
        assert_eq!(bus.history(), &[event("t", 9)]);
    }

    #[test]
    fn untargeted_topics_do_not_dispatch() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        let ran = Rc::new(RefCell::new(0u32));
        let r = Rc::clone(&ran);
        bus.subscribe(
            "other",
            Box::new(move |_| {
                *r.borrow_mut() += 1;
                Ok(())
            }),
        );
        bus.publish(event("t", 0)).unwrap();
        assert_eq!(*ran.borrow(), 0);
        assert_eq!(
            bus.history().len(),
            1,
            "history records regardless of subscribers"
        );
    }

    // ---- interceptor chain ----

    struct Recorder {
        label: &'static str,
        priority: i32,
        seen: Rc<RefCell<Vec<&'static str>>>,
        verdict: fn(&Event<&'static str>) -> Intercept<&'static str>,
    }

    impl Interceptor<&'static str> for Recorder {
        fn priority(&self) -> i32 {
            self.priority
        }
        fn name(&self) -> &str {
            self.label
        }
        fn intercept(&mut self, event: &Event<&'static str>) -> Intercept<&'static str> {
            self.seen.borrow_mut().push(self.label);
            (self.verdict)(event)
        }
    }

    fn allow(_: &Event<&'static str>) -> Intercept<&'static str> {
        Intercept::Allow
    }

    #[test]
    fn interceptors_run_priority_desc_with_registration_tiebreak() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        for (label, priority) in [("low", 1), ("high", 9), ("mid_a", 5), ("mid_b", 5)] {
            bus.register_interceptor(Box::new(Recorder {
                label,
                priority,
                seen: Rc::clone(&seen),
                verdict: allow,
            }));
        }
        bus.publish(event("t", 0)).unwrap();
        // Descending priority; mid_a before mid_b because registered first.
        assert_eq!(*seen.borrow(), vec!["high", "mid_a", "mid_b", "low"]);
    }

    #[test]
    fn a_blocked_event_is_audited_and_never_reaches_history_or_handlers() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        let ran = Rc::new(RefCell::new(0u32));
        let r = Rc::clone(&ran);
        bus.subscribe(
            "t",
            Box::new(move |_| {
                *r.borrow_mut() += 1;
                Ok(())
            }),
        );
        bus.register_interceptor(Box::new(Recorder {
            label: "censor",
            priority: 0,
            seen: Rc::new(RefCell::new(Vec::new())),
            verdict: |_| Intercept::Block {
                reason: "state repression".into(),
            },
        }));
        bus.publish(event("t", 3)).unwrap();
        assert_eq!(*ran.borrow(), 0);
        assert!(bus.history().is_empty());
        let blocked = bus.blocked_events();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].interceptor_name, "censor");
        assert_eq!(blocked[0].event, event("t", 3));
    }

    #[test]
    fn a_modified_event_is_what_history_and_handlers_see_but_audit_keeps_the_original() {
        let mut bus: EventBus<&'static str> = EventBus::new();
        // High-priority modifier retargets tick 1 -> tick 100...
        bus.register_interceptor(Box::new(Recorder {
            label: "modifier",
            priority: 9,
            seen: Rc::new(RefCell::new(Vec::new())),
            verdict: |e| {
                Intercept::Modify(Event {
                    topic: e.topic,
                    tick: 100,
                })
            },
        }));
        bus.publish(event("t", 1)).unwrap();
        assert_eq!(bus.history(), &[event("t", 100)]);

        // ...and when a LOWER-priority blocker fires after a modifier, the
        // audit channel records the ORIGINAL event, per the Python bus.
        bus.register_interceptor(Box::new(Recorder {
            label: "blocker",
            priority: 0,
            seen: Rc::new(RefCell::new(Vec::new())),
            verdict: |_| Intercept::Block {
                reason: "after modify".into(),
            },
        }));
        bus.publish(event("t", 2)).unwrap();
        assert_eq!(
            bus.blocked_events()[0].event,
            event("t", 2),
            "original, not modified"
        );
    }
}
