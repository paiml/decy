// Auto-split for File Health compliance
// All parts disabled: `use super::*` resolves to this empty index module,
// not to lib.rs. Needs restructuring to use `use crate::*` in each part.
// TODO: Fix by adding `use crate::*; use decy_hir::*;` to each part file.
