//! TCM format parser for Geometry Dash bot replays.
//!
//! This library supports both TCM v1 and v2 formats with auto-detection capabilities.
//!
//! # Quick Start
//!
//! ## Auto-Detection (Recommended)
//!
//! ```rust
//! use tcm::{DynamicReplay, meta::Meta};
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("examples/data/restartv1.tcm")?;
//! let mut reader = BufReader::new(file);
//! let replay = DynamicReplay::from_reader(&mut reader)?;
//!
//! println!("TCM v{} - TPS: {}", replay.meta.version_instance(), replay.meta.tps());
//! # Ok(())
//! # }
//! ```
//!
//! ## Type-Specific Usage
//!
//! ```rust
//! use tcm::{meta::{Meta, MetaV2}, replay::{Replay, ReplayDeserializer}};
//!
//! let replay = Replay::<MetaV2>::new_empty(240.0);
//! println!("TPS: {}", replay.meta.tps());
//! ```

pub mod error;
pub mod input;
pub mod meta;
pub mod replay;

pub type Frame = u64;

// Re-export key types for convenience
pub use replay::{DynamicReplay, Replay};
