//! The original LiquidBounce web theme rendered by CEF's windowless OnPaint path.
//! The backend URL serves the unmodified theme and its REST/WebSocket protocol.
pub mod compositor;
pub mod input;
pub mod paint;

#[cfg(feature = "horizon-gpu")]
pub mod horizon;

#[cfg(all(test, feature = "gpu-tests"))]
mod gpu_tests;
#[cfg(feature = "cef-runtime")]
pub mod runtime;
