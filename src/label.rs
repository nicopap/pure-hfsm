//! State of a state machine and Hierarchical state machine.
//!
//! You want to use a [`NestedMachine`] to manage an individual
//! instance of a hierarchical finite state machine (HFSM), the
//! [`NestedMachine::update`] method does all the magic of managing the
//! state machine

use crate::{Behavior, Error, SHandle, SmHandle, StateData, Target, Transition};

#[derive(Debug)]
pub enum Complete {
    Done,
    Running,
}

/// Data for individual state
struct State {
    handle: SHandle,
    behavior: StateData,
    transitions: Option<Vec<StateData>>,
}
impl State {
    fn new(handle: SHandle) -> Self {
        State {
            handle,
            behavior: Box::new(()),
            transitions: None,
        }
    }
    fn update<'w, 's, 'ww, 'ss, B, Trs, Wrd, Updt>(
        &mut self,
        state: &crate::State<B, Trs>,
        commands: &mut Updt,
        world: &Wrd,
    ) -> Target
    where
        B: Behavior<Update<'w, 's> = Updt, World<'ww, 'ss> = Wrd>,
        Trs: Transition<World<'ww, 'ss> = Wrd>,
    {
        state.behavior.update(&mut self.behavior, commands, world);
        if self.transitions.is_none() {
            let mut transitions: Vec<StateData> = Vec::with_capacity(state.transitions.len());
            for _ in state.transitions.iter() {
                transitions.push(Box::new(()));
            }
            self.transitions = Some(transitions);
        }
        let trans_data = &mut self.transitions.iter_mut().flatten();
        for (transition, data) in state.transitions.iter().zip(trans_data) {
            let target = transition.decide(data, world);
            if !matches!(target, Target::Continue) {
                return target;
            }
        }
        Target::Continue
    }
}

/// Data for individual machines
struct Machine {
    handle: SmHandle,
    state: State,
}
impl Machine {
    fn new(handle: SmHandle) -> Self {
        Machine {
            handle,
            state: State::new(SHandle::INITIAL),
        }
    }

    fn update<'w, 's, 'ww, 'ss, B, Trs, Wrd, Updt>(
        &mut self,
        machine: &crate::StateMachine<B, Trs>,
        commands: &mut Updt,
        world: &Wrd,
    ) -> Result<Target, Error>
    where
        B: Behavior<Update<'w, 's> = Updt, World<'ww, 'ss> = Wrd> + 'static,
        Trs: Transition<World<'ww, 'ss> = Wrd> + 'static,
    {
        let state = machine
            .state(&self.state.handle)
            .ok_or(Error::BadStateName)?;
        let target = self.state.update(state, commands, world);
        if let Target::Goto(ref new_state_handle) = target {
            self.state = State::new(new_state_handle.clone());
        };
        Ok(target)
    }
}

/// The managed state of a Hierarchical Finite State Machine (HFSM)
///
/// This contains the state pointers of innactive state machines that entered a
/// nested machine, and the state `Data` of those machines.
pub struct NestedMachine {
    stack: Vec<Machine>,
}
impl Default for NestedMachine {
    fn default() -> Self {
        Self::new()
    }
}
impl NestedMachine {
    /// Initialize a `NestedMachine` without any active state
    pub fn new() -> Self {
        NestedMachine {
            stack: Vec::with_capacity(1),
        }
    }
    /// Initialize a `NestedMachine` with the first `State` of the first
    /// `Machine` activated.
    pub fn new_active() -> Self {
        let stack = vec![Machine::new(SmHandle(0))];
        NestedMachine { stack }
    }
    /// Enter the nested state described by [`SmHandle`]
    pub fn enter(&mut self, machine: &SmHandle) {
        self.stack.push(Machine::new(machine.clone()));
    }
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }
    pub fn current_state_name<'a, B, T>(
        &self,
        machines: &'a crate::StateMachines<B, T>,
    ) -> Option<&'a str> {
        let machine = self.stack.last()?;
        machines.state_name(&machine.handle, &machine.state.handle)
    }

    pub fn current_machine_name<'a, B, T>(
        &self,
        machines: &'a crate::StateMachines<B, T>,
    ) -> Option<&'a str> {
        let machine = self.stack.last()?;
        machines.machine_name(&machine.handle)
    }

    pub fn update<'w, 's, 'ww, 'ss, B, Trs, Wrd, Updt>(
        &mut self,
        machines: &crate::StateMachines<B, Trs>,
        commands: &mut Updt,
        world: &Wrd,
    ) -> Result<Complete, Error>
    where
        B: Behavior<Update<'w, 's> = Updt, World<'ww, 'ss> = Wrd> + 'static,
        Trs: Transition<World<'ww, 'ss> = Wrd> + 'static,
    {
        use Complete::{Done, Running};

        let current = self.stack.last_mut().ok_or(Error::EmptyStack)?;
        let machine = machines
            .machine(&current.handle)
            .ok_or(Error::BadMachineName)?;
        let target = current.update(machine, commands, world)?;
        match target {
            Target::Enter(nested_machine) => {
                self.enter(&nested_machine);
                Ok(Running)
            }
            Target::Complete => {
                self.stack.pop();
                let empty = self.stack.is_empty();
                Ok(if empty { Done } else { Running })
            }
            Target::Continue | Target::Goto(_) => Ok(Running),
        }
    }
}
