//! FRI-Vail: FRI-based Vector Commitment Scheme with Data Availability Sampling

use crate::traits::{FriVailSampling, FriVailUtils};
use crate::types::*;
use binius_field::field::FieldOps;
pub use binius_field::PackedField;
use binius_field::{Field, PackedExtension, Random};
use binius_iop::fri::vcs_optimal_layers_depths_iter;
use binius_math::{
    bit_reverse::bit_reverse_packed,
    inner_product::{inner_product, inner_product_buffers},
    multilinear::eq::eq_ind_partial_eval,
    ntt::{
        domain_context::{self, GenericPreExpanded},
        AdditiveNTT, NeighborsLastMultiThread,
    },
    BinarySubspace, FieldBuffer, FieldSlice, FieldSliceMut,
};
use binius_prover::{
    fri::{CommitOutput, FRIQueryProver},
    hash::parallel_compression::ParallelCompressionAdaptor,
    merkle_tree::{prover::BinaryMerkleTreeProver, MerkleTreeProver},
};
use binius_spartan_prover::pcs::PCSProver;
use binius_spartan_verifier::pcs::verify as spartan_verify;
use binius_transcript::{Buf, ProverTranscript, VerifierTranscript};
pub use binius_verifier::config::B128;
use binius_verifier::{
    config::{StdChallenger, B1},
    fri::{ConstantArityStrategy, FRIParams},
    hash::{StdCompression, StdDigest},
    merkle_tree::{BinaryMerkleTreeScheme, MerkleTreeScheme},
};

use itertools::{izip, Itertools};
use rand::{rngs::StdRng, SeedableRng};
use std::{marker::PhantomData, mem::MaybeUninit};
use tracing::debug;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// FRI-Vail polynomial commitment scheme
pub struct FriVail<'a, P, VCS, NTT>
where
    NTT: AdditiveNTT<Field = B128> + Sync,
    P: PackedField<Scalar = B128> + PackedExtension<B128> + PackedExtension<B1>,
    VCS: MerkleTreeScheme<P::Scalar>,
{
    _ntt: PhantomData<&'a NTT>,
    pub merkle_prover:
        BinaryMerkleTreeProver<P::Scalar, StdDigest, ParallelCompressionAdaptor<StdCompression>>,
    log_inv_rate: usize,
    num_test_queries: usize,
    arity: usize,
    n_vars: usize,
    log_num_shares: usize,
    _vcs: PhantomData<VCS>,
}

