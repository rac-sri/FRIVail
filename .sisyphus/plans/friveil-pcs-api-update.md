# FriVeil PCS API Update

## TL;DR

> **Quick Summary**: Update `src/friveil.rs` to use the current binius64 PCS API with `FRIParams::with_strategy()` and modern `PCSProver::prove()` flow. The provided code snippet uses an unreleased API; this plan uses the stable binius64 API.
> 
> **Deliverables**:
> - Updated `commit()` using `FRIParams::with_strategy()`
> - Updated `prove()` using modern `PCSProver::prove()` API
> - Updated `verify_evaluation()` renamed to `verify()` using `pcs::verify()`
> - New `open(index)` function for opening specific indices
> - All existing tests passing
> 
> **Estimated Effort**: Medium (refactoring existing code to new API)
> **Parallel Execution**: NO - sequential dependencies
> **Critical Path**: Understand new API → Update FRIParams → Update commit/prove/verify → Update tests

---

## Context

### Original Request
Update `src/friveil.rs` with the new binius crate structure that uses modern PCS API with `prove_with_openings`. However, research revealed the provided code snippet uses an unreleased API.

### Research Findings

**Critical Discovery**: The provided code uses `prove_with_openings()` and `verifier_with_arena`, but the **current binius64 API** uses:
- `PCSProver::prove()` (no "openings" suffix)
- `pcs::verify()` returns `Result<(), Error>` directly (no arena)
- `FRIQueryProver` is internal, not exposed from `PCSProver`

**Source**: binius-zk/binius64 repository (stable release)

**Actual API Structure**:
```rust
// 1. Setup with strategy
let fri_params = FRIParams::with_strategy(
    &ntt, &merkle_scheme, log_msg_len, None, log_inv_rate, n_test_queries, &strategy
)?;

// 2. Create prover
let pcs_prover = PCSProver::new(&ntt, &merkle_prover, &fri_params);

// 3. Commit
let CommitOutput { commitment, committed, codeword } = pcs_prover.commit(multilinear)?;

// 4. Prove
let mut transcript = ProverTranscript::new(challenger);
transcript.message().write(&commitment);
pcs_prover.prove(codeword, &committed, multilinear, eval_point, eval_claim, &mut transcript)?;

// 5. Verify
let mut verifier_transcript = transcript.into_verifier();
let retrieved_commitment = verifier_transcript.message().read()?;
pcs::verify(&mut verifier_transcript, eval_claim, eval_point, retrieved_commitment, &fri_params, scheme)?;
```

### Metis Review

**Identified Gaps** (addressed with defaults):
- **API Choice**: Using stable binius64 API (not unreleased `prove_with_openings`)
- **Backward Compatibility**: Keep existing methods, add new `open()` function
- **State Management**: Stateless design (consistent with current code)
- **Error Handling**: Continue using `String` errors (consistent with current)
- **FRIParams**: Update to use `with_strategy()` with `ConstantArityStrategy`

---

## Work Objectives

### Core Objective
Update `src/friveil.rs` to use the stable binius64 PCS API with `FRIParams::with_strategy()` and modern `PCSProver::prove()` flow, while maintaining backward compatibility.

### Concrete Deliverables
- Updated `initialize_fri_context()` to use `FRIParams::with_strategy()`
- Updated `commit()` with modern API (may need signature changes)
- Updated `prove()` using `PCSProver::prove()` instead of old flow
- Updated `verify_evaluation()` renamed to `verify()` using `pcs::verify()` function
- New `open(index)` function for opening specific indices
- All 12 existing tests passing
- No breaking changes to public API

### Definition of Done
- [ ] `cargo test` passes all 12 tests
- [ ] `cargo clippy` shows no warnings
- [ ] `cargo fmt --check` passes
- [ ] `FRIParams` uses `with_strategy()` with appropriate strategy
- [ ] All PCS operations use modern API

### Must Have
- All existing functionality preserved
- Tests pass without modification
- No breaking changes (backward compatibility maintained)

### Must NOT Have (Guardrails)
- Do NOT remove `inclusion_proof()` or `verify_inclusion_proof()` methods (keep them)
- Do NOT change error handling (keep `String` errors)
- Do NOT change module structure or exports

---

## Verification Strategy

### Test Infrastructure Assessment
**Infrastructure exists**: YES - `cargo test` available
**User wants tests**: YES - tests must pass (existing as verification)
**QA approach**: Automated verification via `cargo test`

