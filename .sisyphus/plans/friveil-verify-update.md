# FRI-Veil Verify Function Update with Extra Query Verification

## TL;DR

> **Quick Summary**: Update the `verify` function in `src/friveil.rs` to support extra j-point query verification using the PCS verification pattern with `vcs_optimal_layers_depths_iter` from `binius_iop::fri`. This is a **breaking change** to the `FriVeilSampling` trait.
> 
> **Deliverables**:
> - Updated `verify()` function with extra query verification capability
> - `binius_iop::fri::vcs_optimal_layers_depths_iter` import and usage
> - Updated trait signature in `src/traits.rs`
> - Updated test to exercise extra verification
> 
> **Estimated Effort**: Medium (2-3 hours)
> **Parallel Execution**: NO - sequential due to trait dependency chain
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 4

---

## Context

### Original Request
Update the verify function to use pcs::verify, as in the provided example code, and additionally do an extra j-point query verification. Create a helper function to get layers from the prover transcript.

### Interview Summary
**Key Discussions**:
- User wants to modify existing verify function (not create a separate one)
- User wants a standalone helper method for layer extraction
- User confirmed only `binius_iop::fri::vcs_optimal_layers_depths_iter` import was needed initially
- User accepts breaking changes to the trait signature
- User wants immediate failure if extra verification fails
- User wants existing test updated (not new comprehensive test)

**Research Findings**:
- `verifier_with_arena` pattern already implemented in `friveil.rs` (lines 773-785)
- `vcs_optimal_layers()` is available on `FRIQueryProver`
- **`vcs_optimal_layers_depths_iter` EXISTS** in `binius_iop::fri` at `../binius/binius64/crates/iop/src/fri/common.rs:161-175`
- Function signature: `pub fn vcs_optimal_layers_depths_iter<'a, F, VCS>(fri_params: &'a FRIParams<F>, vcs: &'a VCS) -> impl Iterator<Item = usize>`
- Used in verification at `../binius/binius64/crates/iop/src/fri/verify.rs` for layer verification

### Metis Review
**Identified Gaps** (addressed):
- Breaking change to trait signature - User accepted
- `vcs_optimal_layers_depths_iter` location - Found in binius_iop crate
- Error handling strategy - Fail immediately
- Testing scope - Update existing test only
- **Helper function not needed** - User clarified layers come from prover, no extraction helper needed

---

## Work Objectives

### Core Objective
Update the `FriVeil::verify()` function to support optional extra j-point query verification by accepting additional parameters (extra_index, terminate_codeword, layers). Use the `vcs_optimal_layers_depths_iter` function from `binius_iop::fri` for layer depth iteration in the verification logic.

### Concrete Deliverables
1. Updated `verify()` method in `FriVeilSampling` trait (`src/traits.rs:27-33`)
2. Updated `verify()` implementation in `FriVeil` struct (`src/friveil.rs:755-788`)
3. Updated `test_full_prove_verify_workflow` test to exercise extra verification

### Definition of Done
- [ ] `cargo check --lib` passes with no errors
- [ ] `cargo test --lib test_full_prove_verify_workflow` passes
- [ ] `cargo build --all-targets` succeeds
- [ ] Verify function signature includes optional extra parameters
- [ ] `binius_iop::fri::vcs_optimal_layers_depths_iter` import added
- [ ] Extra j-point verification executes when parameters provided
- [ ] Uses `vcs_optimal_layers_depths_iter` for layer depth iteration

### Must Have
- Updated trait signature with optional extra verification parameters
- Implementation using existing `spartan_verify` and `verifier_with_arena` pattern
- `binius_iop::fri::vcs_optimal_layers_depths_iter` import and usage
- Error handling that fails immediately on extra verification failure

### Must NOT Have (Guardrails)
- MUST NOT modify `prove()` or `commit()` functions
- MUST NOT add unnecessary abstractions beyond requested scope
- MUST NOT change error handling patterns beyond "fail immediately"
- MUST NOT add new comprehensive tests (only update existing)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES - Built-in Rust test framework (cargo test)
- **User wants tests**: Tests-after - Update existing test
- **Framework**: Built-in `cargo test`

