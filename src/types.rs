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

pub type FriVailDefault = crate::frivail::FriVail<
    'static,
    B128,
    BinaryMerkleTreeScheme<B128, StdDigest, StdCompression>,
    binius_math::ntt::NeighborsLastMultiThread<
        binius_math::ntt::domain_context::GenericPreExpanded<B128>,
    >,
>;

pub type MerkleProver<P> = BinaryMerkleTreeProver<
    <P as PackedField>::Scalar,
    StdDigest,
    ParallelCompressionAdaptor<StdCompression>,
>;

pub type FieldElements<P> = Vec<<P as PackedField>::Scalar>;

pub type FieldResult<P> = Result<FieldElements<P>, String>;

pub type TranscriptResult = Result<VerifierTranscript<StdChallenger>, String>;

pub type ByteResult = Result<Vec<u8>, String>;

pub type CommitmentOutput<P> =
    CommitOutput<
        P,
        digest::Output<StdDigest>,
        <MerkleProver<P> as binius_prover::merkle_tree::MerkleTreeProver<
            <P as PackedField>::Scalar,
        >>::Committed,
    >;

pub type FRIQueryProverAlias<'a, P> = FRIQueryProver<
    'a,
    <P as PackedField>::Scalar,
    P,
    MerkleProver<P>,
    BinaryMerkleTreeScheme<<P as PackedField>::Scalar, StdDigest, StdCompression>,
>;

pub type ProveResult<'a, P> = Result<
    (
        binius_math::FieldBuffer<<P as PackedField>::Scalar>,
        FRIQueryProverAlias<'a, P>,
        Vec<u8>,
    ),
    String,
>;

pub type TestFriVail = crate::frivail::FriVail<
    'static,
    B128,
    BinaryMerkleTreeScheme<B128, StdDigest, StdCompression>,
    binius_math::ntt::NeighborsLastMultiThread<
        binius_math::ntt::domain_context::GenericPreExpanded<B128>,
    >,
>;

pub use crate::frivail::FriVail;
pub use crate::traits::{FriVailSampling, FriVailUtils};
