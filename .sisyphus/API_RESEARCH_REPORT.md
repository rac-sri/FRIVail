# Binius64 API Signatures Research Report

## Executive Summary
Research completed on exact API signatures for FRI-Veil implementation. All target APIs exist in the current binius64 codebase with documented signatures.

---

## 1. BINIUS DEPENDENCY VERSION

**Current Version**: `0.1.0` (local path dependencies)

**Cargo.toml Configuration**:
```toml
binius-prover = { path = "../binius/binius64/crates/prover" }
binius-verifier = { path = "../binius/binius64/crates/verifier" }
binius-math = { path = "../binius/binius64/crates/math" }
binius-field = { path = "../binius/binius64/crates/field" }
binius-transcript = { path = "../binius/binius64/crates/transcript" }
binius-spartan-prover = { path = "../binius/binius64/crates/spartan-prover" }
binius-spartan-verifier = { path = "../binius/binius64/crates/spartan-verifier" }
```

**Key Crates Used**:
- `binius-verifier`: FRI parameters and verification
- `binius-spartan-prover`: PCS prover implementation
- `binius-spartan-verifier`: PCS verification
- `binius-iop`: FRI common types and strategies

---

## 2. FRIParams::with_strategy() - FULL SIGNATURE

**Location**: `/Volumes/Personal/Avail/binius/binius64/crates/iop/src/fri/common.rs` (lines 77-122)

### Signature
```rust
pub fn with_strategy<NTT, MerkleScheme, Strategy>(
    ntt: &NTT,
    merkle_scheme: &MerkleScheme,
    log_msg_len: usize,
    log_batch_size: Option<usize>,
    log_inv_rate: usize,
    n_test_queries: usize,
    strategy: &Strategy,
) -> Result<Self, Error>
where
    NTT: AdditiveNTT<Field = F>,
    MerkleScheme: MerkleTreeScheme<F>,
    Strategy: AritySelectionStrategy,
```

### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `ntt` | `&NTT` | The additive NTT used for Reed-Solomon encoding |
| `merkle_scheme` | `&MerkleScheme` | The Merkle tree scheme used for commitments |
| `log_msg_len` | `usize` | Binary logarithm of message length to commit |
| `log_batch_size` | `Option<usize>` | If `Some(b)`, fixes batch size; if `None`, chosen optimally |
| `log_inv_rate` | `usize` | Binary logarithm of inverse Reed-Solomon code rate |
| `n_test_queries` | `usize` | Number of test queries for FRI protocol |
| `strategy` | `&Strategy` | Strategy for selecting fold arities (implements `AritySelectionStrategy`) |

### Return Type
```rust
Result<Self, Error>
```
- **Success**: Returns configured `FRIParams<F>` instance
- **Error**: Returns `Error` if fold arity sequence is invalid

### Preconditions
- If `log_batch_size` is `Some(b)`, then `b <= log_msg_len`
- `ntt.log_domain_size() >= log_msg_len - log_batch_size.unwrap_or(0) + log_inv_rate`

### Current Usage in Code
**File**: `src/friveil.rs` (lines 174-180)
```rust
let fri_params = FRIParams::new(
    committed_rs_code.clone(),
    fri_log_batch_size,
    fri_arities,
    self.num_test_queries,
)
.map_err(|e| e.to_string())?;
```

**Status**: Currently uses `FRIParams::new()` instead of `with_strategy()`. The `with_strategy()` method is available but not currently used.

---

## 3. PCSProver::prove() - FULL SIGNATURE

**Location**: `/Volumes/Personal/Avail/binius/binius64/crates/spartan-prover/src/pcs.rs` (lines 90-129)

### Signature
```rust
pub fn prove<P, Challenger_>(
    &self,
    committed_codeword: FieldBuffer<P>,
    committed: &'a MerkleProver::Committed,
    multilinear: FieldBuffer<P>,
    evaluation_point: &[F],
    evaluation_claim: F,
    transcript: &mut ProverTranscript<Challenger_>,
) -> Result<(), Error>
where
    P: PackedField<Scalar = F> + PackedExtension<F>,
    Challenger_: Challenger,
```

### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `&self` | `&PCSProver<'a, F, NTT, MTProver>` | The PCS prover instance |
| `committed_codeword` | `FieldBuffer<P>` | The committed codeword from FRI |
| `committed` | `&'a MerkleProver::Committed` | The committed merkle tree structure |
| `multilinear` | `FieldBuffer<P>` | The multilinear polynomial (must match what was committed) |
| `evaluation_point` | `&[F]` | The evaluation point in F^n_vars |
| `evaluation_claim` | `F` | The claimed evaluation of multilinear at evaluation_point |
| `transcript` | `&mut ProverTranscript<Challenger_>` | The prover's transcript (mutable) |

### Return Type
```rust
Result<(), Error>
```
- **Success**: Returns `Ok(())` if proof generation succeeds
- **Error**: Returns `Error` if proof generation fails

### Assertions
- Multilinear log_len must equal evaluation_point length
- Error message: "multilinear has {} variables but evaluation point has {} coordinates"

### Current Usage in Code
**File**: `src/friveil.rs` (lines 368-376)
```rust
pcs.prove(
    commit_output.codeword.clone(),
    &commit_output.committed,
    packed_mle,
    evaluation_point,
    evaluation_claim,
    &mut prover_transcript,
)
.map_err(|e| e.to_string())?;
```

**Status**: ✅ Currently used with correct signature. All parameters match expected types.

---

## 4. pcs::verify() - FULL FUNCTION SIGNATURE

**Location**: `/Volumes/Personal/Avail/binius/binius64/crates/verifier/src/pcs.rs` (lines 41-53)

