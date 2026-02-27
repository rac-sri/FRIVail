# Type Alias Refactoring Plan

## Objective
Define type aliases for all complex return types in the frivail codebase to improve readability and maintainability.

## Complex Return Types Identified

### 1. Common Result Types (appear multiple times)
- `Result<Vec<P::Scalar>, String>` - Field vector results
- `Result<VerifierTranscript<StdChallenger>, String>` - Transcript results  
- `Result<Vec<u8>, String>` - Byte vector results

### 2. Complex Single-Use Types
- `CommitOutput<P, digest::Output<StdDigest>, <BinaryMerkleTreeProver<...> as MerkleTreeProver<P::Scalar>>::Committed>` - commit() return
- `(FieldBuffer<P::Scalar>, FRIQueryProver<'b, P::Scalar, P, BinaryMerkleTreeProver<...>, BinaryMerkleTreeScheme<...>>, Vec<u8>)` - prove() return tuple

## Type Aliases to Add

Add these in `src/frivail.rs` after line 72 (after `FriVeilDefault`):

```rust
// ============================================================================
// Type Aliases for Complex Types
// ============================================================================

/// Type alias for the Merkle tree prover
type MerkleProver<P> = BinaryMerkleTreeProver<
    <P as PackedField>::Scalar,
    StdDigest,
    ParallelCompressionAdaptor<StdCompression>,
>;

/// Type alias for field element vectors
pub type FieldElements<P> = Vec<<P as PackedField>::Scalar>;

/// Type alias for results with field elements
pub type FieldResult<P> = Result<FieldElements<P>, String>;

/// Type alias for transcript results
pub type TranscriptResult = Result<VerifierTranscript<StdChallenger>, String>;

/// Type alias for byte vector results
pub type ByteResult = Result<Vec<u8>, String>;

/// Type alias for commitment output
type CommitmentOutput<P> = CommitOutput<
    P,
    digest::Output<StdDigest>,
    <MerkleProver<P> as MerkleTreeProver<<P as PackedField>::Scalar>>::Committed,
>;

/// Type alias for FRI query prover
type FRIQueryProverAlias<'a, P> = FRIQueryProver<
    'a,
    <P as PackedField>::Scalar,
    P,
    MerkleProver<P>,
    BinaryMerkleTreeScheme<<P as PackedField>::Scalar, StdDigest, StdCompression>,
>;

/// Type alias for prove() return type
type ProveResult<'a, P> = Result<
    (
        FieldBuffer<<P as PackedField>::Scalar>,
        FRIQueryProverAlias<'a, P>,
        Vec<u8>,
    ),
    String,
>;
```

## Functions to Update in `src/frivail.rs`

| Function | Line | Current Return Type | New Return Type |
|----------|------|---------------------|-----------------|
| `calculate_evaluation_point_random` | ~205 | `Result<Vec<P::Scalar>, String>` | `FieldResult<P>` |
| `commit` | ~283 | Complex `CommitOutput<...>` | `Result<CommitmentOutput<P>, String>` |
| `prove` | ~350 | Complex tuple | `ProveResult<'b, P>` |
| `inclusion_proof` | ~666 | `Result<VerifierTranscript<StdChallenger>, String>` | `TranscriptResult` |
| `open` | ~695 | `Result<VerifierTranscript<StdChallenger>, String>` | `TranscriptResult` |
| `decode_codeword` | ~793 | `Result<Vec<P::Scalar>, String>` | `FieldResult<P>` |
| `extract_commitment` | ~849 | `Result<Vec<u8>, String>` | `ByteResult` |

## Trait Updates in `src/traits.rs`

| Method | Line | Current Return Type | New Return Type |
|--------|------|---------------------|-----------------|
| `inclusion_proof` | ~58 | `Result<VerifierTranscript<StdChallenger>, String>` | `TranscriptResult` |
| `open` | ~74 | `Result<VerifierTranscript<StdChallenger>, String>` | `TranscriptResult` |
| `decode_codeword` | ~81 | `Result<Vec<P::Scalar>, String>` | `FieldResult<P>` |
| `extract_commitment` | ~86 | `Result<Vec<u8>, String>` | `ByteResult` |

Also update the `committed` parameter in `inclusion_proof` to use the `MerkleProver` alias.

## Parameter Type Updates

In `prove()` function (line ~340):
- Change `commit_output` parameter from complex `CommitOutput<...>` to `&'b CommitmentOutput<P>`

## Verification Steps

1. Run `cargo check` - should compile without errors
2. Run `cargo test` - all tests should pass
3. Run `cargo check --features parallel` - should compile

## Example Transformation

**Before:**
```rust
pub fn calculate_evaluation_point_random(&self) -> Result<Vec<P::Scalar>, String> {
    // ...
}
```

**After:**
```rust
pub fn calculate_evaluation_point_random(&self) -> FieldResult<P> {
    // ...
}
```

**Before:**
```rust
pub fn commit(...) -> Result<
    CommitOutput<
        P,
        digest::Output<StdDigest>,
        <BinaryMerkleTreeProver<...> as MerkleTreeProver<...>>::Committed,
    >,
    String,
> {
```

**After:**
```rust
pub fn commit(...) -> Result<CommitmentOutput<P>, String> {
```

## Benefits

- **Readability**: Complex types reduced from 80+ characters to <20 characters
- **Maintainability**: Changes to underlying types only need updates in one place
- **Consistency**: All Result types follow clear naming patterns
- **Developer Experience**: Easier to understand function signatures at a glance

## Risk Mitigation

- Keep aliases `pub` where needed for public API
- Don't change any logic, only type definitions
- Run full test suite after changes
- Use `cargo check` frequently during implementation
