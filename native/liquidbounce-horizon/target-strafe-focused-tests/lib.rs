#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="math.rs"]pub mod auto_bow_math;
pub use auto_bow_math as target_strafe_math;
#[path="../../liquidbounce-behaviors/src/target_strafe.rs"]pub mod target_strafe;
mod config;
#[path="bridge.rs"]pub mod target_strafe_bridge;
