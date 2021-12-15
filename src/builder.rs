//! Builder describing multiple interacting state machines
//!
//! You must define a [`StateMachines`] and use [`StateMachines::build`] to get a
//! runnable [`crate::StateMachines`].
//!
//! In order for [`crate::Transition`] impls to be able to return
//! [`crate::Target`], that cannot be constructed due to the arguments of
//! [`crate::Target::Goto`] and [`crate::Target::Enter`] being non-constructible,
//! you need to first define a [`IntoTransition`] impl. [`IntoTransition::into_with`]
//! takes a [`NameMapping`] argument that let you construct [`crate::Target`]
//! that you will be able to use in the impl of [`crate::Transition`].
//!
//! Once your obtain a [`crate::StateMachines`] through
//! the [`StateMachines::build`] method, you can use it with
//! [`label::NestedMachine`](crate::label::NestedMachine) to manage a state
//! machine.
use ahash::AHashMap;
use serde::Deserialize;
use smallvec::SmallVec;

use crate::{SHandle, SHandleInner, SmHandle, SmHandleInner};

/// Convert `Self` into something that implements [`crate::Transition`]
///
/// You will need [`NameMapping`] to be able to instantiate the [`crate::Target`]
/// necessary for transitions to work. The [`NameMapping`] contains the
/// [`SHandle`] and [`SmHandle`] necessary for implementing
/// [`crate::Transition`]. The names correspond to the ones you provided in
/// [`State`] and [`StateMachine`] `name` fields.
pub trait IntoTransition<T> {
    /// Convert `Self` into `T`
    fn into_with(self, mapping: &NameMapping) -> T;
}

/// Obtain a [`Target`](crate::Target) based on serialized state and machine names
///
/// Use the [`NameMapping::target`], [`NameMapping::goto`] and [`NameMapping::enter`]
/// methods to convert a [`Target`] with a [`String`] state/machine name into a
/// [`Target`](crate::Target) with [`SHandle`] and [`SmHandle`] state names,
/// usable with the [`crate::StateMachines`] compact and efficient struct.
///
/// The names correspond to the ones you provided in [`State`] and [`StateMachine`]
/// `name` fields.
pub struct NameMapping {
    state_names: AHashMap<String, SHandleInner>,
    machine_names: AHashMap<String, SmHandleInner>,
}
impl NameMapping {
    fn new() -> Self {
        NameMapping {
            state_names: AHashMap::new(),
            machine_names: AHashMap::new(),
        }
    }
    /// Get [`crate::Target`] corresponding to this [`Target`]
    pub fn target(&self, target: &Target) -> Option<crate::Target> {
        match target {
            Target::Goto(name) => self.goto(name),
            Target::Enter(name) => self.enter(name),
            Target::End => Some(crate::Target::Complete),
        }
    }
    /// Get a [`crate::Target::Goto`] pointing to `State` named `name`
    pub fn goto(&self, name: &str) -> Option<crate::Target> {
        let target = self.state_names.get(name)?;
        Some(crate::Target::Goto(SHandle(*target)))
    }
    /// Get a [`crate::Target::Enter`] pointing to `StateMachine` named `name`
    pub fn enter(&self, name: &str) -> Option<crate::Target> {
        let target = self.machine_names.get(name)?;
        Some(crate::Target::Enter(SmHandle(*target)))
    }
}

/// Convenience enum for serialized state machines
///
/// Pass this enum to the [`NameMapping::target`] method to get the corresponding
/// [`crate::Target`] needed to implement the [`crate::Transition`] trait.
#[derive(Deserialize)]
pub enum Target {
    Goto(String),
    Enter(String),
    End,
}

pub struct State<B, T> {
    pub name: String,
    pub behavior: B,
    pub transitions: Vec<T>,
}

/// A single state machine which states can refer to each other by [`String`] name
pub struct StateMachine<B, T> {
    pub name: String,
    pub states: Vec<State<B, T>>,
}

/// Multiple state machines that may refer each other by [`String`] name
///
/// Use [`StateMachines::build`] to get a [`crate::StateMachines`] usable with
/// [`label::NestedMachine`](crate::label::NestedMachine) for an efficient
/// state machine. `T` must implement [`IntoTransition`].
#[derive(Deserialize)]
#[serde(transparent)]
pub struct StateMachines<B, T>(pub Vec<StateMachine<B, T>>);

impl<B, T> StateMachines<B, T> {
    /// Convert `Self` into a [`crate::StateMachines`]
    ///
    /// See [`NameMapping`] and [`IntoTransition`] for details on why this is
    /// necessary.
    pub fn build<Trs>(self) -> crate::StateMachines<B, Trs>
    where
        T: IntoTransition<Trs>,
    {
        let mut mapping = NameMapping::new();
        let mut ret = crate::StateMachines {
            machines: SmallVec::with_capacity(self.0.len()),
            machine_names: Vec::with_capacity(self.0.len()),
            state_names: Vec::with_capacity(self.0.len()),
        };
        // First: iterate through the builder to collect all state and machine names
        for (mi, StateMachine { name, states }) in self.0.iter().enumerate() {
            ret.machine_names.push(name.clone());
            mapping
                .machine_names
                .insert(name.clone(), mi as SmHandleInner);

            ret.state_names.push(Vec::with_capacity(states.len()));
            let state_names = ret.state_names.last_mut().unwrap();
            for (si, State { name, .. }) in states.iter().enumerate() {
                state_names.push(name.clone());
                mapping.state_names.insert(name.clone(), si as SHandleInner);
            }
        }
        // Then, we can finally build the REAL crate::StateMachines now that we
        // know the String->index mapping
        for StateMachine { states, .. } in self.0.into_iter() {
            let mut machine = Vec::with_capacity(states.len());
            for State {
                transitions,
                behavior,
                ..
            } in states.into_iter()
            {
                machine.push(crate::State {
                    transitions: transitions
                        .into_iter()
                        .map(|t| t.into_with(&mapping))
                        .collect(),
                    behavior,
                });
            }
            ret.machines.push(crate::StateMachine {
                states: machine.into(),
            });
        }
        ret
    }
}
