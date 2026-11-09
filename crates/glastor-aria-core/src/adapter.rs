//! Reactive adapter abstractions.
//!
//! Provides traits to connect headless state machines with modern reactive signal
//! architectures (Leptos signals, Dioxus signals, Yew callbacks, or raw channels).

use core::fmt::Debug;

/// Abstraction for a reactive read-write signal or state cell.
pub trait ReactiveState<T>: Clone {
    /// Obtains a clone of the current state value.
    fn get(&self) -> T;

    /// Sets the state to a new value.
    fn set(&self, value: T);

    /// Updates the state in-place using a mutating closure.
    fn update<F: FnOnce(&mut T)>(&self, f: F);
}

/// A sink that consumes actions produced by an FSM.
pub trait ActionSink<A>: Clone {
    /// Dispatches an action to the environment / DOM.
    fn emit(&self, action: A);

    /// Dispatches a batch of actions.
    fn emit_all(&self, actions: Vec<A>) {
        for action in actions {
            self.emit(action);
        }
    }
}

/// Minimal in-memory reactive cell implementation suitable for `no_std` and unit tests.
#[derive(Debug, Clone)]
pub struct InMemoryCell<T> {
    inner: std::sync::Arc<std::sync::Mutex<T>>,
}

impl<T> InMemoryCell<T> {
    /// Creates a new in-memory reactive cell.
    pub fn new(initial: T) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(initial)),
        }
    }
}

impl<T: Clone> ReactiveState<T> for InMemoryCell<T> {
    fn get(&self) -> T {
        self.inner.lock().expect("mutex poisoned").clone()
    }

    fn set(&self, value: T) {
        *self.inner.lock().expect("mutex poisoned") = value;
    }

    fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut guard = self.inner.lock().expect("mutex poisoned");
        f(&mut guard);
    }
}

/// Action sink collecting emitted actions into a recorded buffer (useful for testing and auditing).
#[derive(Debug, Clone, Default)]
pub struct ActionRecorder<A> {
    recorded: std::sync::Arc<std::sync::Mutex<Vec<A>>>,
}

impl<A> ActionRecorder<A> {
    /// Creates a new action recorder.
    pub fn new() -> Self {
        Self {
            recorded: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Returns all collected actions.
    pub fn actions(&self) -> Vec<A>
    where
        A: Clone,
    {
        self.recorded.lock().expect("mutex poisoned").clone()
    }

    /// Clears the recorded action buffer.
    pub fn clear(&self) {
        self.recorded.lock().expect("mutex poisoned").clear();
    }
}

impl<A: Clone + Debug> ActionSink<A> for ActionRecorder<A> {
    fn emit(&self, action: A) {
        self.recorded.lock().expect("mutex poisoned").push(action);
    }
}

/// Universal reactive wrapper connecting any `StateMachine` to a reactive signal and an action sink.
///
/// Designed to be compatible with Leptos `RwSignal`, Dioxus `Signal`, or in-memory testing cells.
#[derive(Debug, Clone)]
pub struct ReactiveFsm<
    M: crate::fsm::StateMachine,
    S: ReactiveState<M::State>,
    K: ActionSink<M::Action>,
> {
    machine: std::sync::Arc<std::sync::Mutex<M>>,
    state_signal: S,
    action_sink: K,
}

impl<M: crate::fsm::StateMachine, S: ReactiveState<M::State>, K: ActionSink<M::Action>>
    ReactiveFsm<M, S, K>
{
    /// Connects an existing FSM instance to a reactive signal and an action sink.
    pub fn new(machine: M, state_signal: S, action_sink: K) -> Self {
        state_signal.set(machine.state().clone());
        Self {
            machine: std::sync::Arc::new(std::sync::Mutex::new(machine)),
            state_signal,
            action_sink,
        }
    }

    /// Dispatches an event into the machine, atomically updating the reactive signal
    /// and routing emitted actions to the sink.
    pub fn send(&self, event: M::Event) -> Vec<M::Action> {
        let (actions, new_state) = {
            let mut guard = self.machine.lock().expect("mutex poisoned");
            let acts = guard.transition(event);
            (acts, guard.state().clone())
        };
        self.state_signal.set(new_state);
        self.action_sink.emit_all(actions.clone());
        actions
    }

    /// Returns a clone of the current reactive state.
    pub fn state(&self) -> M::State {
        self.state_signal.get()
    }
}