impl<'a, P, VCS, NTT> FriVail<'a, P, VCS, NTT>
where
    P: PackedField<Scalar = B128> + PackedExtension<B128> + PackedExtension<B1>,
    VCS: MerkleTreeScheme<P::Scalar>,
    NTT: AdditiveNTT<Field = B128> + Sync,
{
    /// Create a new FRI-Vail instance
    ///
    /// # Arguments
    /// * `log_inv_rate` - Logarithm of inverse rate for Reed-Solomon encoding
    /// * `num_test_queries` - Number of test queries for FRI protocol (security parameter)
    /// * `arity` - Arity for FRI folding strategy
    /// * `n_vars` - Number of variables for multilinear extension
    /// * `log_num_shares` - Logarithm of number of shares for Merkle tree
    ///
    /// # Returns
    /// New FriVail instance
    pub fn new(
        log_inv_rate: usize,
        num_test_queries: usize,
        arity: usize,
        n_vars: usize,
        log_num_shares: usize,
    ) -> Self {
        Self {
            merkle_prover: BinaryMerkleTreeProver::<P::Scalar, StdDigest, _>::new(
                ParallelCompressionAdaptor::new(StdCompression::default()),
            ),
            log_inv_rate,
            num_test_queries,
            arity,
            n_vars,
            log_num_shares,
            _ntt: PhantomData,
            _vcs: PhantomData,
        }
    }

    /// Initialize FRI protocol context and NTT for Reed-Solomon encoding
    ///
    /// # Arguments
    /// * `packed_buffer_log_len` - Logarithm of packed buffer length
    ///
    /// # Returns
    /// Tuple containing FRI parameters and NTT instance
    ///
    /// # Errors
    /// When FRI parameter initialization fails
    pub fn initialize_fri_context(
        &self,
        packed_buffer_log_len: usize,
    ) -> Result<
        (
            FRIParams<P::Scalar>,
            NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
        ),
        String,
    > {
        // Create subspace and NTT first (needed for with_strategy)
        let code_log_len = packed_buffer_log_len + self.log_inv_rate;
        let subspace = BinarySubspace::with_dim(code_log_len);

        let domain_context = domain_context::GenericPreExpanded::generate_from_subspace(&subspace);
        let ntt = NeighborsLastMultiThread::new(domain_context, self.log_num_shares);

        // Use with_strategy to create FRI parameters
        let fri_params = FRIParams::with_strategy(
            &ntt,
            self.merkle_prover.scheme(),
            packed_buffer_log_len,
            Some(0), // hardcoded to 0, DAS doesn't need the data to be clubbed
            // into cosets
            self.log_inv_rate,
            self.num_test_queries,
            &ConstantArityStrategy::new(self.arity),
        )
        .map_err(|e| e.to_string())?;

        Ok((fri_params, ntt))
    }

    /// Generate a random evaluation point for polynomial evaluation
    ///
    /// # Returns
    /// Vector of random field elements representing the evaluation point
    ///
    /// # Errors
    /// When random number generation fails
    pub fn calculate_evaluation_point_random(&self) -> FieldResult<P> {
        let mut rng = StdRng::from_seed([0; 32]);
        let evaluation_point: Vec<P::Scalar> = (0..self.n_vars)
            .map(|_| <B128 as Random>::random(&mut rng))
            .collect();
        Ok(evaluation_point)
    }

    /// Calculate the evaluation claim for a polynomial at a given point
    ///
    /// # Arguments
    /// * `values` - Polynomial values to evaluate
    /// * `evaluation_point` - Point at which to evaluate the polynomial
    ///
    /// # Returns
    /// Evaluation claim (inner product result)
    ///
    /// # Errors
    /// When evaluation calculation fails
    pub fn calculate_evaluation_claim(
        &self,
        values: &[P::Scalar],
        evaluation_point: &[P::Scalar],
    ) -> Result<P::Scalar, String> {
        // Compute inner product with equality polynomial
        let evaluation_claim = inner_product::<P::Scalar>(
            values.to_vec(),
            eq_ind_partial_eval(evaluation_point)
                .as_ref()
                .iter()
                .copied()
                .collect_vec(),
        );

        Ok(evaluation_claim)
    }

    /// Generate a polynomial commitment and codeword
    ///
    /// # Arguments
    /// * `packed_mle` - Packed multilinear extension to commit to
    /// * `fri_params` - FRI protocol parameters
    /// * `ntt` - Number Theoretic Transform instance
    ///
    /// # Returns
    /// Commitment output containing commitment and codeword
    ///
    /// # Errors
    /// When commitment generation fails
    pub fn commit(
        &self,
        packed_mle: FieldBuffer<P>,
        fri_params: FRIParams<P::Scalar>,
        ntt: &NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
    ) -> Result<CommitmentOutput<P>, String> {
        let pcs = PCSProver::new(ntt, &self.merkle_prover, &fri_params);
        pcs.commit(packed_mle.to_ref()).map_err(|e| e.to_string())
    }

    /// Generate an evaluation proof for the committed polynomial
    ///
    /// # Arguments
    /// * `packed_mle` - Packed multilinear extension
    /// * `fri_params` - FRI protocol parameters
    /// * `ntt` - Number Theoretic Transform instance
    /// * `commit_output` - Previous commitment output
    /// * `evaluation_point` - Point at which to evaluate the polynomial
    ///
    /// # Returns
    /// Tuple containing terminal codeword, query prover, and transcript bytes
    ///
    /// # Errors
    /// When proof generation fails
    pub fn prove<'b>(
        &'b self,
        packed_mle: FieldBuffer<P>,
        fri_params: &'b FRIParams<P::Scalar>,
        ntt: &'b NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
        commit_output: &'b CommitmentOutput<P>,
        evaluation_point: &[P::Scalar],
    ) -> ProveResult<'b, P> {
        let pcs = PCSProver::new(ntt, &self.merkle_prover, fri_params);

        let mut prover_transcript = ProverTranscript::new(StdChallenger::default());

        // Write commitment to transcript
        prover_transcript.message().write(&commit_output.commitment);

        let eval_point_eq = eq_ind_partial_eval(evaluation_point);
        let _evaluation_claim = inner_product_buffers(&packed_mle, &eval_point_eq);

        // Use prove_with_openings instead of prove
        let (terminate_codeword, query_prover) = pcs
            .prove_with_openings(
                commit_output.codeword.clone(),
                &commit_output.committed,
                packed_mle,
                evaluation_point,
                _evaluation_claim,
                &mut prover_transcript,
            )
            .map_err(|e| e.to_string())?;

        // Get transcript bytes
        let transcript_bytes = prover_transcript.finalize();

        Ok((terminate_codeword, query_prover, transcript_bytes))
    }

    /// Encode data using Reed-Solomon code with NTT
    #[allow(dead_code)]
    pub fn encode_codeword(
        &self,
        data: &[P::Scalar],
        fri_params: FRIParams<P::Scalar>,
        ntt: &NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
    ) -> Result<Vec<P::Scalar>, String> {
        let rs_code = fri_params.rs_code();
        let len = 1
            << (rs_code.log_dim() + fri_params.log_batch_size() - P::LOG_WIDTH
                + rs_code.log_inv_rate());

        let mut encoded = Vec::with_capacity(len);

        let data_log_len = rs_code.log_dim() + fri_params.log_batch_size();
        let encoded_buffer = rs_code.encode_batch(
            ntt,
            FieldSlice::from_slice(data_log_len, data),
            fri_params.log_batch_size(),
        );
        encoded.extend_from_slice(encoded_buffer.as_ref());

        Ok(encoded)
    }

    /// Compute Lagrange interpolation at a specific point
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
}

