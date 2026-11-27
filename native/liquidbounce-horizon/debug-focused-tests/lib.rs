#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="../../liquidbounce-behaviors/src/tracers_curve.rs"]pub mod tracers_curve;
pub mod tracers{pub use crate::tracers_curve as curve;}
#[path="../../liquidbounce-behaviors/src/debug.rs"]pub mod debug;
mod config;
