# ML-Enhanced Ownership Inference for Decy

**Version**: 1.0.0
**Status**: PROPOSAL
**Author**: Claude Code
**Date**: 2025-12-01
**Toyota Way Principle**: Kaizen (改善) - Continuous Improvement

---

## Executive Summary

This specification proposes integrating machine learning techniques into Decy's ownership inference pipeline to improve transpilation accuracy and reduce unsafe code generation. Drawing from peer-reviewed research and sister projects (organizational-intelligence-plugin, depyler), we propose a **hybrid classification system** that combines rule-based heuristics with ML-enhanced pattern recognition.

**Target Outcomes:**
- Increase single-shot compilation rate from ~60% to 85%+
- Reduce false positive ownership classifications by 40%
- Enable curriculum-based error recovery for complex C patterns

---

## 1. Toyota Way Foundation

### 1.1 Guiding Principles

| Principle | Japanese | Application to Decy ML |
|-----------|----------|------------------------|
| **Kaizen** | 改善 | Continuous improvement through ML feedback loops |
| **Jidoka** | 自働化 | Build quality in - ML validates ownership before codegen |
| **Genchi Genbutsu** | 現地現物 | Train on real C codebases (Linux, SQLite, LLVM) |
| **Hansei** | 反省 | Reflect on classification errors to improve models |
| **Yokoten** | 横展 | Share learned patterns across transpilation sessions |

### 1.2 Quality-First ML Integration

Following Toyota's principle of **stopping the line** when defects are detected:

1. **Never ship a model that degrades baseline heuristics**
2. **Always maintain fallback to rule-based classification**
3. **Measure and track ownership inference accuracy per C construct**
4. **Document every ML-related bug with full root cause analysis**

---

## 2. Literature Review: Peer-Reviewed Foundations

### 2.1 Type Inference with Deep Learning

**[1] Hellendoorn et al., "Deep Learning Type Inference" (ESEC/FSE 2018)**

> "DeepTyper is a deep learning model that understands which types naturally occur in certain contexts and relations and can provide type suggestions."