impl<'a, P, VCS, NTT> FriVailSampling<P, NTT> for FriVail<'a, P, VCS, NTT>
where
    NTT: AdditiveNTT<Field = B128> + Sync,
    P: PackedField<Scalar = B128> + PackedExtension<B128> + PackedExtension<B1>,
    VCS: MerkleTreeScheme<P::Scalar>,
{
    /// Decode a Reed-Solomon codeword with error correction for missing points
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
    ) -> Result<(), String> {
        let n = corrupted_codeword.len();
        let domain = (0..corrupted_codeword.len())
            .map(|i| P::Scalar::from(i as u128))
            .collect::<Vec<_>>();
        if corrupted_indices.is_empty() {
            return Ok(());
        }

        // Collect known points (x_j, y_j)
        let known: Vec<(P::Scalar, P::Scalar)> = (0..n)
            .filter(|i| !corrupted_indices.contains(i))
            .map(|i| (domain[i], corrupted_codeword[i]))
            .collect();

        let k = known.len();
        if k == 0 {
            return Err("No known points available for reconstruction".into());
        }

        // For each erased position, interpolate and evaluate
        #[cfg(feature = "parallel")]
        {
            // Parallel version using rayon
            let reconstructed_values: Vec<(usize, P::Scalar)> = corrupted_indices
                .par_iter()
                .map(|&missing| {
                    debug!("Calculating value for missing index: {}", missing);
                    let x_e = domain[missing];
                    let value = self.interpolate_at_point(x_e, &known, k);

                    debug!(
                        "Reconstructed value for missing index {}: {:?}",
                        missing, value
                    );
                    (missing, value)
                })
                .collect();

            // Apply the reconstructed values to the codeword
            for (missing, value) in reconstructed_values {
                corrupted_codeword[missing] = value;
            }
        }

        #[cfg(not(feature = "parallel"))]
        {
            // Sequential version
            for &missing in corrupted_indices {
                debug!("Calculating value for missing index: {}", missing);
                let x_e = domain[missing];
                let value = self.interpolate_at_point(x_e, &known, k);

                debug!(
                    "Reconstructed value for missing index {}: {:?}",
                    missing, value
                );
                corrupted_codeword[missing] = value;
            }
        }

        Ok(())
    }

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
    ) -> Result<(), String> {
        // Extract commitment from transcript
        let retrieved_codeword_commitment = verifier_transcript
            .message()
            .read()
            .map_err(|e| e.to_string())?;

        let merkle_prover_scheme = self.merkle_prover.scheme().clone();

        let n_packed_vars = fri_params.rs_code().log_dim() + fri_params.log_batch_size();
        let eval_point = &evaluation_point[..n_packed_vars];

        // Verify and get verifier_with_arena using the verifier_with_arena pattern
        let verifier_with_arena = spartan_verify(
            verifier_transcript,
            evaluation_claim,
            eval_point,
            retrieved_codeword_commitment,
            fri_params,
            &merkle_prover_scheme,
        )
        .map_err(|e| e.to_string())?;

        // Get the verifier from arena (demonstrates the verifier_with_arena pattern)
        let verifier = verifier_with_arena.verifier();

        // If extra parameters provided, perform extra query verification
        if let (Some(idx), Some(codeword), Some(layers), Some(extra_transcript)) =
            (extra_index, terminate_codeword, layers, extra_transcript)
        {
            // Verify layers match commitments using vcs_optimal_layers_depths_iter
            for (commitment, layer_depth, layer) in izip!(
                std::iter::once(verifier.codeword_commitment).chain(verifier.round_commitments),
                vcs_optimal_layers_depths_iter(verifier.params, verifier.vcs),
                layers
            ) {
                verifier
                    .vcs
                    .verify_layer(commitment, layer_depth, layer)
                    .map_err(|e| e.to_string())?;
            }

            // Create advice reader from extra transcript for query verification
            let mut advice = extra_transcript.decommitment();

            // Verify the extra query proof
            verifier
                .verify_query(idx, ntt, codeword, layers, &mut advice)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

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
    ) -> TranscriptResult {
        let mut proof_writer = ProverTranscript::new(StdChallenger::default());
        self.merkle_prover
            .prove_opening(committed, 0, index, &mut proof_writer.message())
            .map_err(|e| e.to_string())?;

        let proof_reader = proof_writer.into_verifier();

        Ok(proof_reader)
    }

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
    fn open<'b>(
        &self,
        index: usize,
        query_prover: &FRIQueryProverAlias<'b, P>,
    ) -> TranscriptResult {
        // Create new transcript for the query proof
        let mut proof_transcript = ProverTranscript::new(StdChallenger::default());
        let mut advice = proof_transcript.decommitment();

        // Generate proof for specific index
        query_prover
            .prove_query(index, &mut advice)
            .map_err(|e| e.to_string())?;

        // Return verifier transcript
        Ok(proof_transcript.into_verifier())
    }

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
    ) -> Result<(), String> {
        let tree_depth = fri_params.rs_code().log_len();
        self.merkle_prover
            .scheme()
            .verify_opening(
                index,
                data,
                0,
                tree_depth,
                &[commitment.into()],
                &mut verifier_transcript.message(),
            )
            .map_err(|e| e.to_string())
    }

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
    ) -> FieldResult<P> {
        let rs_code = fri_params.rs_code();
        let len = 1 << (rs_code.log_len() + fri_params.log_batch_size() - P::LOG_WIDTH);

        let mut decoded = Vec::with_capacity(len);
        self.decode_batch(
            rs_code.log_len(),
            rs_code.log_inv_rate(),
            fri_params.log_batch_size(),
            ntt,
            codeword.as_ref(),
            decoded.spare_capacity_mut(),
        )
        .map_err(|e| e.to_string())?;

        unsafe {
            // Safety: decode_batch guarantees all elements are initialized on success
            decoded.set_len(len);
        }

        // Trim to original data size (remove redundancy)
        let trim_len = 1 << (rs_code.log_dim() + fri_params.log_batch_size() - P::LOG_WIDTH);
        decoded.resize(trim_len, P::Scalar::zero());

        // Undo bit-reversal that encode_batch applied internally
        let data_log_len = rs_code.log_dim() + fri_params.log_batch_size();
        bit_reverse_packed(FieldSliceMut::from_slice(
            data_log_len,
            decoded.as_mut_slice(),
        ));

        Ok(decoded)
    }

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
    #[allow(dead_code)]
    fn extract_commitment(
        &self,
        verifier_transcript: &mut VerifierTranscript<StdChallenger>,
    ) -> ByteResult {
        verifier_transcript
            .message()
            .read()
            .map_err(|e| e.to_string())
    }

    /// Low-level batch decoding using inverse NTT
    ///
    /// # Arguments
    /// * `log_len` - Logarithm of dimension
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
        log_len: usize,
        log_inv: usize,
        log_batch_size: usize,
        ntt: &NeighborsLastMultiThread<GenericPreExpanded<P::Scalar>>,
        data: &[P::Scalar],
        output: &mut [MaybeUninit<P::Scalar>],
    ) -> Result<(), String> {
        let data_log_len = log_len + log_batch_size;

        let expected_data_len = if data_log_len >= P::LOG_WIDTH {
            1 << (data_log_len - P::LOG_WIDTH)
        } else {
            1
        };

        if data.len() != expected_data_len {
            return Err(format!(
                "Unexpected data length: {} {} ",
                expected_data_len,
                data.len()
            ));
        }

        let _scope = tracing::trace_span!(
            "Reed-Solomon encode",
            log_len = log_len,
            log_batch_size = log_batch_size,
        )
        .entered();

        let data_portion_len = data.len();

        for i in 0..data_portion_len {
            output[i].write(data[i]);
        }

        for i in data_portion_len..output.len() {
            output[i].write(P::Scalar::zero());
        }

        let output_initialized =
            unsafe { uninit::out_ref::Out::<[P::Scalar]>::from(output).assume_init() };
        let mut code = FieldSliceMut::from_slice(log_len + log_batch_size, output_initialized);

        let skip_early = log_inv;
        let skip_late = log_batch_size;

        // TODO: create an optimised version PR to binius 64 for inverse_ntt
        let log_d = code.log_len();
        use binius_math::ntt::DomainContext;
        for layer in (skip_early..(log_d - skip_late)).rev() {
            let num_blocks = 1 << layer;
            let block_size_half = 1 << (log_d - layer - 1);
            for block in 0..num_blocks {
                let twiddle = ntt.domain_context().twiddle(layer, block);
                let block_start = block << (log_d - layer);
                for idx0 in block_start..(block_start + block_size_half) {
                    let idx1 = block_size_half | idx0;
                    // perform butterfly
                    let mut u = code.get(idx0);
                    let mut v = code.get(idx1);

                    v += u;
                    u += v * twiddle;
                    code.set(idx0, u);
                    code.set(idx1, v);
                }
            }
        }

        Ok(())
    }
}

