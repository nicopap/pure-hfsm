//! Deserialize state machines
use serde::{de::Visitor, Deserialize, Deserializer};
use std::marker::PhantomData;

use crate::builder::{State, StateMachine};

struct StateMachineVisitor<B, T>(PhantomData<(B, T)>);
impl<B, T> StateMachineVisitor<B, T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<'de, B: Deserialize<'de>, T: Deserialize<'de>> Visitor<'de> for StateMachineVisitor<B, T> {
    type Value = StateMachine<B, T>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A List describing a StateMachine [ \"Name\" State1 State2 ..]")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let name = seq.next_element()?.ok_or_else(|| todo!())?;
        let mut states = Vec::with_capacity(seq.size_hint().unwrap_or(2));
        while let Some(state) = seq.next_element()? {
            states.push(state);
        }
        Ok(StateMachine { name, states })
    }
}

struct StateVisitor<B, T>(PhantomData<(B, T)>);
impl<B, T> StateVisitor<B, T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<'de, B: Deserialize<'de>, T: Deserialize<'de>> Visitor<'de> for StateVisitor<B, T> {
    type Value = State<B, T>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A List describing a State [ \"Name\" Behavior Trs1 Trs2 .. ]")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let name = seq.next_element()?.ok_or_else(|| todo!())?;
        let behavior = seq.next_element()?.ok_or_else(|| todo!())?;
        let mut transitions = Vec::with_capacity(seq.size_hint().unwrap_or(2));
        while let Some(transition) = seq.next_element()? {
            transitions.push(transition);
        }
        Ok(State {
            name,
            behavior,
            transitions,
        })
    }
}

impl<'de, B: Deserialize<'de>, T: Deserialize<'de>> Deserialize<'de> for StateMachine<B, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StateMachineVisitor::new())
    }
}

impl<'de, B: Deserialize<'de>, T: Deserialize<'de>> Deserialize<'de> for State<B, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StateVisitor::new())
    }
}
