// SPDX-License-Identifier: GPL-3.0-or-later
// Based on LiquidBounce event/Event.kt, EventManager.kt and EventHookRegistry.kt.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// A typed, mutable event. Cancellation is sticky across repeated dispatches.
#[derive(Debug)]
pub struct Event<T> {
    pub payload: T,
    completed: bool,
    cancelled: Option<bool>,
}

impl<T> Event<T> {
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            completed: false,
            cancelled: None,
        }
    }

    pub fn cancellable(payload: T) -> Self {
        Self {
            payload,
            completed: false,
            cancelled: Some(false),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled == Some(true)
    }

    /// Like CancellableEvent.cancelEvent, this cannot undo cancellation or run
    /// after dispatch completes. Cancellation does not stop later listeners.
    pub fn cancel(&mut self) -> Result<(), CancelError> {
        if self.completed {
            return Err(CancelError::Completed);
        }
        match self.cancelled.as_mut() {
            Some(cancelled) => {
                *cancelled = true;
                Ok(())
            }
            None => Err(CancelError::NotCancellable),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelError {
    Completed,
    NotCancellable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DispatchReport {
    pub suppressed: bool,
    pub known_event: bool,
    pub invoked: usize,
    pub skipped: usize,
    pub panicked: usize,
}

type Callback = dyn Fn(&mut dyn Any) + Send + Sync;

struct Hook {
    id: HandlerId,
    listener: ListenerId,
    priority: i16,
    running: Arc<dyn Fn() -> bool + Send + Sync>,
    callback: Arc<Callback>,
}

/// Registrations use copy-on-dispatch snapshots. Registering or unregistering
/// inside a callback affects the next call, as in the upstream atomic registry.
/// Gates are evaluated immediately before each handler, including after cancel.
#[derive(Default)]
pub struct EventDispatcher {
    hooks: RwLock<HashMap<TypeId, Vec<Arc<Hook>>>>,
    next_id: AtomicU64,
    suppressed: AtomicBool,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn listener(&self) -> ListenerId {
        ListenerId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// The bridge sets this for the upstream self-destruct/error-screen gate.
    pub fn set_suppressed(&self, suppressed: bool) {
        self.suppressed.store(suppressed, Ordering::Release);
    }

    /// Register event types even before they have listeners. An unknown type is
    /// returned untouched; a known type with no hooks is still completed.
    pub fn register_event<T: 'static>(&self) {
        self.hooks
            .write()
            .expect("event registry poisoned")
            .entry(TypeId::of::<T>())
            .or_default();
    }

    pub fn on<T: 'static>(
        &self,
        listener: ListenerId,
        priority: i16,
        running: impl Fn() -> bool + Send + Sync + 'static,
        callback: impl Fn(&mut Event<T>) + Send + Sync + 'static,
    ) -> HandlerId {
        let id = HandlerId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let hook = Arc::new(Hook {
            id,
            listener,
            priority,
            running: Arc::new(running),
            callback: Arc::new(move |event| {
                callback(
                    event
                        .downcast_mut::<Event<T>>()
                        .expect("typed event mismatch"),
                );
            }),
        });
        let mut registry = self.hooks.write().expect("event registry poisoned");
        let hooks = registry.entry(TypeId::of::<T>()).or_default();
        // Insert after equal priorities: descending priority, stable registration order.
        let index = hooks.partition_point(|existing| existing.priority >= priority);
        hooks.insert(index, hook);
        id
    }

    pub fn unregister(&self, id: HandlerId) -> bool {
        let mut removed = false;
        for hooks in self
            .hooks
            .write()
            .expect("event registry poisoned")
            .values_mut()
        {
            hooks.retain(|hook| {
                if hook.id == id {
                    removed = true;
                    false
                } else {
                    true
                }
            });
        }
        removed
    }

    pub fn unregister_listener(&self, listener: ListenerId) {
        for hooks in self
            .hooks
            .write()
            .expect("event registry poisoned")
            .values_mut()
        {
            hooks.retain(|hook| hook.listener != listener);
        }
    }

    pub fn dispatch<T: 'static>(&self, event: &mut Event<T>) -> DispatchReport {
        if self.suppressed.load(Ordering::Acquire) {
            return DispatchReport {
                suppressed: true,
                ..DispatchReport::default()
            };
        }
        let hooks = self
            .hooks
            .read()
            .expect("event registry poisoned")
            .get(&TypeId::of::<T>())
            .cloned();
        let Some(hooks) = hooks else {
            return DispatchReport::default();
        };
        let mut report = DispatchReport {
            known_event: true,
            ..DispatchReport::default()
        };
        event.completed = false;
        for hook in hooks {
            if !(hook.running)() {
                report.skipped += 1;
                continue;
            }
            report.invoked += 1;
            // Ordinary upstream handler failures are logged and do not skip later
            // hooks. The bridge receives the count; Rust aborting panics cannot be caught.
            if catch_unwind(AssertUnwindSafe(|| (hook.callback)(event))).is_err() {
                report.panicked += 1;
            }
        }
        event.completed = true;
        report
    }
}
