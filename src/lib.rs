//! Rust port of TCM (tcbot macro) format
//!
//! Made specifically for tcbot.pro Geometry Dash bot.
//! This library provides functionality to parse and serialize TCM format.
//! Meant to be used in converters, editors, etc.

pub mod input;
pub mod meta;
pub mod replay;

pub type Frame = u64;
