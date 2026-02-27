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
        binius_math::ntt::domain_context::GenericPreExpanded<B128>,
    >,
>;

/// Merkle tree prover
pub type MerkleProver<P> = BinaryMerkleTreeProver<
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
pub type CommitmentOutput<P> =
    CommitOutput<
        P,
        digest::Output<StdDigest>,
        <MerkleProver<P> as binius_prover::merkle_tree::MerkleTreeProver<
            <P as PackedField>::Scalar,
        >>::Committed,
    >;

/// FRI query prover
pub type FRIQueryProverAlias<'a, P> = FRIQueryProver<
    'a,
    <P as PackedField>::Scalar,
    P,
    MerkleProver<P>,
    BinaryMerkleTreeScheme<<P as PackedField>::Scalar, StdDigest, StdCompression>,
>;

/// prove() return type
pub type ProveResult<'a, P> = Result<
    (
        binius_math::FieldBuffer<<P as PackedField>::Scalar>,
        FRIQueryProverAlias<'a, P>,
        Vec<u8>,
    ),
    String,
>;

/// Test configuration
pub type TestFriVail = crate::frivail::FriVail<
    'static,
    B128,
    BinaryMerkleTreeScheme<B128, StdDigest, StdCompression>,
    binius_math::ntt::NeighborsLastMultiThread<
        binius_math::ntt::domain_context::GenericPreExpanded<B128>,
    >,
>;

// Re-export for public use
pub use crate::frivail::FriVail;
pub use crate::traits::{FriVailSampling, FriVailUtils};