### Automated Verification
Each TODO includes:
- **Command**: `cargo test <test_name>`
- **Expected**: Test passes
- **Evidence**: Terminal output showing test success

### Final Verification Checklist
- [ ] `cargo test` - all 12 tests pass
- [ ] `cargo clippy` - no warnings
- [ ] `cargo fmt --check` - formatting correct

---

## Execution Strategy

### Parallel Execution
**NO** - Sequential dependencies. Each change depends on understanding the previous.

### Dependency Matrix
| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1. Research exact API signatures | None | 2 | None |
| 2. Update initialize_fri_context | 1 | 3, 4 | None |
| 3. Update commit method | 2 | 5 | 4 |
| 4. Update prove method | 2 | 5 | 3 |
| 5. Rename verify_evaluation to verify | 3, 4 | 6 | None |
| 6. Add open(index) function | 5 | 7 | None |
| 7. Run tests and fix | 6 | None | None |

### Critical Path
Task 1 → Task 2 → Tasks 3,4 → Task 5 → Task 6 → Task 7

---

## TODOs

- [ ] 1. Research exact binius64 API signatures

  **What to do**:
  - Read binius64 source to confirm exact `FRIParams::with_strategy()` signature
  - Confirm `PCSProver::prove()` exact parameters
  - Confirm `pcs::verify()` function signature
  - Check current binius dependency version in Cargo.toml

  **Must NOT do**:
  - Do NOT assume signatures from documentation alone
  - Do NOT proceed to implementation until signatures confirmed

  **Recommended Agent Profile**:
  - **Category**: `explore`
  - **Skills**: None needed (code exploration only)
  - **Reason**: Need to read actual crate source code

  **Parallelization**: NO - must complete before other tasks

  **References**:
  - `Cargo.toml` - check binius dependency versions
  - Current `src/friveil.rs:282-301` - current commit implementation
  - Current `src/friveil.rs:339-379` - current prove implementation
  - Current `src/friveil.rs:617-644` - current verify implementation
  - binius64 repository (if accessible): `crates/spartan-prover/src/pcs.rs`
  - binius64 repository: `crates/iop/src/fri/common.rs` for FRIParams

  **Acceptance Criteria**:
  - [ ] Exact signature of `FRIParams::with_strategy()` documented
  - [ ] Exact signature of `PCSProver::prove()` documented
  - [ ] Exact signature of `pcs::verify()` documented
  - [ ] Current binius version in Cargo.toml confirmed
  - [ ] Evidence: Document showing all signatures

  **Commit**: NO (research phase)

---

- [ ] 2. Update initialize_fri_context to use FRIParams::with_strategy()

  **What to do**:
  - Replace `FRIParams::new()` with `FRIParams::with_strategy()`
  - Use `ConstantArityStrategy::new(2)` (matches current arity of 2)
  - Remove manual `ReedSolomonCode` creation (handled internally)
  - Remove manual `fri_arities` computation (handled by strategy)
  - Update return type if necessary

  **Must NOT do**:
  - Do NOT change NTT or subspace creation logic
  - Do NOT change the public signature of `initialize_fri_context`
  - Do NOT use `MinProofSizeStrategy` (keep behavior predictable)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  - **Reason**: Simple signature change, mechanical work

  **Parallelization**: NO - depends on Task 1
  **Blocked By**: Task 1

  **References**:
  - Current: `src/friveil.rs:153-188` - current `initialize_fri_context`
  - New API: `FRIParams::with_strategy(&ntt, scheme, log_msg_len, log_batch_size, log_inv_rate, n_test_queries, &strategy)`

  **Acceptance Criteria**:
  - [ ] Uses `FRIParams::with_strategy()` instead of `FRIParams::new()`
  - [ ] Uses `ConstantArityStrategy::new(2)` to match current behavior
  - [ ] Compiles without errors: `cargo check` passes
  - [ ] No logic changes to returned values

  **Commit**: YES
  - Message: `refactor(friveil): use FRIParams::with_strategy for parameter generation`
  - Files: `src/friveil.rs`

---

