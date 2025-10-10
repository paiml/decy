//! Book-based verification for transpiled Rust code.
//!
//! Uses mdBook to verify transpilation by compiling generated Rust and ensuring it passes lint.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]