### Automated Verification (No User Intervention Required)

**By Deliverable Type:**

| Type | Verification Tool | Automated Procedure |
|------|------------------|---------------------|
| **Library/Module** | Bash cargo test | Agent runs: `cargo test --lib test_full_prove_verify_workflow` → Assert: test passes |
| **Compilation** | Bash cargo check | Agent runs: `cargo check --lib` → Assert: no compilation errors |
| **Build** | Bash cargo build | Agent runs: `cargo build --all-targets` → Assert: successful build |

**Evidence Requirements:**
- Terminal output from `cargo test` showing test result
- Terminal output from `cargo check` showing no errors
- Terminal output from `cargo build` showing success

---

## Execution Strategy

### Sequential Execution (No Parallelization)

Due to trait dependency chain, tasks must execute sequentially:

```
Wave 1 (Start):
└── Task 1: Update trait signature

Wave 2 (After Task 1):
└── Task 2: Update verify implementation

Wave 3 (After Task 2):
└── Task 3: Update test

Wave 4 (After Task 3):
└── Task 4: Verify all targets build

Critical Path: Task 1 → Task 2 → Task 3 → Task 4
Parallel Speedup: 0% - all tasks depend on previous
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 (Trait) | None | 2 | None |
| 2 (Impl) | 1 | 3 | None |
| 3 (Test) | 2 | 4 | None |
| 4 (Verify) | 3 | None | None |

---

## TODOs

### Task 1: Update FriVeilSampling Trait Signature

**What to do**:
- Modify `src/traits.rs` lines 27-33
- Add optional parameters for extra query verification
- Change signature from:
  ```rust
  fn verify(
      &self,
      verifier_transcript: &mut VerifierTranscript<StdChallenger>,
      evaluation_claim: P::Scalar,
      evaluation_point: &[P::Scalar],
      fri_params: &FRIParams<P::Scalar>,
  ) -> Result<(), String>;
  ```
  To:
  ```rust
  fn verify(
      &self,
      verifier_transcript: &mut VerifierTranscript<StdChallenger>,
      evaluation_claim: P::Scalar,
      evaluation_point: &[P::Scalar],
      fri_params: &FRIParams<P::Scalar>,
      extra_index: Option<usize>,
      terminate_codeword: Option<&[P::Scalar]>,
      layers: Option<&[Vec<digest::Output<StdDigest>>>],
  ) -> Result<(), String>;
  ```

**Must NOT do**:
- Don't change return type
- Don't add non-optional parameters
- Don't modify other trait methods

**Recommended Agent Profile**:
- **Category**: `quick`
  - Reason: Simple signature change, low complexity
- **Skills**: None required

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Sequential
- **Blocks**: Task 2
- **Blocked By**: None (can start immediately)

**References**:
- `src/traits.rs:27-33` - Current trait signature
- `src/friveil.rs:755-788` - Current implementation to understand parameter usage

**Acceptance Criteria**:
- [ ] Trait signature updated with 3 new optional parameters
- [ ] `cargo check --lib` shows no errors for trait definition

**Commit**: YES
- Message: `refactor(traits): add optional extra verification params to FriVeilSampling::verify`
- Files: `src/traits.rs`
- Pre-commit: `cargo check --lib`

---

### Task 2: Update verify Implementation

**What to do**:
- Modify `src/friveil.rs` lines 755-788
- Update function signature to match trait
- Add extra query verification logic when optional params are Some
- Add import: `use binius_iop::fri::vcs_optimal_layers_depths_iter;`
- Implementation pattern based on user's example code:
  ```rust
  // Keep existing verification
  let verifier_with_arena = spartan_verify(
      verifier_transcript,
      evaluation_claim,
      eval_point,
      retrieved_codeword_commitment,
      fri_params,
      &merkle_prover_scheme,
  )?;
  
  // Get verifier from arena
  let verifier = verifier_with_arena.verifier();
  
  // If extra parameters provided, perform extra query verification
  if let (Some(idx), Some(codeword), Some(layers)) = 
      (extra_index, terminate_codeword, layers) {
      // Use vcs_optimal_layers_depths_iter for layer depth iteration
      for (commitment, layer_depth, layer) in izip!(
          iter::once(verifier.codeword_commitment).chain(verifier.round_commitments),
          vcs_optimal_layers_depths_iter(verifier.params, verifier.vcs),
          layers
      ) {
          verifier.vcs.verify_layer(commitment, layer_depth, layer)?;
      }
      
      // Verify extra query
      verifier.verify_query(idx, &ntt, codeword, layers, &mut advice)?;
  }
  ```
- Fail immediately with error if extra verification fails

**Must NOT do**:
- Don't remove existing main verification
- Don't change error handling beyond "fail immediately"

**Recommended Agent Profile**:
- **Category**: `unspecified-high`
  - Reason: Complex FRI verification logic, needs careful implementation
- **Skills**: None required

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Sequential
- **Blocks**: Task 3
- **Blocked By**: Task 1 (trait signature must match)

**References**:
- `src/friveil.rs:755-788` - Current verify implementation
- `src/friveil.rs:774-785` - Existing verifier_with_arena pattern
- `../binius/binius64/crates/iop/src/fri/common.rs:161-175` - vcs_optimal_layers_depths_iter definition
- `../binius/binius64/crates/iop/src/fri/verify.rs:117-121` - vcs_optimal_layers_depths_iter usage in verification
- User's example code showing:
  - `verifier_with_arena.verifier()` usage
  - `verifier.verify_last_oracle()` pattern
  - `verifier.verify_layer()` pattern with layer depths iterator
  - `verifier.verify_query()` pattern

**Acceptance Criteria**:
- [ ] Import added: `use binius_iop::fri::vcs_optimal_layers_depths_iter;`
- [ ] Function signature matches updated trait
- [ ] Existing main verification still works
- [ ] Uses `vcs_optimal_layers_depths_iter` for layer depth iteration in extra verification
- [ ] Extra query verification executes when all 3 optional params are Some
- [ ] Returns Err immediately if extra verification fails
- [ ] `cargo check --lib` shows no errors

**Commit**: YES
- Message: `feat(friveil): add extra query verification to verify()`
- Files: `src/friveil.rs`
- Pre-commit: `cargo check --lib`

---

### Task 3: Update test_full_prove_verify_workflow

**What to do**:
- Modify test at `src/friveil.rs:1327-1392`
- After calling `prove()`, extract layers from query_prover directly
- Update `verify()` call to include extra parameters
- Use index 0 as extra_index for testing
- Test should verify with extra query and still pass

Current test flow:
```rust
let (_terminate_codeword, _query_prover, transcript_bytes) = prove_result.unwrap();
// ... create verifier_transcript ...
let verify_result = friveil.verify(
    &mut verifier_transcript,
    evaluation_claim,
    &evaluation_point,
    &fri_params,
);
```

Updated test flow:
```rust
let (terminate_codeword, query_prover, transcript_bytes) = prove_result.unwrap();
// Extract layers directly from query_prover
let layers = query_prover.vcs_optimal_layers().expect("Failed to get layers");
// ... create verifier_transcript ...
let terminate_codeword_vec: Vec<_> = terminate_codeword.iter_scalars().collect();
let verify_result = friveil.verify(
    &mut verifier_transcript,
    evaluation_claim,
    &evaluation_point,
    &fri_params,
    Some(0), // extra_index
    Some(&terminate_codeword_vec), // terminate_codeword
    Some(&layers), // layers
);
```

**Must NOT do**:
- Don't create new test function (user wants existing test updated)
- Don't remove existing verification without extra params
- Don't change test assertions beyond what's needed

**Recommended Agent Profile**:
- **Category**: `quick`
  - Reason: Test update, low complexity
- **Skills**: None required

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Sequential
- **Blocks**: Task 4
- **Blocked By**: Task 2

**References**:
- `src/friveil.rs:1327-1392` - Current test to update
- `src/friveil.rs:1361-1370` - Current prove() call
- `src/friveil.rs:1381-1391` - Current verify() call

**Acceptance Criteria**:
- [ ] Test extracts layers directly from query_prover
- [ ] Test extracts terminate_codeword as vector of scalars
- [ ] Test calls `verify()` with all 3 optional parameters (Some(...))
- [ ] Test passes with `cargo test --lib test_full_prove_verify_workflow`

**Commit**: YES
- Message: `test(friveil): update test_full_prove_verify_workflow with extra query`
- Files: `src/friveil.rs` (test section)
- Pre-commit: `cargo test --lib test_full_prove_verify_workflow`

---

### Task 4: Verify Build and All Tests Pass

**What to do**:
- Run full build to ensure all targets compile
- Run all tests to verify no regressions

**Recommended Agent Profile**:
- **Category**: `quick`
  - Reason: Build verification only
- **Skills**: None required

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Sequential
- **Blocks**: None (final task)
- **Blocked By**: Task 3

**Acceptance Criteria**:
- [ ] `cargo build --all-targets` succeeds
- [ ] `cargo test --lib` shows all tests pass
- [ ] `cargo check --lib` shows no warnings

**Commit**: NO (verification task only)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `refactor(traits): add optional extra verification params to FriVeilSampling::verify` | src/traits.rs | `cargo check --lib` |
| 2 | `feat(friveil): add extra query verification to verify()` | src/friveil.rs (verify impl) | `cargo check --lib` |
| 3 | `test(friveil): update test_full_prove_verify_workflow with extra query` | src/friveil.rs (test) | `cargo test --lib test_full_prove_verify_workflow` |

---

## Success Criteria

### Verification Commands
```bash
# 1. Library compiles without errors
cargo check --lib
# Expected: No errors or warnings

