# Comprehensive Refactoring Plan

## Tasks

### 1. Move All Types to Separate File
Create `src/types.rs` and move all type aliases from `src/frivail.rs`:

**Types to move:**
- `FriVailDefault` (after renaming)
- `MerkleProver<P>`
- `FieldElements<P>`
- `FieldResult<P>`
- `TranscriptResult`
- `ByteResult`
- `CommitmentOutput<P>`
- `FRIQueryProverAlias<'a, P>`
- `ProveResult<'a, P>`
- `TestFriVail` (after renaming)

**Update `src/lib.rs`:**
- Add `pub mod types;`
- Re-export types: `pub use types::*;`

**Update `src/frivail.rs`:**
- Remove type alias definitions
- Add `use crate::types::*;`

**Update `src/traits.rs`:**
- Add `use crate::types::*;`

### 2. Fix Naming (FriVeil → FriVail)

**src/traits.rs:**
- `FriVeilSampling` → `FriVailSampling`
- `FriVeilUtils` → `FriVailUtils`

**src/frivail.rs:**
- `FriVeil` → `FriVail` (struct)
- `FriVeilDefault` → `FriVailDefault`
- `TestFriVeil` → `TestFriVail`
- `friveil` → `friVail` (variables)
- `FriVeilSampling` → `FriVailSampling` (trait usage)
- `FriVeilUtils` → `FriVailUtils` (trait usage)

**README.md:**
- `friveil` → `frivail` (module import)
- `friVeil` → `friVail` (variable names)
- `FriVeilDefault` → `FriVailDefault`
- `FriVeilSampling` → `FriVailSampling`
- `FriVeilUtils` → `FriVailUtils`

### 3. Remove Excessive Comments

**src/frivail.rs:**
- Lines 1-20: Simplify module doc to 1-2 lines
- All verbose doc comments (lines 61-946): Simplify to 1 line each
- Remove obvious inline comments

**src/poly.rs:**
- Lines 14-20: Simplify `Utils` struct doc
- Lines 22-38: Simplify `PackedMLE` struct doc
- Lines 45-50: Simplify `new()` doc
- Lines 52-57: Simplify `bytes_to_scalar()` doc
- Lines 59-120: Simplify `bytes_to_packed_mle()` doc

## File Structure After Changes

```
src/
├── lib.rs          (exports modules and types)
├── types.rs        (NEW - all type aliases)
├── frivail.rs      (implementation, imports types)
├── traits.rs       (trait definitions, imports types)
└── poly.rs         (Utils implementation)
```

## Implementation Steps

1. Create `src/types.rs` with all type aliases
2. Update `src/lib.rs` to include types module
3. Rename all FriVeil → FriVail across all files
4. Update imports in `src/frivail.rs` and `src/traits.rs`
5. Remove excessive comments from `src/frivail.rs` and `src/poly.rs`
6. Update README.md
7. Run `cargo check` to verify

## New File: src/types.rs

```rust
//! Type aliases for FRI-Vail

pub use binius_field::PackedField;
use binius_prover::{
    fri::{CommitOutput, FRIQueryProver},
    hash::parallel_compression::ParallelCompressionAdaptor,
    merkle_tree::prover::BinaryMerkleTreeProver,
};
use binius_transcript::VerifierTranscript;
pub use binius_verifier::config::B128;
use binius_verifier::{
    config::StdChallenger,
    hash::{StdCompression, StdDigest},
    merkle_tree::BinaryMerkleTreeScheme,
};

/// Default FRI-Vail configuration
pub type FriVailDefault = crate::frivail::FriVail<
    'static,
    B128,
    BinaryMerkleTreeScheme<B128, StdDigest, StdCompression>,
    binius_math::ntt::NeighborsLastMultiThread<
        binius_math::ntt::domain_context::GenericPreExpanded<B128>
    >,
>;

/// Merkle tree prover
type MerkleProver<P> = BinaryMerkleTreeProver<
    <P as PackedField>::Scalar,
    StdDigest,
    ParallelCompressionAdaptor<StdCompression>,
>;

/// Field element vectors
pub type FieldElements<P> = Vec<<P as PackedField>::Scalar>;

/// Results with field elements
pub type FieldResult<P> = Result<FieldElements<P>, String>;

/// Transcript results
pub type TranscriptResult = Result<VerifierTranscript<StdChallenger>, String>;

/// Byte vector results
pub type ByteResult = Result<Vec<u8>, String>;

/// Commitment output
type CommitmentOutput<P> = CommitOutput<
    P,
    digest::Output<StdDigest>,
    <MerkleProver<P> as binius_prover::merkle_tree::MerkleTreeProver<<P as PackedField>::Scalar>>::Committed,
>;

/// FRI query prover
type FRIQueryProverAlias<'a, P> = FRIQueryProver<
    'a,
    <P as PackedField>::Scalar,
    P,
    MerkleProver<P>,
    BinaryMerkleTreeScheme<<P as PackedField>::Scalar, StdDigest, StdCompression>,
>;

/// prove() return type
type ProveResult<'a, P> = Result<
    (
        binius_math::FieldBuffer<<P as PackedField>::Scalar>,
        FRIQueryProverAlias<'a, P>,
        Vec<u8>,
    ),
    String,
>;

/// Test configuration
type TestFriVail = crate::frivail::FriVail<
    'static,
    B128,
    BinaryMerkleTreeScheme<B128, StdDigest, StdCompression>,
    binius_math::ntt::NeighborsLastMultiThread<
        binius_math::ntt::domain_context::GenericPreExpanded<B128>
    >,
>;

// Re-export for public use
pub use crate::frivail::FriVail;
pub use crate::traits::{FriVailSampling, FriVailUtils};
```

## Verification

After all changes:
```bash
cargo check
```

Expected: No errors, possibly warnings about unused code which can be cleaned up separately.
