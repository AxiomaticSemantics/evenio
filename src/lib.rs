#![doc = include_str!("../README.md")]

extern crate alloc;

// Lets us use our own derive macros internally.
extern crate self as evenio;

pub mod access;
pub mod archetype;
pub mod bit_set;
mod blob_vec;
pub mod bool_expr;
pub mod component;
mod debug_checked;
pub mod entity;
pub mod event;
#[doc(hidden)]
pub mod exclusive;
pub mod fetch;
mod layout_util;
pub mod query;
mod slot_map;
pub mod sparse;
mod sparse_map;
pub mod system;
pub mod world;

/// For macros only.
#[doc(hidden)]
pub mod __private {
    pub use memoffset::offset_of;
}

pub mod prelude {
    pub use crate::component::{Component, ComponentId};
    pub use crate::entity::EntityId;
    pub use crate::event::{
        AddComponent, AddEvent, AddSystem, Despawn, Event, EventId, EventMut, Insert, Receiver,
        Remove, Sender, Spawn,
    };
    pub use crate::fetch::{FetchError, Fetcher};
    pub use crate::query::{Has, Not, Or, Query, ReadOnlyQuery, With, Xor};
    pub use crate::system::{IntoSystem, SystemId};
    pub use crate::world::World;
}

const _: () = assert!(
    std::mem::size_of::<usize>() >= std::mem::size_of::<u32>(),
    "unsupported target"
);
