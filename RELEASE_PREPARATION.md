# Release Preparation: Decy v1.0.0

**Target Release Date**: Friday (next available)
**Release Type**: Major milestone - Core Safety Validation Mission Complete
**Ticket**: DECY-067

---

## Release Readiness Status

### ✅ Completed Preparation Tasks

1. **Release Policy Established**
   - Added Friday-only release policy to CLAUDE.md
   - Documented rationale: Toyota Way, blast radius containment, predictability
   - Exception process for security patches defined

2. **Roadmap Updated**
   - DECY-067 created as release preparation ticket
   - Type changed from "release" to "release_preparation"
   - Status: in_progress, Phase: GREEN

3. **Crate Metadata Verified**
   - All 13 crates have correct metadata structure
   - Version: 1.0.0 (workspace inheritance)
   - Descriptions: ✅ All present
   - License: MIT OR Apache-2.0 (workspace)
   - Repository, homepage, documentation: ✅ (workspace)
   - Keywords and categories: ✅ (workspace)

4. **CHANGELOG.md Complete**
   - Comprehensive v1.0.0 changelog ready
   - 12 CWE classes documented
   - 200+ integration tests documented
   - 40,000+ property test executions documented
   - Real-world CVE prevention highlighted
   - Safety transformation table complete

### ⚠️ Blocked Tasks (Require Network Access)

**Current Blocker**: crates.io access returns 403 Forbidden
**Error**: `failed to get successful HTTP response from https://index.crates.io/config.json (21.0.0.153), got 403`

**Tasks Blocked**:
1. ❌ Quality gates execution (requires cargo clippy, cargo test)
2. ❌ Dry-run publish (`cargo publish --dry-run`)
3. ❌ Build verification

**Resolution Required Before Friday**:
- Network/proxy configuration fix
- Alternative: Use CI/GitHub Actions for validation
- Verify tests pass: Last known passing commit: `6f173a0` (Release v1.0.0)

### 📋 Pending Tasks (For Friday)

1. **Quality Gates Verification**
   - Run `make quality-gates` and ensure all pass
   - Target: 0 clippy warnings, all tests passing, coverage ≥80%
   - Known from commit 6f173a0: Should pass (v1.0.0 was validated)

2. **Dry-Run Publish**
   ```bash
   for crate in crates/*/; do
       (cd "$crate" && cargo publish --dry-run)
   done
   ```

3. **Final Validation**
   - Confirm all 13 crates pass dry-run
   - Verify dependency order is correct
   - Check that decy-parser publishes first (no dependencies)

4. **Actual Publication (Friday Only)**
   - Follow CLAUDE.md → Release Policy → Friday Release Checklist
   - Publish in dependency order (see below)

---

## Publication Order (Dependency-Aware)

Based on Cargo.toml analysis, publish in this exact order:

```bash
# Tier 1: No dependencies (foundational)
cargo publish -p decy-parser

# Tier 2: Depends only on parser
cargo publish -p decy-hir

# Tier 3: Depends on hir
cargo publish -p decy-analyzer

# Tier 4: Depends on analyzer
cargo publish -p decy-ownership

# Tier 5: Depends on ownership
cargo publish -p decy-verify

# Tier 6: Depends on verify
cargo publish -p decy-codegen

# Tier 7: Depends on multiple crates
cargo publish -p decy-core        # Depends on parser, hir, analyzer, ownership, verify, codegen
cargo publish -p decy-book        # Depends on core
cargo publish -p decy-agent       # Depends on core
cargo publish -p decy-mcp         # Depends on core
cargo publish -p decy-repo        # Depends on core
cargo publish -p decy-debugger    # Depends on parser, hir

# Tier 8: CLI (depends on everything)
cargo publish -p decy            # Depends on core, verify, agent, mcp, repo, book
```

**Wait Time**: Allow 30-60 seconds between tiers for crates.io index to update.

---

## Friday Release Checklist

Use this checklist on Friday (from CLAUDE.md):

