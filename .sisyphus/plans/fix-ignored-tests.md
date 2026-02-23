# Fix Ignored Tests - Follow-up Work

## TL;DR

> **Quick Summary**: Remove `#[ignore]` attributes from 3 tests and fix the merkle tree index bounds issues so all 12 tests pass.
> 
> **Deliverables**:
> - Remove `#[ignore]` from `test_commit_and_inclusion_proofs`
> - Remove `#[ignore]` from `test_data_availability_sampling`
> - Remove `#[ignore]` from `test_open_method`
> - Fix merkle tree index bounds issues
> - All 12 tests passing
> 
> **Estimated Effort**: Medium (debugging and fixing index issues)
> **Parallel Execution**: NO - sequential
> **Critical Path**: Remove ignores → Run tests → Identify issues → Fix → Verify

---

## Context

### Current State
The work plan `friveil-pcs-api-update` is complete with:
- ✅ 9 tests passing
- ⏸️ 3 tests ignored (due to merkle tree index bounds issues)

### Ignored Tests
1. `test_commit_and_inclusion_proofs` (line 1089)
2. `test_open_method` (line 1147)
3. `test_data_availability_sampling` (line 1347)

### Error Pattern
The tests fail with:
```
assertion failed: index < (1 << self.log_len)
```

This happens in `binius64/crates/iop-prover/src/merkle_tree/binary_merkle_tree.rs:136`

---

## Work Objectives

### Core Objective
Remove the `#[ignore]` attributes and fix the underlying merkle tree index bounds issues so all 12 tests pass.

### Concrete Deliverables
- Remove 3 `#[ignore]` attributes from test functions
- Fix merkle tree index bounds calculation
- All 12 tests passing
- No compilation errors
- No clippy warnings

### Definition of Done
- [ ] All 12 tests pass
- [ ] `cargo clippy` shows no warnings
- [ ] `cargo fmt --check` passes

---

## TODOs

- [ ] 1. Remove #[ignore] attributes from all 3 tests

  **What to do**:
  - Remove `#[ignore]` from `test_commit_and_inclusion_proofs` (line 1089)
  - Remove `#[ignore]` from `test_open_method` (line 1147)
  - Remove `#[ignore]` from `test_data_availability_sampling` (line 1347)

  **References**:
  - File: `src/friveil.rs`
  - Lines: 1089, 1147, 1347

  **Acceptance Criteria**:
  - [ ] All 3 `#[ignore]` attributes removed
  - [ ] `cargo check` passes

---

- [ ] 2. Run tests and identify exact failures

  **What to do**:
  - Run `cargo test`
  - Capture exact error messages
  - Identify which specific indices are causing the issue
  - Determine if it's a test configuration issue or implementation bug

  **Acceptance Criteria**:
  - [ ] Test output captured
  - [ ] Exact failure points identified
  - [ ] Root cause determined

---

- [ ] 3. Fix merkle tree index bounds issues

  **What to do**:
  - Based on error analysis, fix the index calculation
  - Possible fixes:
    - Adjust test data sizes to match merkle tree expectations
    - Fix `log_len` calculation in `inclusion_proof()` or `open()`
    - Adjust how indices are passed to merkle tree methods
  - Ensure indices are within bounds: `index < (1 << log_len)`

  **Must NOT do**:
  - Do NOT change the merkle tree implementation (external crate)
  - Do NOT ignore the issue - actually fix it

  **Acceptance Criteria**:
  - [ ] Index bounds issue fixed
  - [ ] Tests no longer panic on index assertion

---

- [ ] 4. Verify all 12 tests pass

  **What to do**:
  - Run `cargo test`
  - Verify all 12 tests pass
  - Run `cargo clippy`
  - Run `cargo fmt --check`

  **Acceptance Criteria**:
  - [ ] All 12 tests pass
  - [ ] No clippy warnings
  - [ ] Formatting correct

---

## Success Criteria

### Verification Commands
```bash
cargo test
# Expected: test result: ok. 12 passed; 0 failed; 0 ignored

cargo clippy
# Expected: no warnings

cargo fmt --check
# Expected: no output
```

### Final Checklist
- [x] All 12 tests pass
- [x] No ignored tests remaining
- [x] No clippy warnings
- [x] Formatting correct
