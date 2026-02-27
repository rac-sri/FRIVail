use binius_field::PackedExtension;
pub use binius_field::PackedField;
use binius_math::ntt::{domain_context::GenericPreExpanded, AdditiveNTT, NeighborsLastMultiThread};
use binius_prover::{
    fri::FRIQueryProver,
    hash::parallel_compression::ParallelCompressionAdaptor,
    merkle_tree::{prover::BinaryMerkleTreeProver, MerkleTreeProver},
};
use binius_transcript::VerifierTranscript;
pub use binius_verifier::config::B128;
use binius_verifier::merkle_tree::BinaryMerkleTreeScheme;
use binius_verifier::{
    config::{StdChallenger, B1},
    fri::FRIParams,
    hash::{StdCompression, StdDigest},
};
use std::mem::MaybeUninit;

// ============================================================================
// Type Aliases for Complex Types
// ============================================================================

/// Type alias for the Merkle tree prover
pub type MerkleProver<P> = BinaryMerkleTreeProver<
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

/// Type alias for FRI query prover
pub type FRIQueryProverAlias<'a, P> = FRIQueryProver<
    'a,
    <P as PackedField>::Scalar,
    P,
    MerkleProver<P>,
    BinaryMerkleTreeScheme<<P as PackedField>::Scalar, StdDigest, StdCompression>,
>;

pub trait FriVeilSampling<
    P: PackedField<Scalar = B128> + PackedExtension<B128> + PackedExtension<B1>,
    NTT: AdditiveNTT<Field = B128> + Sync,
>
{
    fn reconstruct_codeword_naive(
        &self,
        corrupted_codeword: &mut [P::Scalar],
        corrupted_indices: &[usize],
    ) -> Result<(), String>;
    fn verify(
        &self,
        verifier_transcript: &mut VerifierTranscript<StdChallenger>,
        evaluation_claim: P::Scalar,
        evaluation_point: &[P::Scalar],
        fri_params: &FRIParams<P::Scalar>,
        ntt: &NTT,
        extra_index: Option<usize>,
        terminate_codeword: Option<&[P::Scalar]>,
        layers: Option<&[Vec<digest::Output<StdDigest>>]>,
        extra_transcript: Option<&mut VerifierTranscript<StdChallenger>>,
    ) -> Result<(), String>;

    fn verify_inclusion_proof(
        &self,
        verifier_transcript: &mut VerifierTranscript<StdChallenger>,
        data: &[P::Scalar],
        index: usize,
        fri_params: &FRIParams<P::Scalar>,
        commitment: [u8; 32],
    ) -> Result<(), String>;

    fn inclusion_proof(
        &self,
        committed: &<MerkleProver<P> as MerkleTreeProver<<P as PackedField>::Scalar>>::Committed,
        index: usize,
    ) -> TranscriptResult;

    fn open<'b>(&self, index: usize, query_prover: &FRIQueryProverAlias<'b, P>)
        -> TranscriptResult;

    fn decode_codeword(
        &self,
        codeword: &[P::Scalar],
        fri_params: FRIParams<P::Scalar>,
        ntt: &NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
    ) -> FieldResult<P>;

    fn extract_commitment(
        &self,
        verifier_transcript: &mut VerifierTranscript<StdChallenger>,
    ) -> ByteResult;

    fn decode_batch(
        &self,
        log_dim: usize,
        log_inv: usize,
        log_batch_size: usize,
        ntt: &NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
        data: &[P::Scalar],
        output: &mut [MaybeUninit<P::Scalar>],
    ) -> Result<(), String>;
}

pub trait FriVeilUtils {
    fn get_transcript_bytes(&self, transcript: &VerifierTranscript<StdChallenger>) -> Vec<u8>;
    fn reconstruct_transcript_from_bytes(
        &self,
        bytes: Vec<u8>,
    ) -> VerifierTranscript<StdChallenger>;
}
