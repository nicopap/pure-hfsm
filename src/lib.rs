//! # Pure HFSM
//!
//! A finite state machine library with a clear separation between the machine
//! definition and its changing state.
//!
//! I developed this library for my bevy project. This is why I give generic
//! lifetime parameters to `Behavior::World`, `Behavior::Update` and
//! `Transition::World` types. This requires GATs, but I found it was the only way
//! to get it to work with the horror that is `SystemParam` in bevy.
//!
//! The goal of the library is to have the state machine description be completely
//! independent from the state. From this, we get a lot of cool stuff that other
//! state machine libraries do not let you do easily or trivially, such as:
//! * Serialization and deserialization of state machines
//! * Compact representation of both the state and the descriptions
//! * Minimal mutable data, separate from the state machine. In bevy ECS, I can
//!   store it independently from the state machine, as a `Component` (while
//!   the state machines are loaded as `Asset`)
//! * Shared state machines with as many instances as you want
//!
//! There is a few downsides to know before using this library:
//! * There is more boilerplate to write to get things to work
//! * It's not type safe! However, you will get an error _when constructing_ the
//!   state machine if you are referring to non-existent states or machines
//!   in your `builder::StateMachines`! Which is far better than an error when
//!   running transitions, but still not as optimal as a compilation error.
//! * There are _three_ different `StateMachines` type you have to interact with in
//!   order to use this library (1) is the serialized representation `builder` (2)
//!   is the compact immutable description (3) is the mutable state handle or `label`.
//!
//! For the serialized representation of the state machine to still be convenient
//! and maintainable while having a compact internal representation, you will need
//! to use `builder::StateMachines` for serde interface and convert it into a
//! runnable state machine as a second step.
//!
//! ## Features that may or may not be added in the future
//!
//! - [ ] `serde` cargo feature flag to be able to compile the library without serde
//! - [ ] Better documentation
//! - [ ] A version without the `StateData` `Box<dyn Any>`
//! - [ ] Tests
//! - [ ] A visual state machine editor
//!
//! # License
//!
//! Copyright © 2021 Nicola Papale
//!
//! This software is licensed under either MIT or Apache 2.0 at your leisure. See
//! LICENSE file for details.
//!
//! # How to use this library
//!
//! This library is divided in three types of state machines:
//! * [`builder::StateMachines`]: A serializable description of multiple interacting state machines
//! * [`StateMachines`]: is a compact description of multiple interacting state machines
//! * [`label::NestedMachine`]: is the running state of a state machine
//!
//! You will need to first describe the state machine with
//! [`builder::StateMachines`], use it's
//! [`build`](builder::StateMachines::build) method to get a [`StateMachines`].
//!
//! You will be able to control the execution of a Hierarchical Finite State
//! Machine (aka HFSM) with the [`label::NestedMachine`], passing it
//! a [`StateMachines`] when necessary.
#![feature(generic_associated_types)]
pub mod builder;
mod de;
pub mod label;

use smallvec::SmallVec;
use std::any::Any;

type SHandleInner = u8;
type SmHandleInner = u16;

pub type StateData = Box<dyn Any + Sync + Send>;

/// Behavior to adopt when in a state
pub trait Behavior {
    /// The world we live in and influnces our behavior
    type World<'w, 's>;

    /// Things our behavior changes in the world
    type Update<'w, 's>;

    /// The behavior, what to do to `commands` given `world`
    fn update<'w, 's, 'ww, 'ss>(
        &self,
        data: &mut StateData,
        commands: &mut Self::Update<'w, 's>,
        world: &Self::World<'ww, 'ss>,
    );
}

/// Result of a transition
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Target {
    /// Keep the current `State`
    Continue,
    /// Transition into a new `State`
    Goto(SHandle),
    /// Start a nested `StateMachine`, will come back to this `State` once
    /// the nested state machine completes
    Enter(SmHandle),
    /// Terminate the state machine
    Complete,
}

