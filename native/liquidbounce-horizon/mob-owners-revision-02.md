# MobOwners revision 02

The aggregate 95-03 native check reached the actual Game implementation and
reported the LAN guest's signed `entity_id: i32` compared against Horizon's
unsigned projectile-owner identity `u32`. The guest comparison now casts the
signed value to u32, exactly like the local remote-player comparison directly
above it. Rust's `as u32` preserves all 32 protocol bits, including negative
entity IDs; no checked conversion, absolute value, or numeric clamp is used.

This is a native type correction outside the focused declaration-only fixture.
The original smoke and original freeze remain historical evidence. Aggregate
compilation after this correction is owned by the root task.