impl FriVailUtils for FriVailDefault {
    fn get_transcript_bytes(&self, transcript: &VerifierTranscript<StdChallenger>) -> Vec<u8> {
        let mut cloned = transcript.clone();
        let mut message_reader = cloned.message();
        let buffer = message_reader.buffer();
        let remaining = buffer.remaining();

        if remaining == 0 {
            return Vec::new();
        }

        // Read all remaining bytes
        let mut bytes = vec![0u8; remaining];
        buffer.copy_to_slice(&mut bytes);
        bytes
    }
    fn reconstruct_transcript_from_bytes(
        &self,
        bytes: Vec<u8>,
    ) -> VerifierTranscript<StdChallenger> {
        VerifierTranscript::new(StdChallenger::default(), bytes)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::poly::Utils;
    use binius_math::ntt::{domain_context::GenericPreExpanded, NeighborsLastMultiThread};
    use binius_verifier::{
        config::B128,
        hash::{StdCompression, StdDigest},
        merkle_tree::BinaryMerkleTreeScheme,
    };

    fn create_test_data(size_bytes: usize) -> Vec<u8> {
        (0..size_bytes).map(|i| (i % 256) as u8).collect()
    }

    #[test]
    fn test_friveil_new() {
        const LOG_INV_RATE: usize = 1;
        const NUM_TEST_QUERIES: usize = 3;
        const N_VARS: usize = 10;
        const LOG_NUM_SHARES: usize = 2;

        let friVail = TestFriVail::new(LOG_INV_RATE, NUM_TEST_QUERIES, 2, N_VARS, LOG_NUM_SHARES);

        assert_eq!(friVail.log_inv_rate, LOG_INV_RATE);
        assert_eq!(friVail.num_test_queries, NUM_TEST_QUERIES);
        assert_eq!(friVail.n_vars, N_VARS);
        assert_eq!(friVail.log_num_shares, LOG_NUM_SHARES);
    }

    #[test]
    fn test_calculate_evaluation_point_random() {
        const N_VARS: usize = 8;
        let friVail = TestFriVail::new(1, 3, 2, N_VARS, 2);

        let result = friVail.calculate_evaluation_point_random();
        assert!(result.is_ok());

        let evaluation_point = result.unwrap();
        assert_eq!(evaluation_point.len(), N_VARS);

        // Test deterministic behavior with fixed seed
        let result2 = friVail.calculate_evaluation_point_random();
        assert!(result2.is_ok());
        let evaluation_point2 = result2.unwrap();
        assert_eq!(evaluation_point, evaluation_point2);
    }

    #[test]
    fn test_initialize_fri_context() {
        let friVail = TestFriVail::new(1, 3, 2, 12, 2);

        // Create test data
        let test_data = create_test_data(1024); // 1KB test data
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let result = friVail.initialize_fri_context(packed_mle_values.packed_mle.log_len());
        assert!(result.is_ok());

        let (fri_params, _ntt) = result.unwrap();

        // Verify FRI parameters are reasonable
        assert_eq!(fri_params.rs_code().log_inv_rate(), friVail.log_inv_rate);
        assert_eq!(fri_params.n_test_queries(), friVail.num_test_queries);
    }

    #[test]
    #[ignore]
    fn test_commit_and_inclusion_proofs() {
        // Create test data
        let test_data = create_test_data(1024);
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 2);

        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        // Test commit
        let commit_result = friVail.commit(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
        );
        assert!(commit_result.is_ok());

        let commit_output = commit_result.unwrap();
        assert!(!commit_output.commitment.is_empty());
        assert!(commit_output.codeword.len() > 0);

        let commitment_bytes: [u8; 32] = commit_output
            .commitment
            .to_vec()
            .try_into()
            .expect("We know commitment size is 32 bytes");
        // Test inclusion proofs for first few elements
        for i in 0..std::cmp::min(5, commit_output.codeword.len()) {
            let value = commit_output.codeword[i];

            // Generate inclusion proof
            let inclusion_proof_result = friVail.inclusion_proof(&commit_output.committed, i);
            assert!(inclusion_proof_result.is_ok());

            let mut inclusion_proof = inclusion_proof_result.unwrap();

            // Verify inclusion proof
            let verify_result = friVail.verify_inclusion_proof(
                &mut inclusion_proof,
                &[value],
                i,
                &fri_params,
                commitment_bytes,
            );
            assert!(
                verify_result.is_ok(),
                "Inclusion proof verification failed for index {}",
                i
            );
        }
    }