/// Decider for state transition
///
/// Each state has many `Transition`s. `Transition`s are ran every update in
/// order. The first to return a non-[`Target::Continue`] result will dictate
/// the next `State` or nested `StateMachine` the machine enters.
pub trait Transition {
    /// The world to observe to make a transition decision
    type World<'w, 's>;

    /// To what [`Target`] transition given `world`?
    fn decide<'w, 's>(&self, data: &mut StateData, world: &Self::World<'w, 's>) -> Target;
}

/// `State` handle
#[derive(Debug, Clone, PartialEq)]
pub struct SHandle(SHandleInner);
impl SHandle {
    const INITIAL: Self = SHandle(0);
}

/// `StateMachine` handle
#[derive(Debug, Clone, PartialEq)]
pub struct SmHandle(SmHandleInner);

/// A classical state machine, you know the deal `:)`
#[derive(Debug, Clone)]
struct StateMachine<B, Trs> {
    states: SmallVec<[State<B, Trs>; 2]>,
}
impl<B, T> StateMachine<B, T> {
    fn state<'s>(&'s self, state: &SHandle) -> Option<&'s State<B, T>> {
        self.states.get(state.0 as usize)
    }
}

/// State and the transitions in a state machine
#[derive(Debug, Clone)]
struct State<B, Trs> {
    /// Criterias for exiting the current State (See [`Transition`])
    transitions: Vec<Trs>,
    /// What to do when in this state (see [`Behavior`])
    behavior: B,
}

/// Potental errors from running a state machine
#[derive(Debug)]
pub enum Error {
    EmptyStack,
    BadMachineName,
    BadStateName,
}

// TODO: consider adding a version field to this and S[m]Name and check against
// this the version, then use indexing ops rather than slice::get, since we
// know we are running a state machine compatible with the provided S[m]Name
// there is no risk of out-of-bound access
/// A collection of state machines
///
/// Each state machines within this collection can refer to each other. You
/// should be using [`label::NestedMachine`] to manage the state of a state
/// machine.
///
/// State machines in this `struct` are represented in a space-efficient and
/// type-safe format, where it is impossible to refer to non-existing states
/// and machines.
#[derive(Debug)]
pub struct StateMachines<B, T> {
    machines: SmallVec<[StateMachine<B, T>; 8]>,
    machine_names: Vec<String>,
    state_names: Vec<Vec<String>>,
}
impl<B, T> StateMachines<B, T> {
    /// Get all machine names with their handles
    pub fn machines<'s>(&'s self) -> impl Iterator<Item = (SmHandle, &'s str)> {
        let to_name = |(i, n): (_, &'s String)| (SmHandle(i as u16), n.as_ref());
        self.machine_names.iter().enumerate().map(to_name)
    }
    /// Get all state names with their state handles in provided machine
    pub fn states<'s>(
        &'s self,
        machine: &SmHandle,
    ) -> Option<impl Iterator<Item = (SHandle, &'s str)>> {
        let to_name = |(i, n): (_, &'s String)| (SHandle(i as u8), n.as_ref());
        self.state_names
            .get(machine.0 as usize)
            .map(|names| names.iter().enumerate().map(to_name))
    }
    fn machine<'s>(&'s self, machine: &SmHandle) -> Option<&'s StateMachine<B, T>> {
        self.machines.get(machine.0 as usize)
    }
    fn state_name(&self, machine: &SmHandle, state: &SHandle) -> Option<&str> {
        self.state_names
            .get(machine.0 as usize)
            .and_then(|machine| machine.get(state.0 as usize))
            .map(String::as_ref)
    }
    fn machine_name(&self, machine: &SmHandle) -> Option<&str> {
        self.machine_names
            .get(machine.0 as usize)
            .map(String::as_ref)
    }
    /// Get machine handle for provided machine name
    pub fn machine_handle(&self, name: &str) -> Option<SmHandle> {
        self.machines()
            .find(|handle| handle.1 == name)
            .map(|hn| hn.0)
    }
}