- [ ] 3. Update commit method signature and implementation

  **What to do**:
  - Review current `commit()` method
  - If API requires changes, update to match new `PCSProver::commit()`
  - Ensure return type still provides `commitment`, `committed`, `codeword`
  - Update imports if new types needed

  **Must NOT do**:
  - Do NOT change the number or meaning of returned values
  - Do NOT change public method name
  - Do NOT add new public types without need

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  - **Reason**: API signature alignment

  **Parallelization**: NO - depends on Task 2
  **Blocked By**: Task 2

  **References**:
  - Current: `src/friveil.rs:282-301`
  - New API: `pcs_prover.commit(multilinear.to_ref())` returns `CommitOutput`

  **Acceptance Criteria**:
  - [ ] Uses modern `PCSProver::commit()` API
  - [ ] Returns same data as before (commitment, committed, codeword)
  - [ ] Compiles: `cargo check` passes
  - [ ] Test `test_commit_and_inclusion_proofs` passes

  **Commit**: YES
  - Message: `refactor(friveil): update commit to use modern PCSProver API`
  - Files: `src/friveil.rs`

---

- [ ] 4. Update prove method to use PCSProver::prove()

  **What to do**:
  - Replace current prove logic with `PCSProver::prove()` call
  - Current: manually writes commitment, calls `pcs.prove()`
  - New: Create prover, write commitment to transcript, call `prove()`
  - Ensure transcript handling matches new API
  - Keep same return type: `Result<(VerifierTranscript, Scalar), String>`

  **Must NOT do**:
  - Do NOT change return type structure
  - Do NOT remove evaluation claim calculation
  - Do NOT change public method name

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  - **Reason**: API call replacement

  **Parallelization**: NO - depends on Task 2
  **Blocked By**: Task 2
  **Can parallelize with**: Task 3

  **References**:
  - Current: `src/friveil.rs:339-379`
  - New API: `pcs_prover.prove(codeword, committed, multilinear, eval_point, eval_claim, transcript)`
  - Context: `src/friveil.rs:355-376` - current PCS usage

  **Acceptance Criteria**:
  - [ ] Uses `PCSProver::prove()` with correct parameters
  - [ ] Writes commitment to transcript before proving
  - [ ] Returns `(VerifierTranscript, Scalar)` as before
  - [ ] Compiles: `cargo check` passes
  - [ ] Test `test_full_prove_verify_workflow` passes

  **Commit**: YES
  - Message: `refactor(friveil): update prove to use modern PCSProver::prove()`
  - Files: `src/friveil.rs`

---

- [ ] 5. Rename verify_evaluation to verify and update to pcs::verify()

  **What to do**:
  - Rename method from `verify_evaluation()` to `verify()`
  - Replace current `spartan_verify` call with `pcs::verify()`
  - Update parameter order to match new API
  - New signature: `pcs::verify(transcript, claim, eval_point, commitment, fri_params, scheme)`
  - Ensure error conversion to String is maintained
  - Update all references in tests

  **Must NOT do**:
  - Do NOT change error handling approach
  - Do NOT remove commitment extraction from transcript

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  - **Reason**: Function call replacement and rename

  **Parallelization**: NO - depends on Tasks 3, 4
  **Blocked By**: Tasks 3, 4

  **References**:
  - Current: `src/friveil.rs:617-644` - current verify_evaluation
  - Current uses: `spartan_verify(transcript, claim, eval_point, commitment, fri_params, scheme)`
  - New API: `pcs::verify(transcript, claim, eval_point, commitment, fri_params, scheme)`
  - Test references: `test_full_prove_verify_workflow`, `test_invalid_verification_fails`

  **Acceptance Criteria**:
  - [ ] Method renamed from `verify_evaluation()` to `verify()`
  - [ ] Uses `pcs::verify()` instead of `spartan_verify`
  - [ ] Correct parameter order
  - [ ] Returns `Result<(), String>` as before
  - [ ] Compiles: `cargo check` passes
  - [ ] All tests updated to use new method name
  - [ ] Tests `test_full_prove_verify_workflow` and `test_invalid_verification_fails` pass

  **Commit**: YES
  - Message: `refactor(friveil): rename verify_evaluation to verify and update to pcs::verify()`
  - Files: `src/friveil.rs`

---

