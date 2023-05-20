//! [ArceOS](https://github.com/rcore-os/arceos) synchronization primitives.
//!
//! Currently supported primitives:
//!
//! - [`Mutex`]: A mutual exclusion primitive.
//! - mod [`spin`](spinlock): spin-locks.
//!
//! # Cargo Features
//!
//! - `multitask`: For use in the multi-threaded environments. If the feature is
//!   not enabled, [`Mutex`] will be an alias of [`spin::SpinNoIrq`]. This
//!   feature is enabled by default.

#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]

pub use spinlock as spin;

#[cfg(feature = "multitask")]
mod mutex;

#[cfg(feature = "multitask")]
#[doc(cfg(feature = "multitask"))]
pub use self::mutex::{Mutex, MutexGuard};

#[cfg(not(feature = "multitask"))]
#[doc(cfg(not(feature = "multitask")))]
pub use spinlock::{SpinNoIrq as Mutex, SpinNoIrqGuard as MutexGuard};
