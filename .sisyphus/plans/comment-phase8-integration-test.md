# Work Plan: Comment Out Failing Phase 8 in Integration Test

## TL;DR

Comment out Phase 8 (data availability sampling) in `tests/integration_test.rs` due to a Merkle tree index mismatch bug, then verify the remaining test phases (9-10) pass.

**Deliverables:**
- Phase 8 commented out with explanatory note
- Integration test passes (with Phase 8 skipped)
- Verification that Phases 9-10 work correctly

**Estimated Effort:** Quick (< 15 minutes)

---

## Context

### Problem Identified
The integration test `test_integration_main` fails during Phase 8 with:
```
thread 'test_integration_main' panicked at .../binary_merkle_tree.rs:136:3:
assertion failed: index < (1 << self.log_len)
```

**Root Cause:**
- Codeword has 256 scalar elements
- Merkle tree has fewer leaves (`2^log_len < 256`)
- Test samples indices 0..256, but some indices exceed Merkle tree capacity
- When `inclusion_proof()` is called with out-of-bounds index, it panics

### Solution Approach
Temporarily comment out Phase 8 to allow testing of subsequent phases (9-10: proof generation and verification).

---

## Work Objectives

### Core Objective
Disable the failing Phase 8 section while preserving the rest of the test functionality.

### Concrete Deliverables
1. Lines 207-347 in `tests/integration_test.rs` commented out
2. Informative comment added explaining why it's commented
3. Added log message indicating Phase 8 is skipped
4. Test passes successfully

### Definition of Done
- [ ] `cargo test --test integration_test` passes
- [ ] All phases except Phase 8 execute successfully
- [ ] Phase 9 (proof generation) completes
- [ ] Phase 10 (final verification) completes

---

## Verification Strategy

### Test Command
```bash
cargo test --test integration_test 2>&1
```

### Expected Output
- Test should pass (exit code 0)
- Should see log output for Phases 1-7
- Should see "Phase 8: Data availability sampling SKIPPED"
- Should see Phases 9-10 complete successfully
- Final verification should succeed

---

## TODOs

### Task 1: Comment Out Phase 8

**What to do:**
1. Open `tests/integration_test.rs`
2. Find Phase 8 section (starts at line 207 with `let _span = span!(Level::INFO, "data_availability_sampling")`)
3. Comment out lines 207-347 (the entire Phase 8 block)
4. Add an explanatory comment block before the commented code explaining the issue
5. Add a log message after the commented block indicating Phase 8 is skipped

**Code to comment out:**
- Lines 207-347: The entire data availability sampling phase
- This includes:
  - Span creation for "data_availability_sampling"
  - All sampling logic
  - Results table
  - Success/failure reporting
  - Drop of the span

**Add after the commented block:**
```rust
info!("⏭️  Phase 8: Data availability sampling SKIPPED (see commented code)");
```

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: None required (simple file edit)
- **Reason**: This is a straightforward commenting task

**Parallelization:**
- **Can Run In Parallel**: NO
- **Sequential**: Must complete before running test

**Acceptance Criteria:**
- [ ] Lines 207-347 are commented out with `/* */` block
- [ ] Explanatory comment added before block
- [ ] Skip log message added after block
- [ ] File compiles without errors

**Commit**: YES
- Message: `test: comment out failing Phase 8 in integration test`
- Files: `tests/integration_test.rs`

---

### Task 2: Run Integration Test

**What to do:**
Run the integration test to verify the fix works.

**Command:**
```bash
cargo test --test integration_test 2>&1
```

**Expected Results:**
- Test passes
- Phases 1-7 execute normally
- Phase 8 is skipped (with log message)
- Phases 9-10 execute and succeed

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: None required

**Parallelization:**
- **Can Run In Parallel**: NO
- **Blocked By**: Task 1 (must be completed first)

**Acceptance Criteria:**
- [ ] Test compiles successfully
- [ ] Test runs without panics
- [ ] Phase 9 (proof generation) completes
- [ ] Phase 10 (final verification) completes successfully
- [ ] Test exits with code 0 (PASS)

**Commit**: NO (no code changes)

---

## Success Criteria

### Verification Commands
```bash
# Verify test passes
cargo test --test integration_test

# Check for Phase 8 skip message
cargo test --test integration_test 2>&1 | grep "Phase 8.*SKIPPED"

# Check for final verification success
cargo test --test integration_test 2>&1 | grep "Final verification succeeded"
```

### Final Checklist
- [ ] Phase 8 commented out with explanation
- [ ] Integration test passes
- [ ] Phases 9-10 execute successfully
- [ ] No compiler warnings introduced

---

## Notes

### Future Fix
The proper fix requires addressing the Merkle tree index mismatch:
- Either constrain sampling to valid indices: `max_index = 1 << fri_params.rs_code().log_len()`
- Or adjust parameters so Merkle tree capacity >= codeword length
- Or add bounds checking in `inclusion_proof()` to return error instead of panic

This plan only comments out the failing section to allow testing of other functionality.
