---
name: Ticket
about: PMAT-qualified ticket from roadmap.yaml
title: 'DECY-XXX: '
labels: ticket
assignees: ''

---

## Ticket Information
**Ticket ID:** DECY-XXX
**Sprint:** X
**Story Points:** X
**Priority:** critical/high/medium/low
**Type:** feature/bug/refactor

## Description
<!-- From roadmap.yaml -->

## Requirements
<!-- From roadmap.yaml -->
- [ ] Requirement 1
- [ ] Requirement 2

## Test Requirements

### Unit Tests
- [ ] test_1
- [ ] test_2

### Property Tests
- [ ] property_1
- [ ] property_2

### Doctests
- [ ] doctest_1

### Examples
- [ ] examples/example.rs

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Coverage ≥80%
- [ ] 0 clippy warnings
- [ ] 0 SATD comments

## RED-GREEN-REFACTOR Workflow

### RED Phase
- [ ] Write failing tests
- [ ] Commit: `[RED] DECY-XXX: Add failing tests`

### GREEN Phase
- [ ] Implement minimal solution
- [ ] Make tests pass
- [ ] Commit: `[GREEN] DECY-XXX: Implement feature`

### REFACTOR Phase
- [ ] Improve code quality
- [ ] Meet quality gates
- [ ] Commit: `[REFACTOR] DECY-XXX: Meet quality gates`

## Definition of Done
- [ ] RED phase complete with failing tests
- [ ] GREEN phase complete with passing tests
- [ ] REFACTOR phase complete with quality gates met
- [ ] Coverage ≥ 80%
- [ ] Mutation kill rate ≥ 90% (by Sprint 5)
- [ ] 0 clippy warnings
- [ ] 0 SATD comments
- [ ] All tests passing (unit, property, integration, doctest, examples)
- [ ] Documentation complete
- [ ] Code review approved
- [ ] CI pipeline green

## Roadmap Reference
See `roadmap.yaml` for full ticket specification.