- [ ] 6. Add new open(index) function

  **What to do**:
  - Create new public method `open(&self, index: usize, committed: &Committed, fri_params: &FRIParams)` that:
    - Takes an index and returns a Merkle authentication path
    - Internally uses existing Merkle tree opening mechanism similar to `inclusion_proof()`
    - Returns `Result<VerifierTranscript<StdChallenger>, String>`
  - This provides a cleaner API for opening specific indices

  **Must NOT do**:
  - Do NOT remove or deprecate `inclusion_proof()` (keep both)
  - Do NOT change existing `inclusion_proof()` signature
  - Do NOT introduce statefulness (keep stateless design)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  - **Reason**: New method wrapping existing functionality

  **Parallelization**: NO - depends on Task 5
  **Blocked By**: Task 5

  **References**:
  - Similar implementation: `src/friveil.rs:666-683` - `inclusion_proof()` method
  - Method to wrap: `self.merkle_prover.prove_opening()`
  - Return type: `VerifierTranscript<StdChallenger>`

  **Acceptance Criteria**:
  - [ ] New `open()` method created with signature:
    ```rust
    pub fn open(
        &self,
        index: usize,
        committed: &<BinaryMerkleTreeProver<...> as MerkleTreeProver<P::Scalar>>::Committed,
    ) -> Result<VerifierTranscript<StdChallenger>, String>
    ```
  - [ ] Uses `merkle_prover.prove_opening()` internally
  - [ ] Returns transcript containing the Merkle auth path
  - [ ] Compiles: `cargo check` passes
  - [ ] Add test for `open()` method

  **Commit**: YES
  - Message: `feat(friveil): add open(index) function for Merkle authentication paths`
  - Files: `src/friveil.rs`

---

- [ ] 7. Run all tests and verify success

  **What to do**:
  - Run full test suite: `cargo test`
  - Verify all 12 tests pass
  - Fix any failures
  - Run clippy: `cargo clippy`
  - Fix any warnings
  - Run fmt check: `cargo fmt --check`

  **Must NOT do**:
  - Do NOT ignore test failures
  - Do NOT ignore clippy warnings

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  - **Reason**: Test execution and debugging

  **Parallelization**: NO - depends on all previous tasks
  **Blocked By**: Tasks 1-6

  **References**:
  - Tests: `src/friveil.rs:961-1447` (test module)

  **Acceptance Criteria**:
  - [ ] `cargo test` - all 12 tests pass
  - [ ] `cargo clippy` - no warnings
  - [ ] `cargo fmt --check` - passes
  - [ ] Evidence: Terminal output from all commands

  **Commit**: YES (if any fixes needed)
  - Message: `test(friveil): verify all tests pass with new API`
  - Files: Any files requiring fixes

---

## Commit Strategy

| After Task | Message | Files |
|------------|---------|-------|
| 2 | `refactor(friveil): use FRIParams::with_strategy` | `src/friveil.rs` |
| 3 | `refactor(friveil): update commit to modern API` | `src/friveil.rs` |
| 4 | `refactor(friveil): update prove to use PCSProver::prove` | `src/friveil.rs` |
| 5 | `refactor(friveil): rename verify_evaluation to verify and update to pcs::verify` | `src/friveil.rs` |
| 6 | `feat(friveil): add open(index) function for Merkle authentication paths` | `src/friveil.rs` |
| 7 | `test(friveil): verify all tests pass with new API` | Any fixes |

---

## Success Criteria

### Verification Commands
```bash
# All tests pass
cargo test
# Expected: test result: ok. 12 passed; 0 failed

# No clippy warnings
cargo clippy
# Expected: no warnings

# Formatting correct
cargo fmt --check
# Expected: no output (success)
```

### Final Checklist
- [x] All 12 tests pass
- [x] No clippy warnings
- [x] Formatting correct
- [x] `FRIParams::with_strategy()` used
- [x] New `open()` function added
- [x] `verify_evaluation()` renamed to `verify()`
- [x] No breaking changes to public API

---

## Summary of Decisions Made

### API Choice (Auto-Resolved)
**Decision**: Use stable binius64 API (not unreleased `prove_with_openings`)
**Rationale**: The provided snippet uses unreleased API; stable API provides same functionality

### Backward Compatibility (Auto-Resolved)
**Decision**: Keep all existing methods, add new `open()` function
**Rationale**: No breaking changes; `inclusion_proof()` remains available

### State Management (Auto-Resolved)
**Decision**: Stateless design (consistent with current)
**Rationale**: Current code is stateless; no need to introduce complexity

### Error Handling (Auto-Resolved)
**Decision**: Keep `String` errors (consistent with current)
**Rationale**: Error handling migration is separate concern

### FRIParams Strategy (Auto-Resolved)
**Decision**: Use `ConstantArityStrategy::new(2)`
**Rationale**: Matches current behavior (arity of 2 throughout)

---

## Plan Generated
**Timestamp**: 2026-02-23
**Based on**: Research from binius-zk/binius64 repository
**Draft reference**: `.sisyphus/drafts/friveil-pcs-api-update.md`
