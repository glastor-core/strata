//! Reactive adapter abstractions.
//!
//! Provides traits and zero-boilerplate closure adapters to connect headless state machines
//! with modern reactive signal architectures (Leptos signals, Dioxus signals, Yew callbacks,
//! or raw channels) in both `std` and `no_std` (`alloc`) environments.

use alloc::vec::Vec;
use core::fmt::Debug;

#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
#[cfg(not(feature = "std"))]
use core::cell::RefCell;

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

/// Abstraction for a reactive read-write signal or state cell.
pub trait ReactiveState<T>: Clone {
    /// Obtains a clone of the current state value.
    fn get(&self) -> T;

    /// Sets the state to a new value.
    fn set(&self, value: T);

    /// Updates the state in-place using a mutating closure.
    fn update<F: FnOnce(&mut T)>(&self, f: F);
}

/// Closure-backed [`ReactiveState`] adapter that wraps any framework signal (e.g. Leptos `RwSignal` or Dioxus `Signal`)
/// without requiring a custom wrapper struct.
#[derive(Debug, Clone, Copy)]
pub struct FnState<G, S> {
    get_fn: G,
    set_fn: S,
}

impl<G, S> FnState<G, S> {
    /// Creates a reactive state adapter from `get` and `set` closures.
    pub fn new(get_fn: G, set_fn: S) -> Self {
        Self { get_fn, set_fn }
    }
}

impl<T, G, S> ReactiveState<T> for FnState<G, S>
where
    T: Clone,
    G: Fn() -> T + Clone,
    S: Fn(T) + Clone,
{
    fn get(&self) -> T {
        (self.get_fn)()
    }

    fn set(&self, value: T) {
        (self.set_fn)(value);
    }

    fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut current = (self.get_fn)();
        f(&mut current);
        (self.set_fn)(current);
    }
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

/// Closure-backed [`ActionSink`] adapter that executes a closure for each emitted FSM action
/// without requiring a custom struct implementation.
#[derive(Debug, Clone, Copy)]
pub struct FnSink<F> {
    handler: F,
}

impl<F> FnSink<F> {
    /// Wraps an action handler closure into an [`ActionSink`].
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<A, F> ActionSink<A> for FnSink<F>
where
    F: Fn(A) + Clone,
{
    fn emit(&self, action: A) {
        (self.handler)(action);
    }
}

/// Minimal in-memory reactive cell implementation suitable for `no_std` (`alloc`) and unit tests.
#[derive(Debug, Clone)]
pub struct InMemoryCell<T> {
    #[cfg(feature = "std")]
    inner: Arc<Mutex<T>>,
    #[cfg(not(feature = "std"))]
    inner: Rc<RefCell<T>>,
}

impl<T> InMemoryCell<T> {
    /// Creates a new in-memory reactive cell.
    pub fn new(initial: T) -> Self {
        Self {
            #[cfg(feature = "std")]
            inner: Arc::new(Mutex::new(initial)),
            #[cfg(not(feature = "std"))]
            inner: Rc::new(RefCell::new(initial)),
        }
    }
}

impl<T: Clone> ReactiveState<T> for InMemoryCell<T> {
    fn get(&self) -> T {
        #[cfg(feature = "std")]
        {
            self.inner.lock().expect("mutex poisoned").clone()
        }
        #[cfg(not(feature = "std"))]
        {
            self.inner.borrow().clone()
        }
    }

    fn set(&self, value: T) {
        #[cfg(feature = "std")]
        {
            *self.inner.lock().expect("mutex poisoned") = value;
        }
        #[cfg(not(feature = "std"))]
        {
            *self.inner.borrow_mut() = value;
        }
    }

    fn update<F: FnOnce(&mut T)>(&self, f: F) {
        #[cfg(feature = "std")]
        {
            let mut guard = self.inner.lock().expect("mutex poisoned");
            f(&mut guard);
        }
        #[cfg(not(feature = "std"))]
        {
            let mut guard = self.inner.borrow_mut();
            f(&mut guard);
        }
    }
}

/// Action sink collecting emitted actions into a recorded buffer (useful for testing and auditing).
#[derive(Debug, Clone, Default)]
pub struct ActionRecorder<A> {
    #[cfg(feature = "std")]
    recorded: Arc<Mutex<Vec<A>>>,
    #[cfg(not(feature = "std"))]
    recorded: Rc<RefCell<Vec<A>>>,
}

impl<A> ActionRecorder<A> {
    /// Creates a new action recorder.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "std")]
            recorded: Arc::new(Mutex::new(Vec::new())),
            #[cfg(not(feature = "std"))]
            recorded: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Returns all collected actions.
    pub fn actions(&self) -> Vec<A>
    where
        A: Clone,
    {
        #[cfg(feature = "std")]
        {
            self.recorded.lock().expect("mutex poisoned").clone()
        }
        #[cfg(not(feature = "std"))]
        {
            self.recorded.borrow().clone()
        }
    }

    /// Clears the recorded action buffer.
    pub fn clear(&self) {
        #[cfg(feature = "std")]
        {
            self.recorded.lock().expect("mutex poisoned").clear();
        }
        #[cfg(not(feature = "std"))]
        {
            self.recorded.borrow_mut().clear();
        }
    }
}

impl<A: Clone + Debug> ActionSink<A> for ActionRecorder<A> {
    fn emit(&self, action: A) {
        #[cfg(feature = "std")]
        {
            self.recorded.lock().expect("mutex poisoned").push(action);
        }
        #[cfg(not(feature = "std"))]
        {
            self.recorded.borrow_mut().push(action);
        }
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
    #[cfg(feature = "std")]
    machine: Arc<Mutex<M>>,
    #[cfg(not(feature = "std"))]
    machine: Rc<RefCell<M>>,
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
            #[cfg(feature = "std")]
            machine: Arc::new(Mutex::new(machine)),
            #[cfg(not(feature = "std"))]
            machine: Rc::new(RefCell::new(machine)),
            state_signal,
            action_sink,
        }
    }

    /// Dispatches an event into the machine, atomically updating the reactive signal
    /// and routing emitted actions to the sink.
    pub fn send(&self, event: M::Event) -> Vec<M::Action> {
        let (actions, new_state) = {
            #[cfg(feature = "std")]
            {
                let mut guard = self.machine.lock().expect("mutex poisoned");
                let acts = guard.transition(event);
                (acts, guard.state().clone())
            }
            #[cfg(not(feature = "std"))]
            {
                let mut guard = self.machine.borrow_mut();
                let acts = guard.transition(event);
                (acts, guard.state().clone())
            }
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
