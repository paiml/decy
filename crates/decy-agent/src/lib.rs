//! Background daemon for incremental transpilation and file watching.
//!
//! Provides a long-running process that watches C files and incrementally transpiles changes.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]
