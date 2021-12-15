# Pure HFSM

[![Latest version](https://img.shields.io/crates/v/pure-hfsm.svg)](https://crates.io/crates/pure-hfsm)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/pure-hfsm/badge.svg)](https://docs.rs/pure-hfsm)

A finite state machine library with a clear separation between the machine
definition and its changing state.

I developed this library for my bevy project. This is why I give generic
lifetime parameters to `Behavior::World`, `Behavior::Update` and
`Transition::World` types. This requires GATs, but I found it was the only way
to get it to work with the horror that is `SystemParam` in bevy.

The goal of the library is to have the state machine description be completely
independent from the state. From this, we get a lot of cool stuff that other
state machine libraries do not let you do easily or trivially, such as:
* Serialization and deserialization of state machines
* Compact representation of both the state and the descriptions
* Minimal mutable data, separate from the state machine. In bevy ECS, I can
  store it independently from the state machine, as a `Component` (while the
  state machines are loaded as `Asset`)
* Shared state machines with as many instances as you want

There is a few downsides to know before using this library:
* There is more boilerplate to write to get things to work
* It's not type safe! However, you will get an error _when constructing_ the
  state machine if you are referring to non-existent states or machines in your
  `builder::StateMachines`! Which is far better than an error when running
  transitions, but still not as optimal as a compilation error.
* There are _three_ different `StateMachines` type you have to interact with in
  order to use this library (1) is the serialized representation `builder` (2)
  is the compact immutable description (3) is the mutable state handle or
  `label`.

For the serialized representation of the state machine to still be convenient
and maintainable while having a compact internal representation, you will need
to use `builder::StateMachines` for serde interface and convert it into a
runnable state machine as a second step.

## Features that may or may not be added in the future

- [ ] `serde` cargo feature flag to be able to compile the library without serde
- [ ] Better documentation
- [ ] A version without the `StateData` `Box<dyn Any>`
- [ ] Tests
- [ ] A visual state machine editor

# License

Copyright © 2021 Nicola Papale

This software is licensed under either MIT or Apache 2.0 at your leisure. See
LICENSE file for details.

## Additional non binding conditions

If you use this library in a commercial product that earns more than 1 million
of revenue in equivalent January 2021 US dollar, you will do one of the following:
* Send me (Nicola Papale) a cake
* Send me a post card thanking me, with the signature of your dev team or at
  least the lead developers
* Have contributed significantly to this project ("significantly" being defined
  at your leisure, as long as you are honest about it)
* Feel shame and assume full moral responsibility of being a twat, boohoo

Not doing one of the first three items, it will always be assumed that the terms
are complied-with according to the last item. Moral responsibility does not
imply any other obligation than feeling shame. Most notably, no financial or
legal obligations are implied by this.
