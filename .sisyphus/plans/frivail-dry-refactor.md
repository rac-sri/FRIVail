# Work Plan: DRY Refactoring for frivail

## TL;DR

> **Quick Summary**: Apply DRY principles to the frivail FRI-based Vector Commitment library through 5 incremental refactoring waves, eliminating ~40 lines of duplicated interpolation logic, removing unused field, fixing benchmark compilation errors, and improving type signature maintainability.
> 
> **Deliverables**:
> - Fixed benchmark compilation (11 arity parameter fixes)
> - Removed 1 unused field (log_scalar_bit_width)
> - Extracted shared Lagrange interpolation helper (~40 lines → ~15 lines)
> - Simplified 8+ complex type signatures with internal aliases
> - Unified parallel/sequential byte-to-scalar conversion
> - Consistent naming throughout codebase
> 
> **Estimated Effort**: Medium (5 incremental waves, ~2-3 hours total)
> **Parallel Execution**: NO - sequential waves with dependencies
> **Critical Path**: Wave 1 → Wave 2 → Wave 3 → Wave 4 → Wave 5

---

## Context

### Original Request
User wants to refactor the frivail codebase to apply DRY (Don't Repeat Yourself) principles and achieve cleaner code.

### Interview Summary
**Key Decisions**:
- **Approach**: Incremental (6 waves), each independently reviewable
- **Scope**: ALL items in scope - benchmarks, dead code removal, DRY extractions, type aliases, naming
- **API Compatibility**: Full backward compatibility required
- **Risk Tolerance**: Medium - tests must pass after each wave
- **Unused Code**: Remove completely (not just mark)

**Research Findings**:
- Codebase: Rust FRI-based Vector Commitment library (frivail)
- Main modules: `frivail.rs` (1500+ lines), `poly.rs` (128 lines), `traits.rs` (105 lines)
- All 10 tests pass (9 unit + 1 integration), 14-second runtime
- Duplicated logic found in:
  1. Lagrange interpolation (parallel vs sequential versions) - ~40 lines duplicated
  2. Byte-to-scalar conversion (parallel vs sequential) - ~25 lines duplicated
  3. Complex type signatures repeated 8+ times
- Unused code: `log_scalar_bit_width` field (encode_codeword and extract_commitment kept as requested)
- Bug: 11 benchmark calls missing `arity` parameter

### Metis Review
**Identified Gaps** (addressed):
- Feature flag testing needed for parallel builds
- Benchmark verification step after Wave 1
- Rollback strategy: git revert per wave if tests fail
- Acceptance criteria must cover both `cargo test` and `cargo test --features parallel`

---

## Work Objectives

### Core Objective
Eliminate code duplication and improve maintainability through 6 incremental refactoring waves while preserving all existing behavior and maintaining backward API compatibility.

### Concrete Deliverables
1. **Wave 1**: `benches/commitment.rs` - 11 `arity` parameter additions
2. **Wave 2**: `src/poly.rs` - Remove unused `log_scalar_bit_width` field from `Utils<P>`
3. **Wave 3**: `src/frivail.rs` - Extract `interpolate_at_point()` helper
4. **Wave 4**: `src/frivail.rs` - Add `MerkleProver<P>` type alias
5. **Wave 5**: `src/poly.rs` - Extract `bytes_to_scalar()` helper
6. **Wave 6**: Documentation - Naming consistency fixes

### Definition of Done
- [ ] All existing tests pass (`cargo test`)
- [ ] Parallel feature tests pass (`cargo test --features parallel`)
- [ ] Benchmarks compile successfully (`cargo bench --no-run`)
- [ ] No compiler warnings
- [ ] Public API unchanged (backward compatible)

### Must Have
- All 5 waves completed
- Tests passing after each wave
- Feature flag builds working
- Benchmark compilation fixed

### Must NOT Have (Guardrails)
- No public API signature changes
- No removal of used code
- No changes to cryptographic logic (only code organization)
- No breaking changes to dependencies
- No single large PR (must be incremental)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES - `cargo test` available
- **User wants tests**: Manual verification after each wave (existing tests sufficient)
- **Framework**: Built-in Rust test harness

### Automated Verification (Agent-Executable)

**For Each Wave:**

**Unit Test Verification**:
```bash
# Agent runs:
cargo test
# Expected: 9 tests passed, 0 failed

cargo test --features parallel
# Expected: 9 tests passed, 0 failed
```

**Benchmark Compilation Verification** (Wave 1 and after):
```bash
# Agent runs:
cargo bench --no-run
# Expected: Compiles without errors
```

**Warning Check**:
```bash
# Agent runs:
cargo check 2>&1 | grep -i warning
# Expected: No warnings (or only expected ones from dependencies)
```

**Evidence to Capture:**
- [ ] Terminal output from `cargo test` showing pass count
- [ ] Terminal output from `cargo test --features parallel`
- [ ] Terminal output from `cargo bench --no-run`

---

## Execution Strategy

### Sequential Waves (NO Parallel Execution)

```
Wave 1: Fix Benchmark Errors
    ↓ (tests must pass)
Wave 2: Remove Unused Field
    ↓ (tests must pass)
Wave 3: Extract Interpolation Logic
    ↓ (tests must pass)
Wave 4: Type Aliases
    ↓ (tests must pass)
Wave 5: Unify Byte Conversion
    ↓ (tests must pass)
Wave 6: Naming Consistency
```

**Dependency Matrix**:
| Wave | Depends On | Blocks |
|------|------------|--------|
| 1 | None | 2 |
| 2 | 1 | 3 |
| 3 | 2 | 4 |
| 4 | 3 | 5 |
| 5 | 4 | 6 |
| 6 | 5 | None |

**Critical Path**: 1 → 2 → 3 → 4 → 5 → 6

### Agent Dispatch Summary

| Wave | Tasks | Recommended Approach |
|------|-------|---------------------|
| 1-6 | Each is a single focused change | Sequential execution, one wave at a time |

---

## TODOs

### Wave 1: Fix Benchmark Compilation Errors

**What to do**:
- Add missing `arity` parameter (value: 4) to 11 `FriVeilDefault::new()` calls in `benches/commitment.rs`
- Affected lines: 129, 153, 176, 199, 223, 242, 260, 279, 297, 316, 334
- Insert `4,` as 5th argument (after `log_num_shares`)

**Must NOT do**:
- Change any other benchmark logic
- Modify test code
- Change source library code

**Recommended Agent Profile**:
- **Category**: `quick` - Simple parameter addition, no complex logic
- **Skills**: None needed - straightforward edit

**Parallelization**:
- **Can Run In Parallel**: NO - Wave 1 starts the sequence
- **Blocks**: Wave 2
- **Blocked By**: None

**References**:
- Working example at line 106: `FriVeilDefault::new(1, 100, 4, packed_mle_values.total_n_vars, 80, 3)`
- API definition: `src/frivail.rs:122-141` - `new(log_inv_rate, num_test_queries, arity, n_vars, log_num_shares)`

**Acceptance Criteria**:
- [ ] `cargo bench --no-run` compiles without errors
- [ ] `cargo test` passes (9 tests)
- [ ] `cargo test --features parallel` passes (9 tests)
- [ ] No compiler warnings in benchmark code

**Commit**: YES
- Message: `fix(bench): add missing arity parameter to all FriVeilDefault::new() calls`
- Files: `benches/commitment.rs`

---

### Wave 2: Remove Unused Field

**What to do**:
1. **src/poly.rs**:
   - Remove `log_scalar_bit_width: usize` field from `Utils<P>` struct (line 19)
   - Remove `log_scalar_bit_width: <P::Scalar as ExtensionField<B1>>::LOG_DEGREE,` initialization in `new()` (line 52)
   - Note: Keep `_p: PhantomData<P>` as it's needed for the generic type parameter

**Must NOT do**:
- Remove `encode_codeword()` function (kept as requested)
- Remove `extract_commitment()` function (kept as requested)
- Remove any code that has active callers
- Change public function signatures

**Verification of unused status**:
- Run `cargo build` before removal - should show warning about `log_scalar_bit_width` being unused
- Check with `grep -r "log_scalar_bit_width" src/ benches/ tests/ --include="*.rs"`

**Recommended Agent Profile**:
- **Category**: `quick` - Deletion of clearly unused field
- **Skills**: None needed

**Parallelization**:
- **Can Run In Parallel**: NO - Sequential wave
- **Blocks**: Wave 3
- **Blocked By**: Wave 1

**References**:
- `src/poly.rs:17-23` - Utils struct definition
- `src/poly.rs:47-55` - new() method

**Acceptance Criteria**:
- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo test` passes (9 tests)
- [ ] `cargo test --features parallel` passes (9 tests)
- [ ] `cargo bench --no-run` still compiles
- [ ] No warnings about `log_scalar_bit_width` (should be removed)
- [ ] `encode_codeword()` and `extract_commitment()` still exist
- [ ] `cargo clippy` (if available) shows no warnings

**Commit**: YES
- Message: `refactor: remove unused log_scalar_bit_width field from Utils struct`
- Files: `src/poly.rs`

---

### Wave 3: Extract Shared Interpolation Logic (Major DRY)

**What to do**:
- Extract Lagrange interpolation computation from `reconstruct_codeword_naive()` into private helper
- Current: ~40 lines duplicated in parallel and sequential branches
- Target: ~15 lines shared helper + 5 lines per branch for iteration

**Implementation Plan**:
1. Add private helper method to `FriVeil`:
```rust
/// Compute Lagrange interpolation at a specific point
/// 
/// # Arguments
/// * `x_e` - Point at which to evaluate
/// * `known` - Known (x, y) pairs for interpolation
/// * `k` - Number of known points
/// 
/// # Returns
/// * Interpolated value at x_e
fn interpolate_at_point(
    &self,
    x_e: P::Scalar,
    known: &[(P::Scalar, P::Scalar)],
    k: usize,
) -> P::Scalar {
    let mut value = P::Scalar::zero();
    for j in 0..k {
        let (x_j, y_j) = known[j];
        let mut l_j = P::Scalar::ONE;
        for m in 0..k {
            if m == j {
                continue;
            }
            let (x_m, _) = known[m];
            l_j = l_j * (x_e - x_m) * (x_j - x_m).invert().unwrap();
        }
        value = value + y_j * l_j;
    }
    value
}
```

2. Refactor `reconstruct_codeword_naive()`:
   - Keep parallel and sequential iteration paths
   - Replace inner interpolation loops with call to `interpolate_at_point()`

**Must NOT do**:
- Change the interpolation algorithm (keep same mathematical operations)
- Change parallel vs sequential execution behavior
- Modify the result values

**Recommended Agent Profile**:
- **Category**: `unspecified-medium` - Requires careful extraction without changing behavior
- **Skills**: None needed - internal refactoring only

**Parallelization**:
- **Can Run In Parallel**: NO
- **Blocks**: Wave 4
- **Blocked By**: Wave 2

**References**:
- `src/frivail.rs:449-549` - Current reconstruct_codeword_naive implementation
- Parallel branch: lines 474-513
- Sequential branch: lines 515-546
- Shared logic: lines 483-498 and 524-539 (nearly identical)

**Acceptance Criteria**:
- [ ] `cargo test` passes - especially `test_error_correction_reconstruction`
- [ ] `cargo test --features parallel` passes
- [ ] Code compiles with `--features parallel` flag
- [ ] New helper is private (not in public API)
- [ ] No behavioral changes (same interpolation results)

**Commit**: YES
- Message: `refactor: extract shared Lagrange interpolation into interpolate_at_point() helper`
- Files: `src/frivail.rs`
- Pre-commit: `cargo test test_error_correction_reconstruction`

---

### Wave 4: Simplify Type Signatures with Aliases

**What to do**:
- Create internal type alias for frequently repeated complex type
- Replace 8+ occurrences throughout `frivail.rs`

**Implementation Plan**:
1. Add type alias in `FriVeil` impl block or module level:
```rust
/// Type alias for the Merkle tree prover to simplify signatures
type MerkleProver<P> = BinaryMerkleTreeProver<
    <P as PackedField>::Scalar,
    StdDigest,
    ParallelCompressionAdaptor<StdCompression>,
>;
```

2. Replace occurrences:
   - Search for pattern: `BinaryMerkleTreeProver<P::Scalar, StdDigest, ParallelCompressionAdaptor<StdCompression>>`
   - Replace with: `MerkleProver<P>`

**Occurrences to update** (search results):
- Line 95: `merkle_prover` field type
- Line 284-291: `CommitOutput` return type in `commit()`
- Line 340-347: `CommitOutput` parameter in `prove()`
- Line 351-361: `FRIQueryProver` return type
- Line 660-664: `inclusion_proof()` committed parameter
- Line 683-693: `open()` query_prover parameter
- Line 748: `verify_inclusion_proof()` (implicit in self)

**Must NOT do**:
- Change any public type signatures
- Modify runtime behavior
- Affect trait implementations

**Recommended Agent Profile**:
- **Category**: `quick` - Type-only change
- **Skills**: None needed

**Parallelization**:
- **Can Run In Parallel**: NO
- **Blocks**: Wave 5
- **Blocked By**: Wave 3

**References**:
- Type occurrences throughout `src/frivail.rs`
- Full type: `BinaryMerkleTreeProver<P::Scalar, StdDigest, ParallelCompressionAdaptor<StdCompression>>`

**Acceptance Criteria**:
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (9 tests)
- [ ] `cargo test --features parallel` passes
- [ ] No compiler errors
- [ ] Type alias is private (not in public API)

**Commit**: YES
- Message: `refactor: add MerkleProver<P> type alias for repeated complex type`
- Files: `src/frivail.rs`

---

### Wave 5: Unify Byte-to-Scalar Conversion (DRY)

**What to do**:
- Extract shared scalar conversion logic from parallel and sequential branches in `bytes_to_packed_mle()`
- Similar pattern to Wave 3: iteration stays conditional, conversion logic becomes shared

**Implementation Plan**:
1. Add private helper function:
```rust
/// Convert a byte chunk to a field element
fn bytes_to_scalar(&self, chunk: &[u8]) -> P::Scalar {
    let mut bytes_array = [0u8; BYTES_PER_ELEMENT];
    bytes_array[..chunk.len()].copy_from_slice(chunk);
    P::Scalar::from(u128::from_le_bytes(bytes_array))
}
```

2. Refactor `bytes_to_packed_mle()`:
   - Keep `#[cfg(feature = "parallel")]` and `#[cfg(not(feature = "parallel"))]` blocks
   - Replace conversion logic with calls to `bytes_to_scalar()`

**Current duplication** (lines 91-114):
- Both branches have identical logic:
  ```rust
  let mut bytes_array = [0u8; 16];  // or BYTES_PER_ELEMENT
  bytes_array[..chunk.len()].copy_from_slice(chunk);
  P::Scalar::from(u128::from_le_bytes(bytes_array))
  ```

**Must NOT do**:
- Change parallel vs sequential execution strategy
- Modify the conversion algorithm
- Change field element values produced

**Recommended Agent Profile**:
- **Category**: `quick` - Similar to Wave 3 but smaller scope
- **Skills**: None needed

**Parallelization**:
- **Can Run In Parallel**: NO
- **Blocks**: Wave 6
- **Blocked By**: Wave 4

**References**:
- `src/poly.rs:90-114` - Current bytes_to_packed_mle implementation
- `src/poly.rs:91-101` - Parallel branch
- `src/poly.rs:104-114` - Sequential branch

**Acceptance Criteria**:
- [ ] `cargo test` passes
- [ ] `cargo test --features parallel` passes
- [ ] `cargo build` succeeds with and without parallel feature
- [ ] Helper is private
- [ ] Conversion produces identical results

**Commit**: YES
- Message: `refactor: extract shared bytes_to_scalar() helper for DRY conversion`
- Files: `src/poly.rs`

---

### Wave 6: Naming Consistency (Polish)

**What to do**:
- Ensure consistent naming: `FriVeil` (not `friVail` or `friVeil`)
- Update any inconsistent references in comments and docstrings
- Check throughout codebase:
  - Module: `frivail` (lowercase file, correct)
  - Struct: `FriVeil` (correct)
  - Traits: `FriVeilSampling`, `FriVeilUtils` (correct)
  - README references: Check for `friVail` vs `FriVeil`

**Files to check**:
- `src/lib.rs` - module declarations
- `src/frivail.rs` - doc comments
- `README.md` - documentation references
- `src/traits.rs` - trait documentation

**Current inconsistencies found**:
- README.md line 37: `friVail` (should be `FriVeil` or `FriVeilDefault`)
- Various doc comments may have inconsistent casing

**Must NOT do**:
- Change actual API names (file names, struct names)
- Only fix comments and documentation

**Recommended Agent Profile**:
- **Category**: `unspecified-low` - Documentation-only changes
- **Skills**: None needed

**Parallelization**:
- **Can Run In Parallel**: NO
- **Blocks**: None (final wave)
- **Blocked By**: Wave 5

**References**:
- `README.md` - search for "friVail"
- `src/frivail.rs` - doc comments
- `src/lib.rs` - module docs

**Acceptance Criteria**:
- [ ] No inconsistent naming in comments/docs
- [ ] `cargo test` still passes
- [ ] Documentation builds without errors (`cargo doc`)

**Commit**: YES
- Message: `docs: fix naming consistency - FriVeil instead of friVail`
- Files: `README.md`, `src/lib.rs`, `src/frivail.rs` (doc comments only)

---

## Commit Strategy

| After Wave | Message | Files |
|------------|---------|-------|
| 1 | `fix(bench): add missing arity parameter to all FriVeilDefault::new() calls` | `benches/commitment.rs` |
| 2 | `refactor: remove unused log_scalar_bit_width field from Utils struct` | `src/poly.rs` |
| 3 | `refactor: extract shared Lagrange interpolation into interpolate_at_point() helper` | `src/frivail.rs` |
| 4 | `refactor: add MerkleProver<P> type alias for repeated complex type` | `src/frivail.rs` |
| 5 | `refactor: extract shared bytes_to_scalar() helper for DRY conversion` | `src/poly.rs` |
| 6 | `docs: fix naming consistency - FriVeil instead of friVail` | `README.md`, `src/lib.rs`, `src/frivail.rs` |

---

## Success Criteria

### Verification Commands
```bash
# After each wave:
cargo test                          # Expected: 9 tests passed
cargo test --features parallel      # Expected: 9 tests passed  
cargo bench --no-run                # Expected: Compiles OK (Wave 1+)
cargo check                         # Expected: No warnings
cargo doc                           # Expected: No errors (Wave 6)
```

### Final Checklist
- [ ] Wave 1: 11 arity parameters added, benchmarks compile
- [ ] Wave 2: 1 unused field removed (log_scalar_bit_width), encode_codeword and extract_commitment kept
- [ ] Wave 3: ~40 lines of duplicated interpolation reduced to ~15
- [ ] Wave 4: 8+ complex type signatures simplified with alias
- [ ] Wave 5: Parallel/sequential conversion logic unified
- [ ] Wave 6: All naming consistent in documentation
- [ ] All 10 tests pass (`cargo test` + `cargo test --features parallel`)
- [ ] No compiler warnings
- [ ] Public API unchanged (backward compatible)

### Rollback Strategy
If any wave fails:
1. Do NOT proceed to next wave
2. Fix issues in current wave
3. Re-run tests
4. Only proceed when all tests pass

Each wave is independent - can `git revert` single wave if needed.

---

## Notes for Implementer

### Key Safety Rules:
1. **Never modify both parallel and sequential logic simultaneously** - change one, test, then the other
2. **Run feature flag tests** - both with and without `parallel` feature
3. **Check for warnings after Wave 2** - ensure log_scalar_bit_width removal eliminates the warning
4. **Verify benchmark compilation after Wave 1** - use `cargo bench --no-run`

### Common Pitfalls to Avoid:
- **PhantomData**: In Wave 2, keep `PhantomData<P>` as it's needed for the generic parameter
- **Type alias scope**: In Wave 4, ensure alias is accessible where needed (may need module-level vs impl-level)
- **Feature flags**: In Waves 3 and 5, always test both `cargo test` and `cargo test --features parallel`

### Files Modified Summary:
- `benches/commitment.rs` - Wave 1 only
- `src/poly.rs` - Waves 2, 5
- `src/frivail.rs` - Waves 3, 4, 6 (doc comments)
- `README.md` - Wave 6 only
- `src/lib.rs` - Wave 6 only (if needed)
- `src/traits.rs` - Wave 6 only (if needed)