- [ ] Verify today is Friday (critical!)
- [ ] Network/crates.io access working
- [ ] All roadmap tickets are `status: done`, `phase: DONE`
- [ ] Run `make quality-gates` → passes with 0 warnings
- [ ] Coverage ≥80% maintained
- [ ] Run dry-run publish for all crates → all succeed
- [ ] CHANGELOG.md finalized
- [ ] Git tag created: `git tag v1.0.0`
- [ ] Branch clean: `git status` shows no uncommitted changes

**Publication Steps**:

1. **Final Quality Gates**
   ```bash
   make quality-gates
   ```

2. **Dry-Run All Crates**
   ```bash
   for crate in crates/*/; do
       echo "Checking $(basename $crate)..."
       (cd "$crate" && cargo publish --dry-run)
   done
   ```

3. **Publish in Dependency Order** (see Publication Order above)
   - Start with `cargo publish -p decy-parser`
   - Wait 60 seconds between tiers
   - End with `cargo publish -p decy`

4. **Verify Installation**
   ```bash
   cargo install decy
   decy --version  # Should show 1.0.0
   ```

5. **Create GitHub Release**
   ```bash
   git tag v1.0.0
   git push origin claude/continue-work-011CUoG26Bh8UbvYU6NrT6RB --tags
   gh release create v1.0.0 --title "Decy v1.0.0: Core Safety Validation Mission Complete 🎉" --notes-file CHANGELOG.md
   ```

6. **Update Roadmap**
   - Edit `roadmap.yaml`: Set DECY-067 `status: done`, `phase: DONE`
   - Commit: `git commit -m "DECY-067: Release v1.0.0 complete"`

7. **Update README Badges** (if applicable)
   - Add crates.io version badge
   - Add docs.rs badge
   - Add download stats badge

---

## Known Issues & Resolutions

### Issue 1: crates.io 403 Access Denied

**Status**: Active blocker
**Impact**: Cannot run quality gates or dry-run publish locally
**Workaround**: Use GitHub Actions CI for validation
**Resolution**: Network/proxy configuration (outside code scope)
**Mitigation**: Last verified passing: commit `6f173a0` (2025-11-04)

### Issue 2: Unsafe Code Above Target

**From quality gates output**:
- Current: 12.33 unsafe/1000 LOC
- Target: <5 unsafe/1000 LOC
- Concern: 153 unsafe instances outside decy-parser

**Status**: Known, documented in roadmap
**Impact**: v1.0.0 milestone achieves 0 unsafe for **safety patterns** (not codebase-wide)
**Mitigation**: Safety patterns have 0 unsafe (goal achieved)
**Future Work**: Sprint 20+ will reduce overall unsafe count

---

## Success Criteria for Friday Release

- [ ] All 13 crates published to crates.io successfully
- [ ] `cargo install decy` works from crates.io
- [ ] Documentation at docs.rs builds and renders correctly
- [ ] GitHub release v1.0.0 created with CHANGELOG
- [ ] Roadmap DECY-067 marked as DONE
- [ ] No critical bugs discovered during publication

---

## Emergency Rollback Plan

If critical issues discovered during Friday publication:

1. **Stop publication** immediately (Andon Cord principle)
2. **Yank problematic crate versions**: `cargo yank --vers 1.0.0 <crate>`
3. **Create P0 bug ticket** in roadmap.yaml
4. **Fix with EXTREME TDD**: RED-GREEN-REFACTOR
5. **Re-validate**: quality gates + dry-run
6. **Defer to next Friday**: Don't rush, maintain quality

---

## Contact & Support

**Maintainer**: Claude (DECY-067 assignee)
**Release Policy**: See CLAUDE.md → Release Policy
**Issue Tracker**: roadmap.yaml (PMAT-driven)
**Methodology**: EXTREME TDD + Toyota Way + PMAT

---

## Notes

**Date Prepared**: 2025-11-04 (Monday)
**Preparation Phase**: Monday-Thursday
**Target Release**: Friday (next available)
**Confidence Level**: High (v1.0.0 validated in commit 6f173a0)

**Risk Assessment**:
- **Low**: Preparation comprehensive, v1.0.0 already validated
- **Medium**: Network blocker requires resolution
- **Mitigation**: CI/CD can validate quality gates

**Next Steps**:
1. Resolve network/crates.io access issue
2. Run full quality gates suite
3. Execute dry-run publish for all crates
4. Wait for Friday
5. Execute release checklist