    #[test]
    #[ignore]
    fn test_open_method() {
        // Create test data
        let test_data = create_test_data(1024);
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 2);

        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        // Test commit
        let commit_result = friVail.commit(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
        );
        assert!(commit_result.is_ok());

        let commit_output = commit_result.unwrap();
        assert!(!commit_output.commitment.is_empty());
        assert!(commit_output.codeword.len() > 0);

        // Generate evaluation point for prove
        let evaluation_point = friVail
            .calculate_evaluation_point_random()
            .expect("Failed to generate evaluation point");

        // Generate proof to get query_prover
        let prove_result = friVail.prove(
            packed_mle_values.packed_mle.clone(),
            &fri_params,
            &ntt,
            &commit_output,
            &evaluation_point,
        );
        assert!(prove_result.is_ok());

        let (_, query_prover, _) = prove_result.unwrap();

        // Test that open() method works with query_prover
        for i in 0..std::cmp::min(5, commit_output.codeword.len()) {
            let open_result = friVail.open(i, &query_prover);
            assert!(open_result.is_ok(), "open() method failed for index {}", i);
        }
    }

    #[test]
    fn test_calculate_evaluation_claim() {
        let test_data = create_test_data(1024); // 1mb test data
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 3);

        let evaluation_point = friVail
            .calculate_evaluation_point_random()
            .expect("Failed to generate evaluation point");

        println!("evaluation point {:?}", evaluation_point.len());
        let eval_point_eq = eq_ind_partial_eval(&evaluation_point);
        println!("eval_point_eq {:?}", eval_point_eq.len());
        println!("mle value {:?}", packed_mle_values.packed_mle.len());
        let evaluation_claim = inner_product_buffers(&packed_mle_values.packed_mle, &eval_point_eq);

        println!("evaluation claim {:?}", evaluation_claim);

        let result =
            friVail.calculate_evaluation_claim(&packed_mle_values.packed_values, &evaluation_point);
        assert!(result.is_ok());

        let evaluation_claim = result.unwrap();
        // The evaluation claim should be a valid field element
        assert_ne!(evaluation_claim, B128::default()); // Should not be zero for random inputs
    }

    #[test]
    fn test_full_prove_verify_workflow() {
        // Create test data
        let test_data = create_test_data(1024 * 1024); // 2KB test data
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 3);
        // Initialize FRI context
        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        // Generate evaluation point
        let evaluation_point = friVail
            .calculate_evaluation_point_random()
            .expect("Failed to generate evaluation point");
        let eval_point_eq = eq_ind_partial_eval(&evaluation_point);
        let evaluation_claim = inner_product_buffers(&packed_mle_values.packed_mle, &eval_point_eq);

        println!("evaluation claim {:?}", evaluation_claim);
        // The evaluation claim should be a valid field element
        assert_ne!(evaluation_claim, B128::default()); // Should not be zero for random inputs

        // Commit to MLE
        let commit_output = friVail
            .commit(
                packed_mle_values.packed_mle.clone(),
                fri_params.clone(),
                &ntt,
            )
            .expect("Failed to commit");

        // Generate proof
        let prove_result = friVail.prove(
            packed_mle_values.packed_mle.clone(),
            &fri_params,
            &ntt,
            &commit_output,
            &evaluation_point,
        );
        assert!(prove_result.is_ok());

        let (terminate_codeword, query_prover, transcript_bytes) = prove_result.unwrap();

        // Extract layers directly from query_prover
        let layers = query_prover
            .vcs_optimal_layers()
            .expect("Failed to get layers");

        // Reconstruct verifier transcript from bytes
        let mut verifier_transcript =
            VerifierTranscript::new(StdChallenger::default(), transcript_bytes);

        // Recalculate evaluation claim
        let eval_point_eq = eq_ind_partial_eval(&evaluation_point);
        let evaluation_claim = inner_product_buffers(&packed_mle_values.packed_mle, &eval_point_eq);

        // Convert terminate_codeword to vector of scalars
        let terminate_codeword_vec: Vec<_> = terminate_codeword.iter_scalars().collect();

        // Generate extra query proof using open()
        let mut extra_transcript = friVail
            .open(0, &query_prover)
            .expect("Failed to generate extra query proof");

        // Verify proof with extra parameters
        let verify_result = friVail.verify(
            &mut verifier_transcript,
            evaluation_claim,
            &evaluation_point,
            &fri_params,
            &ntt,                          // ntt instance
            Some(0),                       // extra_index - use 0 for testing
            Some(&terminate_codeword_vec), // terminate_codeword
            Some(&layers),                 // layers
            Some(&mut extra_transcript),   // extra query transcript
        );
        assert!(
            verify_result.is_ok(),
            "Verification failed: {:?}",
            verify_result
        );
    }

    #[test]
    fn test_invalid_verification_fails() {
        // Create test data
        let test_data = create_test_data(512);
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");
        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 3);
        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        let commit_output = friVail
            .commit(
                packed_mle_values.packed_mle.clone(),
                fri_params.clone(),
                &ntt,
            )
            .expect("Failed to commit");

        let evaluation_point = friVail
            .calculate_evaluation_point_random()
            .expect("Failed to generate evaluation point");

        let (_terminate_codeword, _query_prover, transcript_bytes) = friVail
            .prove(
                packed_mle_values.packed_mle.clone(),
                &fri_params,
                &ntt,
                &commit_output,
                &evaluation_point,
            )
            .expect("Failed to generate proof");

        // Reconstruct verifier transcript from bytes
        let mut verifier_transcript =
            VerifierTranscript::new(StdChallenger::default(), transcript_bytes);

        // Use wrong evaluation claim (should cause verification to fail)
        let wrong_evaluation_claim = B128::from(42u128);

        let verify_result = friVail.verify(
            &mut verifier_transcript,
            wrong_evaluation_claim,
            &evaluation_point,
            &fri_params,
            &ntt, // ntt instance
            None,
            None,
            None,
            None, // no extra transcript
        );

        // Verification should fail with wrong claim
        assert!(
            verify_result.is_err(),
            "Verification should fail with wrong evaluation claim"
        );
    }

    #[test]
    fn test_data_availability_sampling() {
        use rand::{rngs::StdRng, seq::index::sample, SeedableRng};
        use tracing::Level;

        // Initialize logging for the test
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_test_writer()
            .try_init();

        // Create test data
        let test_data = create_test_data(512); // 512 bytes test data
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 2);

        // Initialize FRI context
        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        // Commit to MLE
        let commit_output = friVail
            .commit(
                packed_mle_values.packed_mle.clone(),
                fri_params.clone(),
                &ntt,
            )
            .expect("Failed to commit");

        println!(
            "commit output codeword len {:?}",
            commit_output.codeword.len()
        );

        let total_samples = commit_output.codeword.len();
        let sample_size = std::cmp::min(5, total_samples / 4); // Limit to 5 samples or 1/4 of total
        let indices =
            sample(&mut StdRng::from_seed([0; 32]), total_samples, sample_size).into_vec();
        let commitment_bytes: [u8; 32] = commit_output
            .commitment
            .to_vec()
            .try_into()
            .expect("We know commitment size is 32 bytes");

        let mut successful_samples = 0;
        let mut failed_samples = Vec::new();

        for &sample_index in indices.iter() {
            println!("sample index {sample_index}");
            match friVail.inclusion_proof(&commit_output.committed, sample_index) {
                Ok(mut inclusion_proof) => {
                    let value = commit_output.codeword[sample_index];
                    match friVail.verify_inclusion_proof(
                        &mut inclusion_proof,
                        &[value],
                        sample_index,
                        &fri_params,
                        commitment_bytes,
                    ) {
                        Ok(_) => {
                            successful_samples += 1;
                        }
                        Err(e) => {
                            failed_samples
                                .push((sample_index, format!("Verification failed: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    failed_samples.push((
                        sample_index,
                        format!("Inclusion proof generation failed: {}", e),
                    ));
                }
            }
        }

        assert_eq!(failed_samples.len(), 0, "Some samples failed verification");
        assert_eq!(
            successful_samples, sample_size,
            "Not all samples were verified"
        );

        println!("Successfully verified {} samples", successful_samples);
    }

    #[test]
    fn test_codeword_decode() {
        // Create test data
        let test_data = create_test_data(512);
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 3);

        // Initialize FRI context
        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        // Encode codeword
        let encoded_codeword = friVail
            .encode_codeword(&packed_mle_values.packed_values, fri_params.clone(), &ntt)
            .expect("Failed to encode codeword");

        // Decode codeword
        let decoded_codeword = friVail
            .decode_codeword(&encoded_codeword, fri_params.clone(), &ntt)
            .expect("Failed to decode codeword");

        // Verify decoded codeword matches original values
        assert_eq!(
            decoded_codeword, packed_mle_values.packed_values,
            "Decoded codeword should match original packed values"
        );

        println!("✅ Codeword decode test passed");
    }

    #[test]
    fn test_error_correction_reconstruction() {
        use rand::{rngs::StdRng, seq::index::sample, SeedableRng};

        // Create test data
        let test_data = create_test_data(2048);
        let packed_mle_values = Utils::<B128>::new()
            .bytes_to_packed_mle(&test_data)
            .expect("Failed to create packed MLE");

        let friVail = TestFriVail::new(1, 3, 2, packed_mle_values.packed_mle.log_len(), 3);

        // Initialize FRI context
        let (fri_params, ntt) = friVail
            .initialize_fri_context(packed_mle_values.packed_mle.log_len())
            .expect("Failed to initialize FRI context");

        // Encode codeword
        let encoded_codeword = friVail
            .encode_codeword(&packed_mle_values.packed_values, fri_params.clone(), &ntt)
            .expect("Failed to encode codeword");

        // Corrupt the codeword
        let mut corrupted_codeword = encoded_codeword.clone();
        let total_elements = corrupted_codeword.len();
        let corruption_percentage = 0.1;

        // Corrupt random elements
        let num_corrupted = (total_elements as f64 * corruption_percentage) as usize;
        let mut rng = StdRng::seed_from_u64(42);
        let corrupted_indices = sample(&mut rng, total_elements, num_corrupted).into_vec();

        for &index in &corrupted_indices {
            corrupted_codeword[index] = B128::zero();
        }

        // Verify corruption happened
        assert_ne!(
            corrupted_codeword, encoded_codeword,
            "Codeword should be corrupted"
        );

        // Reconstruct corrupted codeword
        friVail
            .reconstruct_codeword_naive(&mut corrupted_codeword, &corrupted_indices)
            .expect("Failed to reconstruct codeword");

        // Verify reconstruction succeeded
        assert_eq!(
            corrupted_codeword, encoded_codeword,
            "Reconstructed codeword should match original encoded codeword"
        );

        // Decode the reconstructed codeword to verify it's correct
        let decoded_reconstructed = friVail
            .decode_codeword(&corrupted_codeword, fri_params.clone(), &ntt)
            .expect("Failed to decode reconstructed codeword");

        // Verify decoded reconstructed codeword matches original values
        assert_eq!(
            decoded_reconstructed, packed_mle_values.packed_values,
            "Decoded reconstructed codeword should match original packed values"
        );

        println!(
            "✅ Error correction reconstruction test passed: {} elements, {:.1}% corruption",
            total_elements,
            corruption_percentage * 100.0
        );
    }
}