# 2. All tests pass
cargo test --lib
# Expected: All tests pass including test_full_prove_verify_workflow

# 3. All targets build
cargo build --all-targets
# Expected: Successful build

# 4. Specific test passes with extra verification
cargo test --lib test_full_prove_verify_workflow
# Expected: test test_full_prove_verify_workflow ... ok
```

### Final Checklist
- [ ] FriVeilSampling trait signature updated with optional params
- [ ] FriVeil::verify implementation updated with extra verification
- [ ] FriVeil::get_layers helper method added
- [ ] test_full_prove_verify_workflow updated and passes
- [ ] All existing tests still pass
- [ ] No compilation errors or warnings

---

## Guardrails Summary

### AI-Slop Patterns Avoided
| Pattern | How We Avoided |
|---------|----------------|
| Scope creep | Locked to verify() update with vcs_optimal_layers_depths_iter, and test update only |
| Premature abstraction | No new abstractions beyond what's needed |
| Over-validation | Only "fail immediately" error handling as specified |
| Documentation bloat | Minimal docs, focused on implementation |

### Breaking Change Acknowledgment
**⚠️ WARNING**: This plan includes a **breaking change** to the `FriVeilSampling` trait. All implementors will need to update their code. The user explicitly accepted this risk.

### Known Limitations
1. `vcs_optimal_layers_depths_iter` exists in binius_iop crate - ensure import is correct
2. No new comprehensive tests - only updating existing test as requested
3. Breaking change to trait - will require updates to all implementors

---

## Notes for Executor (Sisyphus)

1. **Sequential Execution Required**: Tasks must be completed in order (1→2→3→4) due to trait/impl dependencies
2. **Breaking Change**: This will break compilation if there are other implementors of FriVeilSampling - user accepted this
3. **vcs_optimal_layers_depths_iter**: Exists in `binius_iop::fri` - add import and use in verification logic
4. **Error Handling**: Must fail immediately on extra verification failure
5. **Test Update**: Only update test_full_prove_verify_workflow, don't create new tests
6. **Type Complexity**: FRIQueryProver has complex generic types - copy from existing usage in prove() method