### Signature
```rust
pub fn verify<F, MTScheme, Challenger_>(
    transcript: &mut VerifierTranscript<Challenger_>,
    evaluation_claim: F,
    eval_point: &[F],
    codeword_commitment: MTScheme::Digest,
    fri_params: &FRIParams<F>,
    merkle_scheme: &MTScheme,
) -> Result<(), Error>
where
    F: Field + BinaryField + PackedField<Scalar = F>,
    Challenger_: Challenger,
    MTScheme: MerkleTreeScheme<F, Digest: DeserializeBytes>,
```

### Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `transcript` | `&mut VerifierTranscript<Challenger_>` | The transcript of the prover's proof (mutable) |
| `evaluation_claim` | `F` | The evaluation claim of the prover |
| `eval_point` | `&[F]` | The evaluation point of the prover |
| `codeword_commitment` | `MTScheme::Digest` | VCS commitment to the codeword |
| `fri_params` | `&FRIParams<F>` | The FRI parameters |
| `merkle_scheme` | `&MTScheme` | The vector commitment scheme (Merkle tree) |

### Return Type
```rust
Result<(), Error>
```
- **Success**: Returns `Ok(())` if verification succeeds
- **Error**: Returns `Error` if verification fails

### Type Constraints
- `F`: Must be `Field + BinaryField + PackedField<Scalar = F>`
- `Challenger_`: Must implement `Challenger` trait
- `MTScheme`: Must implement `MerkleTreeScheme<F>` with `Digest: DeserializeBytes`

### Current Usage in Code
**File**: `src/friveil.rs` (lines 635-642)
```rust
spartan_verify(
    verifier_transcript,
    evaluation_claim,
    eval_point,
    retrieved_codeword_commitment,
    fri_params,
    &merkle_prover_scheme,
)
.map_err(|e| e.to_string())
```

**Status**: ✅ Currently used via import alias `spartan_verify`. All parameters match expected types.

---

## 5. API AVAILABILITY VERIFICATION

### FRIParams::with_strategy()
- **Status**: ✅ **AVAILABLE**
- **Location**: `binius-iop` crate
- **Availability**: Public method, fully documented
- **Preconditions**: Documented in code comments
- **Strategy Trait**: `AritySelectionStrategy` trait available in same file

### PCSProver::prove()
- **Status**: ✅ **AVAILABLE**
- **Location**: `binius-spartan-prover` crate
- **Availability**: Public method, fully documented
- **Assertions**: Validates multilinear dimensions match evaluation point
- **Error Handling**: Returns `Error` type from crate

### pcs::verify()
- **Status**: ✅ **AVAILABLE**
- **Location**: `binius-spartan-verifier` crate
- **Availability**: Public function, fully documented
- **Module**: `binius_spartan_verifier::pcs`
- **Current Import**: Already imported as `spartan_verify` in friveil.rs

---

## 6. DISCREPANCIES & NOTES

### Current Implementation vs. Target APIs

#### FRIParams Initialization
**Current Code**:
```rust
let fri_params = FRIParams::new(
    committed_rs_code.clone(),
    fri_log_batch_size,
    fri_arities,
    self.num_test_queries,
)?;
```

**Target API** (`with_strategy`):
- More flexible parameter selection
- Automatic arity selection via strategy pattern
- Supports optional batch size optimization
- Requires `AritySelectionStrategy` implementation

**Recommendation**: The current `FRIParams::new()` approach is valid and simpler. The `with_strategy()` method is available for more advanced use cases.

#### PCSProver::prove() Usage
**Status**: ✅ **FULLY COMPATIBLE**
- Current usage matches signature exactly
- All parameter types are correct
- Error handling is appropriate

#### Verification Function
**Status**: ✅ **FULLY COMPATIBLE**
- Currently imported as `spartan_verify`
- All parameters match expected types
- Error handling is appropriate

---

## 7. STRATEGY PATTERN AVAILABILITY

**AritySelectionStrategy Trait**:
- **Location**: `binius-iop` crate, `fri/common.rs`
- **Available Implementations**:
  - `ConstantArityStrategy` - Uses fixed arity for all folds
  - Other strategies may be available in the crate

**Example Usage** (from binius tests):
```rust
let fri_params = FRIParams::with_strategy(
    &ntt,
    merkle_prover.scheme(),
    multilinear.log_len(),
    None,
    LOG_INV_RATE,
    n_test_queries,
    &ConstantArityStrategy::new(2),
)?;
```

---

## 8. SUMMARY TABLE

| API | Status | Location | Current Usage | Signature Match |
|-----|--------|----------|----------------|-----------------|
| `FRIParams::with_strategy()` | ✅ Available | `binius-iop` | Not used (using `new()`) | N/A |
| `FRIParams::new()` | ✅ Available | `binius-iop` | ✅ Used | ✅ Match |
| `PCSProver::prove()` | ✅ Available | `binius-spartan-prover` | ✅ Used | ✅ Match |
| `pcs::verify()` | ✅ Available | `binius-spartan-verifier` | ✅ Used (as `spartan_verify`) | ✅ Match |

---

## 9. RECOMMENDATIONS FOR NEXT TASK

1. **FRIParams Initialization**: Current `new()` approach is valid. Consider `with_strategy()` only if automatic arity selection is needed.

2. **PCSProver::prove()**: Current implementation is correct. No changes needed.

3. **Verification**: Current `spartan_verify` usage is correct. No changes needed.

4. **All APIs are available** in the current binius64 version and can be used as documented.

---

**Research Completed**: 2026-02-23
**Status**: Ready for implementation phase
