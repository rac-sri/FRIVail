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

use crate::types::*;

pub trait FriVailSampling<
    P: PackedField<Scalar = B128> + PackedExtension<B128> + PackedExtension<B1>,
    NTT: AdditiveNTT<Field = B128> + Sync,
>
{
    /// Reconstruct a corrupted codeword using naive Lagrange interpolation
    ///
    /// # Arguments
    /// * `corrupted_codeword` - Mutable reference to the corrupted codeword to reconstruct
    /// * `corrupted_indices` - Indices of corrupted elements in the codeword
    ///
    /// # Returns
    /// Ok(()) if reconstruction succeeds
    ///
    /// # Errors
    /// When no known points are available for reconstruction
    fn reconstruct_codeword_naive(
        &self,
        corrupted_codeword: &mut [P::Scalar],
        corrupted_indices: &[usize],
    ) -> Result<(), String>;
    /// Verify an evaluation proof for the committed polynomial
    ///
    /// # Arguments
    /// * `verifier_transcript` - Verifier transcript containing the proof
    /// * `evaluation_claim` - Claimed evaluation result
    /// * `evaluation_point` - Point at which polynomial was evaluated
    /// * `fri_params` - FRI protocol parameters
    /// * `ntt` - Number Theoretic Transform instance
    /// * `extra_index` - Optional index for extra query verification
    /// * `terminate_codeword` - Optional terminal codeword for verification
    /// * `layers` - Optional Merkle tree layers for verification
    /// * `extra_transcript` - Optional extra transcript for query verification
    ///
    /// # Returns
    /// Ok(()) if verification succeeds
    ///
    /// # Errors
    /// When verification fails due to invalid proof or parameters
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

    /// Verify a Merkle inclusion proof for a codeword value
    ///
    /// # Arguments
    /// * `verifier_transcript` - Verifier transcript containing the inclusion proof
    /// * `data` - Data value to verify inclusion for
    /// * `index` - Index in the codeword
    /// * `fri_params` - FRI protocol parameters
    /// * `commitment` - Merkle tree root commitment
    ///
    /// # Returns
    /// Ok(()) if inclusion proof is valid
    ///
    /// # Errors
    /// When inclusion proof verification fails
    fn verify_inclusion_proof(
        &self,
        verifier_transcript: &mut VerifierTranscript<StdChallenger>,
        data: &[P::Scalar],
        index: usize,
        fri_params: &FRIParams<P::Scalar>,
        commitment: [u8; 32],
    ) -> Result<(), String>;

    /// Generate a Merkle inclusion proof for a specific codeword position
    ///
    /// # Arguments
    /// * `committed` - Committed Merkle tree
    /// * `index` - Index in the codeword to generate proof for
    ///
    /// # Returns
    /// Verifier transcript containing the inclusion proof
    ///
    /// # Errors
    /// When proof generation fails
    fn inclusion_proof(
        &self,
        committed: &<MerkleProver<P> as MerkleTreeProver<<P as PackedField>::Scalar>>::Committed,
        index: usize,
    ) -> TranscriptResult;

    /// Open a commitment at a specific index using FRI query prover
    ///
    /// # Arguments
    /// * `index` - Index in the codeword to open
    /// * `query_prover` - FRI query prover instance
    ///
    /// # Returns
    /// Verifier transcript containing the opening proof
    ///
    /// # Errors
    /// When opening fails
    fn open<'b>(&self, index: usize, query_prover: &FRIQueryProverAlias<'b, P>)
        -> TranscriptResult;

    /// Decode a Reed-Solomon encoded codeword back to original data
    ///
    /// # Arguments
    /// * `codeword` - Encoded codeword to decode
    /// * `fri_params` - FRI protocol parameters
    /// * `ntt` - Number Theoretic Transform instance
    ///
    /// # Returns
    /// Decoded packed field values
    ///
    /// # Errors
    /// When decoding fails
    fn decode_codeword(
        &self,
        codeword: &[P::Scalar],
        fri_params: FRIParams<P::Scalar>,
        ntt: &NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
    ) -> FieldResult<P>;

    /// Extract commitment from verifier transcript
    ///
    /// # Arguments
    /// * `verifier_transcript` - Verifier transcript to extract commitment from
    ///
    /// # Returns
    /// Commitment bytes
    ///
    /// # Errors
    /// When commitment extraction fails
    fn extract_commitment(
        &self,
        verifier_transcript: &mut VerifierTranscript<StdChallenger>,
    ) -> ByteResult;

    /// Low-level batch decoding using inverse NTT
    ///
    /// # Arguments
    /// * `log_dim` - Logarithm of dimension
    /// * `log_inv` - Logarithm of inverse rate
    /// * `log_batch_size` - Logarithm of batch size
    /// * `ntt` - Number Theoretic Transform instance
    /// * `data` - Input data to decode
    /// * `output` - Output buffer for decoded data
    ///
    /// # Returns
    /// Ok(()) if decoding succeeds
    ///
    /// # Errors
    /// When decoding fails due to invalid parameters
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

pub trait FriVailUtils {
    /// Get transcript bytes from verifier transcript
    ///
    /// # Arguments
    /// * `transcript` - Verifier transcript to extract bytes from
    ///
    /// # Returns
    /// Vector of transcript bytes
    fn get_transcript_bytes(&self, transcript: &VerifierTranscript<StdChallenger>) -> Vec<u8>;

    /// Reconstruct verifier transcript from bytes
    ///
    /// # Arguments
    /// * `bytes` - Bytes to reconstruct transcript from
    ///
    /// # Returns
    /// Reconstructed verifier transcript
    fn reconstruct_transcript_from_bytes(
        &self,
        bytes: Vec<u8>,
    ) -> VerifierTranscript<StdChallenger>;
}