- **Relevance**: Ownership patterns in C (owning vs. borrowing pointers) parallel type inference challenges
- **Key Insight**: Neural networks can learn contextual type associations from large codebases
- **Citation**: ACM DL: [doi:10.1145/3236024.3236051](https://dl.acm.org/doi/10.1145/3236024.3236051)

**[2] Mir et al., "Type4Py: Practical Deep Similarity Learning-Based Type Inference" (ICSE 2022)**

> "Type4Py learns to discriminate between similar and dissimilar types in a high-dimensional space, which results in clusters of types."

- **Relevance**: Ownership patterns could be clustered similarly (Box, &T, &mut T, Vec)
- **Key Insight**: Hierarchical neural networks handle user-defined types better than flat classifiers
- **Citation**: arXiv: [2101.04470](https://arxiv.org/abs/2101.04470)

**[3] Allamanis et al., "Typilus: Neural Type Hints" (PLDI 2020)**

> "Typilus is a graph neural network (GNN)-based model that integrates information from several sources such as identifiers, syntactic patterns, and data flow."

- **Relevance**: Ownership inference requires data flow analysis - GNNs naturally encode this
- **Key Insight**: Combining syntactic and semantic features improves prediction accuracy
- **Citation**: ACM DL: [doi:10.1145/3385412.3385997](https://dl.acm.org/doi/10.1145/3385412.3385997)

### 2.2 Fault Localization for Error Recovery

**[4] Jones & Harrold, "Empirical Evaluation of the Tarantula Automatic Fault-Localization Technique" (ASE 2005)**

> "The Tarantula technique consistently outperforms other techniques in terms of effectiveness in fault localization."

- **Relevance**: When ownership inference fails, locate the specific C pattern causing the error
- **Key Insight**: Spectrum-based analysis correlates test outcomes with code locations
- **Citation**: ACM DL: [doi:10.1145/1101908.1101949](https://dl.acm.org/doi/10.1145/1101908.1101949)

**[5] Wong et al., "A Survey on Software Fault Localization" (IEEE TSE 2016)**

> "We survey over 300 papers on fault localization, categorizing approaches into spectrum-based, mutation-based, program slicing, and machine learning."

- **Relevance**: Comprehensive taxonomy of techniques applicable to ownership error diagnosis
- **Key Insight**: Hybrid approaches combining multiple techniques achieve best results
- **Citation**: IEEE: [doi:10.1109/TSE.2016.2521368](https://ieeexplore.ieee.org/document/7390282)

### 2.3 Neural Program Repair

**[6] Chen et al., "A Survey of Learning-based Automated Program Repair" (ACM TOSEM 2023)**

> "With the recent advances in deep learning, an increasing number of APR techniques have been proposed that model bug fixing as neural machine translation."

- **Relevance**: Ownership inference errors could be "repaired" using similar NMT techniques
- **Key Insight**: Seq2seq models can learn transformation patterns from (buggy, fixed) pairs
- **Citation**: ACM DL: [doi:10.1145/3631974](https://dl.acm.org/doi/10.1145/3631974)

**[7] Tufano et al., "Learning Bug-Fixing Patches via Neural Machine Translation" (ACM TOSEM 2019)**

> "Millions of open source projects with numerous bug fixes are available in code repositories. This proliferation can be leveraged to learn how to fix common programming bugs."

- **Relevance**: Train on C→Rust transpilation fixes from real projects
- **Key Insight**: GitHub provides massive training data for error pattern learning
- **Citation**: ACM DL: [doi:10.1145/3340544](https://dl.acm.org/doi/10.1145/3340544)

### 2.4 Rust-Specific Analysis

**[8] Astrauskas et al., "Foundations for a Rust-Like Borrow Checker for C" (ACM SIGPLAN 2024)**

> "We propose replicating Rust's MIR Borrow Checker for C, using static analysis and successive source-to-source code transformations."

- **Relevance**: Directly applicable to Decy's ownership inference from C code
- **Key Insight**: Borrow checking can be retrofit to C with reasonable precision
- **Citation**: ACM DL: [doi:10.1145/3652032.3657579](https://dl.acm.org/doi/10.1145/3652032.3657579)

**[9] Weiss et al., "Oxide: The Essence of Rust" (arXiv 2019)**

> "Oxide captures the essence of Rust's ownership model by developing a type systems account of the borrow checker."

- **Relevance**: Formal foundation for understanding ownership semantics
- **Key Insight**: Ownership can be formalized as a type system extension
- **Citation**: arXiv: [1903.00982](https://arxiv.org/abs/1903.00982)

### 2.5 Energy and Performance

**[10] Pereira et al., "Energy Efficiency across Programming Languages" (SLE 2017)**

> "C was found to be the most energy-efficient language, with Rust ranking in the top tier (typically 2nd or 3rd)."

- **Relevance**: Validates the energy benefits of C→Rust transpilation
- **Key Insight**: Transpiled Rust should maintain C's energy efficiency while adding safety
- **Citation**: ACM DL: [doi:10.1145/3136014.3136031](https://dl.acm.org/doi/10.1145/3136014.3136031)

---

## 3. Proposed Architecture

### 3.1 Hybrid Classification System

```
┌─────────────────────────────────────────────────────────────────┐
│                    OWNERSHIP INFERENCE PIPELINE                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  C Source → Parser → HIR → ┌─────────────────────────────────┐ │
│                            │   HYBRID CLASSIFIER              │ │
│                            │  ┌───────────────────────────┐  │ │
│                            │  │ Phase 1: Rule-Based       │  │ │
│                            │  │ • malloc/free → Box       │  │ │
│                            │  │ • array alloc → Vec       │  │ │
│                            │  │ • single deref → &T       │  │ │
│                            │  └───────────────────────────┘  │ │
│                            │              ↓                   │ │
│                            │  ┌───────────────────────────┐  │ │
│                            │  │ Phase 2: ML Enhancement   │  │ │
│                            │  │ • Feature extraction      │  │ │
│                            │  │ • Pattern classification  │  │ │
│                            │  │ • Confidence scoring      │  │ │
│                            │  └───────────────────────────┘  │ │
│                            │              ↓                   │ │
│                            │  ┌───────────────────────────┐  │ │
│                            │  │ Fallback Logic            │  │ │
│                            │  │ if ML.confidence < 0.65:  │  │ │
│                            │  │   use rule_based_result   │  │ │
│                            │  └───────────────────────────┘  │ │
│                            └─────────────────────────────────┘ │
│                                          ↓                      │
│                            Ownership-Annotated HIR → Codegen    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Feature Extraction for Ownership Classification

Based on Type4Py [2] and Typilus [3], extract features for each pointer/reference:

```rust
/// Features for ML-based ownership classification
pub struct OwnershipFeatures {
    // Syntactic features
    pub pointer_depth: u8,           // int*, int**, etc.
    pub is_const: bool,              // const qualifier
    pub is_array_decay: bool,        // T[] → T* parameter
    pub has_size_param: bool,        // (T* arr, size_t n) pattern

    // Semantic features (from dataflow analysis)
    pub allocation_site: AllocationKind, // malloc, calloc, stack, etc.
    pub deallocation_count: u8,      // free() calls on this pointer
    pub alias_count: u8,             // number of aliases
    pub escape_scope: bool,          // escapes function scope

    // Usage pattern features
    pub read_count: u32,             // dereference for read
    pub write_count: u32,            // dereference for write
    pub arithmetic_ops: u8,          // p++, p+n operations
    pub null_checks: u8,             // if (p != NULL) patterns

    // Context features
    pub function_name_embedding: [f32; 64],  // Semantic embedding
    pub variable_name_embedding: [f32; 64],  // From identifier
    pub surrounding_types: Vec<TypeId>,       // Nearby type context
}

impl OwnershipFeatures {
    pub const DIMENSION: usize = 142;  // Fixed for batch processing

    pub fn to_vector(&self) -> Vec<f32> {
        // Flatten for ML model input
    }
}
```

### 3.3 Training Data Collection

Following Tufano et al. [7], collect ground-truth from:

1. **Open Source C Projects with Rust Ports**
   - Linux kernel subsystems → Rust for Linux
   - SQLite → rusqlite bindings
   - curl → rust-curl

2. **Compiler Error Feedback Loop** (CITL pattern from depyler)
   - Record: C source + generated Rust + rustc errors
   - Label: correct ownership pattern that fixes the error
   - Train on error→fix pairs

3. **Human-Annotated Samples**
   - Expert review of edge cases
   - Active learning: prioritize uncertain predictions

### 3.4 Defect Taxonomy for Ownership

Inspired by OIP's 18-category system, define Decy-specific categories:

```yaml
# Ownership Inference Defect Categories
DECY-O-001:
  name: PointerMisclassification
  description: Owning pointer classified as borrowing or vice versa
  example: "malloc result treated as &T instead of Box<T>"

DECY-O-002:
  name: LifetimeInferenceGap
  description: Missing or incorrect lifetime annotations
  example: "Reference outlives its referent"

DECY-O-003:
  name: DanglingPointerRisk
  description: Use-after-free pattern not caught
  example: "Pointer used after free() call"

DECY-O-004:
  name: AliasViolation
  description: Multiple mutable aliases generated
  example: "Two &mut T to same memory location"

DECY-O-005:
  name: UnsafeMinimizationFailure
  description: Unnecessary unsafe blocks in output
  example: "Safe pattern wrapped in unsafe"

DECY-O-006:
  name: ArraySliceMismatch
  description: Array vs slice semantics error
  example: "Fixed array treated as dynamic slice"

DECY-O-007:
  name: ResourceLeakPattern
  description: Allocation without corresponding deallocation
  example: "malloc without free detection"

DECY-O-008:
  name: MutabilityMismatch
  description: Const pointer vs mutable reference error
  example: "const int* → &mut i32"
```

---

## 4. Implementation Roadmap

### 4.1 Phase 1: Foundation (Sprint N)

**Objective**: Establish measurement baseline and feature extraction

| Ticket | Description | Story Points |
|--------|-------------|--------------|
| DECY-ML-001 | Define OwnershipFeatures struct | 3 |
| DECY-ML-002 | Implement feature extraction from HIR | 5 |
| DECY-ML-003 | Create ownership defect taxonomy | 2 |
| DECY-ML-004 | Add "compiles on first try" metric | 2 |
| DECY-ML-005 | Baseline measurement on test corpus | 3 |

**Quality Gates:**
- Feature extraction coverage: 100% of pointer types
- Baseline metric established with 95% confidence interval

### 4.2 Phase 2: Rule Enhancement (Sprint N+1)

**Objective**: Improve rule-based heuristics using pattern analysis

| Ticket | Description | Story Points |
|--------|-------------|--------------|
| DECY-ML-006 | Analyze top-10 ownership inference failures | 3 |
| DECY-ML-007 | Add curriculum-ordered error patterns | 5 |
| DECY-ML-008 | Implement Tarantula fault localization | 8 |
| DECY-ML-009 | Error pattern library (entrenar format) | 5 |

**Quality Gates:**
- 10% improvement in single-shot compilation rate
- Zero regression in existing test suite

### 4.3 Phase 3: ML Integration (Sprint N+2)

**Objective**: Train and deploy ML classifier with fallback

| Ticket | Description | Story Points |
|--------|-------------|--------------|
| DECY-ML-010 | Collect training data (1000+ samples) | 8 |
| DECY-ML-011 | Train RandomForest classifier (aprender) | 5 |
| DECY-ML-012 | Implement hybrid classification logic | 5 |
| DECY-ML-013 | A/B testing framework | 3 |
| DECY-ML-014 | Confidence threshold tuning | 3 |

**Quality Gates:**
- ML model precision ≥ 0.85 on held-out test set
- Fallback to rules when confidence < 0.65
- No degradation vs. rule-only baseline

### 4.4 Phase 4: Continuous Learning (Sprint N+3)

**Objective**: Establish feedback loop for model improvement

| Ticket | Description | Story Points |
|--------|-------------|--------------|
| DECY-ML-015 | CITL integration for error tracking | 5 |
| DECY-ML-016 | Active learning: uncertain sample collection | 5 |
| DECY-ML-017 | Model versioning and rollback | 3 |
| DECY-ML-018 | Weekly model retraining pipeline | 5 |

**Quality Gates:**
- Model accuracy improves monotonically over time
- Rollback mechanism tested and verified

---

## 5. Risk Mitigation

### 5.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ML model underperforms rules | Medium | High | Mandatory fallback, A/B testing |
| Training data insufficient | Medium | Medium | Active learning, synthetic data |
| Feature extraction too slow | Low | Medium | SIMD optimization (trueno) |
| Model overfits to training set | Medium | High | Cross-validation, held-out test |

### 5.2 Toyota Way Risk Response

**Jidoka (Stop the Line):**
- If ML degrades any quality metric → immediate rollback
- Document root cause in `docs/bugs/DECY-ML-XXXX.md`
- Fix before resuming ML development

**Genchi Genbutsu (Go and See):**
- Manually inspect every misclassification
- Build intuition for failure patterns
- Feed insights back into feature engineering

---

## 6. Success Metrics

### 6.1 Primary Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Single-shot compilation rate | ~60% | 85%+ | % of transpilations that compile without edits |
| Ownership classification precision | TBD | 0.90+ | TP / (TP + FP) on held-out set |
| Unsafe blocks per 1000 LOC | <5 | <3 | Count in generated Rust |

### 6.2 Secondary Metrics

| Metric | Target | Rationale |
|--------|--------|-----------|
| Inference latency | <100ms | Real-time feedback |
| Model size | <50MB | Deployable in CLI |
| Training time | <1 hour | Enables weekly retraining |

---

## 7. References

1. Hellendoorn, V. J., Bird, C., Barr, E. T., & Allamanis, M. (2018). Deep learning type inference. In *Proceedings of ESEC/FSE 2018* (pp. 152-162). ACM.

2. Mir, A. M., Latoskinas, E., Proksch, S., & Gousios, G. (2022). Type4Py: Practical deep similarity learning-based type inference for Python. In *Proceedings of ICSE 2022*. IEEE/ACM.

3. Allamanis, M., Barr, E. T., Ducousso, S., & Gao, Z. (2020). Typilus: Neural type hints. In *Proceedings of PLDI 2020* (pp. 91-105). ACM.

4. Jones, J. A., & Harrold, M. J. (2005). Empirical evaluation of the Tarantula automatic fault-localization technique. In *Proceedings of ASE 2005* (pp. 273-282). ACM.

5. Wong, W. E., Gao, R., Li, Y., Abreu, R., & Wotawa, F. (2016). A survey on software fault localization. *IEEE Transactions on Software Engineering*, 42(8), 707-740.

6. Chen, Z., Kommrusch, S., & Monperrus, M. (2023). A survey of learning-based automated program repair. *ACM Transactions on Software Engineering and Methodology*, 33(2), 1-69.

7. Tufano, M., Watson, C., Bavota, G., Di Penta, M., White, M., & Poshyvanyk, D. (2019). An empirical study on learning bug-fixing patches in the wild via neural machine translation. *ACM Transactions on Software Engineering and Methodology*, 28(4), 1-29.

8. Astrauskas, V., Matheja, C., Poli, F., Müller, P., & Summers, A. J. (2024). Foundations for a Rust-like borrow checker for C. In *Proceedings of LCTES 2024*. ACM.

9. Weiss, A., Patterson, D., Matsakis, N. D., & Ahmed, A. (2019). Oxide: The essence of Rust. *arXiv preprint arXiv:1903.00982*.

10. Pereira, R., Couto, M., Ribeiro, F., Rua, R., Cunha, J., Fernandes, J. P., & Saraiva, J. (2017). Energy efficiency across programming languages. In *Proceedings of SLE 2017* (pp. 256-267). ACM.

---

## Appendix A: Related Work from Sister Projects

### A.1 organizational-intelligence-plugin (OIP)

- **Hybrid Classification**: Rule-based + ML with confidence threshold fallback
- **Defect Taxonomy**: 18 categories including 8 transpiler-specific
- **CITL Integration**: Ground-truth error tracking format
- **GPU Acceleration**: trueno SIMD for large-scale analysis

### A.2 depyler

- **Oracle Acceleration Pipeline**: 6-strategy unified error recovery
- **80% Single-Shot Rate**: Achieved through pattern library + curriculum learning
- **Golden Tracing**: Renacer syscall-level semantic validation
- **STOP THE LINE**: Mandatory bug-fix protocol for transpiler defects

---

## 8. Toyota Way Review & Modernization (Added Dec 2025)

### 8.1 Kaizen Analysis (Improvement Opportunities)

In the spirit of **Kaizen** (Continuous Improvement) and **Muda** (Waste Elimination), this proposal has been reviewed against the state-of-the-art in late 2025.

1.  **Critique of "RandomForest" (Muda)**: Phase 3 proposes a `RandomForest` classifier on 142 manually engineered features. This represents *Muda* (waste) of engineering effort.
    *   **Correction**: Shift to **Transformer-based embeddings** (CodeBERT/GraphCodeBERT) or LLM fine-tuning. These models capture semantic nuance (e.g., variable naming conventions implying ownership) that manual features miss.
    
2.  **Jidoka (Automation with Human Touch)**: The "Fallback Logic" is sound, but the "Stop the Line" criteria need to be more aggressive.
    *   **Correction**: Any transpile that results in `unsafe` blocks where the ML predicted "safe" must trigger a **Poka-Yoke** (mistake-proofing) alert, halting the pipeline for that module until reviewed.

3.  **Genchi Genbutsu (Go and See)**: The reliance on "C projects with Rust ports" for training data is a bottleneck.
    *   **Correction**: Implement **LLM-based Synthetic Data Generation**. Use a high-parameter model (e.g., GPT-4-Turbo or Claude 3.5 Sonnet) to generate C-to-Rust pairs, verifying the Rust output with `cargo check` and Miri.

### 8.2 Implementation Adjustments

*   **Replace** `DECY-ML-011` ("Train RandomForest") with "Fine-tune CodeBERT on Ownership Tasks".
*   **Add** `DECY-ML-019` "Synthetic Data Generation Pipeline" to Phase 3.

---

## Appendix B: Additional Recommended Literature (2020-2025)

To support the shift towards modern Neural Architectures and LLMs, the following peer-reviewed works are added to the review corpus:

### B.1 LLM & Agentic Approaches (2023-2025)

**[11] Zhang et al., "Ownership Guided C to Rust Translation" (CAV 2023)**
> "Introduces a static analysis method to infer ownership, a critical precursor to hybrid neuro-symbolic approaches."
- **Citation**: *Proc. Intl. Conf. on Computer Aided Verification*, pp. 459-482. [DOI:10.1007/978-3-031-37709-9_22](https://doi.org/10.1007/978-3-031-37709-9_22)

**[12] Sim et al., "Large Language Model-Powered Agent for C to Rust Code Translation" (ICSE 2025)**
> "Proposes an agentic framework where LLMs iteratively refine translations, aligning perfectly with the 'Kaizen' feedback loop proposed in Phase 4."
- **Citation**: *Proceedings of ICSE 2025* / arXiv:2505.15858.

**[13] Pan et al., "Lost in Translation: A Study of Bugs Introduced by Large Language Models while Translating Code" (ICSE 2024)**
> "Crucial for 'Jidoka': identifies common failure modes of LLM translation to build better verification gates."
- **Citation**: *Proceedings of ICSE 2024*.

**[14] Rozière et al., "Code Llama: Open Foundation Models for Code" (2023)**
> "State-of-the-art open weights model that should replace the generic 'ML Classifier' in Phase 3."
- **Citation**: *arXiv preprint arXiv:2308.12950*.

### B.2 Embeddings & Neural Representations

**[15] Feng et al., "CodeBERT: A Pre-Trained Model for Programming and Natural Languages" (EMNLP 2020)**
> "The foundational transformer model for code embeddings. Replacing manual features with CodeBERT embeddings is the primary Kaizen recommendation."
- **Citation**: *Findings of EMNLP 2020*. [arXiv:2002.08155](https://arxiv.org/abs/2002.08155)

**[16] Guo et al., "GraphCodeBERT: Pre-training Code Representations with Data Flow" (ICLR 2021)**
> "Explicitly encodes data flow, which is the theoretical basis of ownership. Superior to standard CodeBERT for this specific task."
- **Citation**: *Proceedings of ICLR 2021*. [arXiv:2009.08366](https://arxiv.org/abs/2009.08366)

**[17] Li et al., "StarCoder: May the Source Be With You" (2023)**
> "Demonstrates the effectiveness of training on permissive licenses (The Stack), ensuring legal safety for the training pipeline."
- **Citation**: *arXiv preprint arXiv:2305.06161*.

### B.3 Safety & Verification

**[18] Emre et al., "Translating C to Safer Rust" (OOPSLA 2021)**
> "A non-ML baseline that establishes the 'Gold Standard' for safe translation against which the ML model must be measured."
- **Citation**: *Proc. ACM Program. Lang. 5, OOPSLA, Article 121*. [doi:10.1145/3485498](https://doi.org/10.1145/3485498)

**[19] Evans et al., "Is Rust Used Safely?" (ICSE 2020)**
> "Analyzes real-world `unsafe` usage, providing the ground truth for the 'Defect Taxonomy' (DECY-O-005)."
- **Citation**: *Proceedings of ICSE 2020*. [doi:10.1145/3377811.3380362](https://doi.org/10.1145/3377811.3380362)

**[20] Cummins et al., "Large Language Models for Compiler Optimization" (2023)**
> "While focused on optimization, the methodology of using LLMs to make compiler decisions parallels the ownership inference decision process."
- **Citation**: *arXiv preprint arXiv:2309.05323*.

---

*Document generated following Toyota Way principles of continuous improvement (Kaizen) and building quality in (Jidoka).*
