//! The slicing and preview intermediate format.
//!
//! Tris is a lightweight, zero-cost intermediate exchange format that bridges
//! file loaders, the slicer, and the preview engine.
//!
//! Every vertex attribute is stored in a container that is binary compatible
//! with the matching engine type, enabling ownership moves instead of
//! per-vertex arithmetic when converting between tris and the preview engine.

pub mod define;

pub use define::*;
pub use glam::{Mat4, Vec3};

#[cfg(feature = "bevy")]
pub mod compat;
