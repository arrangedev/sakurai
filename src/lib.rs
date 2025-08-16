#![no_std]
#![recursion_limit = "2048"]
#![allow(internal_features)]
#![allow(incomplete_features)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(core_intrinsics)]
#![feature(generic_const_exprs)]
#![doc = include_str!("../README.md")]

//! =====================================================
//!
//! ███████╗ █████╗ ██╗  ██╗██╗   ██╗██████╗  █████╗ ██╗
//! ██╔════╝██╔══██╗██║ ██╔╝██║   ██║██╔══██╗██╔══██╗██║
//! ███████╗███████║█████╔╝ ██║   ██║██████╔╝███████║██║
//! ╚════██║██╔══██║██╔═██╗ ██║   ██║██╔══██╗██╔══██║██║
//! ███████║██║  ██║██║  ██╗╚██████╔╝██║  ██║██║  ██║██║
//! ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝
//!
//! =====================================================

#[cfg(test)]
extern crate std;

pub mod btree;
pub mod fixedvec;
pub mod hashmap;
pub mod queue;
pub mod ring;
pub mod stack;

pub use btree::BTree;
pub use fixedvec::FixedVec;
pub use hashmap::HashMap;
pub use queue::Queue;
pub use ring::RingBuffer;
pub use stack::Stack;

#[macro_export]
macro_rules! unlikely {
    ($cond:expr) => {
        core::intrinsics::unlikely($cond)
    };
}

#[macro_export]
macro_rules! likely {
    ($cond:expr) => {
        core::intrinsics::likely($cond)
    };
}
