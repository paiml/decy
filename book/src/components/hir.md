# HIR (High-level Intermediate Representation) Verification

The HIR is DECY's intermediate representation between the C AST and Rust code generation. It simplifies and normalizes C constructs for easier analysis.

## Purpose

The HIR serves as:

1. **Normalization layer**: Converts complex C syntax into simpler forms
2. **Analysis target**: Provides a clean structure for dataflow and ownership analysis
3. **Decoupling**: Separates parsing from code generation

## Architecture

```
C AST → HIR Lowering → HIR → Analysis → Annotated HIR → Codegen
```

## Summary

HIR verification ensures correct lowering from AST to a simplified intermediate representation suitable for ownership and lifetime analysis.

## Next Steps

- [Dataflow Analysis](./dataflow.md) - Analyzing variable usage in HIR
- [Ownership Inference](./ownership.md) - Determining ownership patterns
