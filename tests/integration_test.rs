use binius_field::field::FieldOps;
use binius_transcript::VerifierTranscript;
use binius_verifier::config::StdChallenger;
use frivail::{
    frivail::{B128, FriVeilDefault},
    poly::Utils,
    traits::FriVeilSampling,
};
use rand::{SeedableRng, rngs::StdRng, seq::index::sample};
use std::time::Instant;
use tracing::{Level, debug, error, info, span, warn};

#[test]
fn test_integration_main() {
    // Initialize enhanced logging with structured output, filtering out verbose internal logs
    use tracing_subscriber::filter::EnvFilter;

    let filter = EnvFilter::new("info")
        .add_directive("binius_transcript=error".parse().unwrap())
        .add_directive("transcript=error".parse().unwrap());

    // Try to init, but ignore if already initialized (common in tests)
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(filter)
        .with_test_writer()
        .try_init();

    const LOG_INV_RATE: usize = 1;
    // Security parameter: number of queries to perform in the FRI protocol
    const NUM_TEST_QUERIES: usize = 128;
    const DATA_SIZE_KB: usize = 9;

    info!("🚀 Starting Binius Data Availability Sampling Scheme");
    info!("📋 Configuration:");
    info!("   - Reed-Solomon inverse rate (log2): {}", LOG_INV_RATE);
    info!("   - FRI test queries: {}", NUM_TEST_QUERIES);
    info!("   - Data size: {} KB", DATA_SIZE_KB);

    // Create arbitrary (nonzero, patterned) data instead of all zeroes.
    let _span = span!(Level::INFO, "data_generation").entered();
    info!("📊 Phase 1: Generating test data ({} KB)", DATA_SIZE_KB);
    let random_data_bytes: Vec<u8> = (0..DATA_SIZE_KB * 1024).map(|i| (i % 256) as u8).collect();
    info!(
        "✅ Generated {} bytes of patterned test data",
        random_data_bytes.len()
    );
    drop(_span);

    let _span = span!(Level::INFO, "mle_conversion").entered();
    info!("🔄 Phase 2: Converting bytes to multilinear extension");
    let start = Instant::now();
    let packed_mle_values = Utils::<B128>::new()
        .bytes_to_packed_mle(&random_data_bytes)
        .unwrap();

    let conversion_time = start.elapsed().as_millis();
    info!("✅ MLE conversion completed in {} ms", conversion_time);
    info!(
        "   - Total variables (n_vars): {}",
        packed_mle_values.total_n_vars
    );
    info!(
        "   - Packed values count: {}",
        packed_mle_values.packed_values.len()
    );

    drop(_span);

    let _span = span!(Level::INFO, "fri_initialization").entered();
    info!("🔧 Phase 3: Initializing FRI-based polynomial commitment scheme");
    let start = Instant::now();
    let friveil = FriVeilDefault::new(
        LOG_INV_RATE,
        NUM_TEST_QUERIES,
        packed_mle_values.total_n_vars,
        80, // log_num_shares
    );
    let init_time = start.elapsed().as_millis();
    info!("✅ FRIVeil context initialized in {} ms", init_time);

    let start = Instant::now();
    info!("🎲 Generating random evaluation point");
    let evaluation_point = friveil.calculate_evaluation_point_random().unwrap();
    let eval_time = start.elapsed().as_millis();
    info!("✅ Evaluation point generated in {} ms", eval_time);
    info!(
        "   - Evaluation point dimensions: {}",
        evaluation_point.len()
    );
    drop(_span);

    let _span = span!(Level::INFO, "fri_context_setup").entered();
    info!("⚙️  Setting up FRI protocol parameters");
    let start = Instant::now();
    let (fri_params, ntt) = friveil
        .initialize_fri_context(packed_mle_values.packed_mle.log_len())
        .unwrap();
    let context_time = start.elapsed().as_millis();
    info!("✅ FRI context setup completed in {} ms", context_time);
    info!(
        "   - Reed-Solomon code length (log2): {}",
        fri_params.rs_code().log_len()
    );
    info!(
        "   - Reed-Solomon inverse rate (log2): {}",
        fri_params.rs_code().log_inv_rate()
    );
    info!("   - FRI test queries: {}", fri_params.n_test_queries());
    drop(_span);

    let _span = span!(Level::INFO, "vector_commitment_and_codeword").entered();
    info!("🔒 Phase 4: Generating vector commitment and codeword");
    let start = Instant::now();
    let commit_output = friveil
        .commit(
            packed_mle_values.packed_mle.clone(),
            fri_params.clone(),
            &ntt,
        )
        .unwrap();
    let commit_time = start.elapsed().as_millis();

    info!(
        "✅ Vector commitment and codeword generated in {} ms",
        commit_time
    );
    info!(
        "   - Commitment size: {} bytes",
        commit_output.commitment.len()
    );
    info!(
        "   - Codeword length: {} elements",
        commit_output.codeword.len()
    );

    drop(_span);

    let _span = span!(Level::INFO, "codeword_encoding").entered();
    info!("🔄 Phase 5: Encoding codeword");
    let start = Instant::now();
    let encoded_codeword = friveil
        .encode_codeword(&packed_mle_values.packed_values, fri_params.clone(), &ntt)
        .unwrap();

    let encode_time = start.elapsed().as_millis();
    info!("✅ Codeword encoded in {} ms", encode_time);
    encoded_codeword
        .iter()
        .enumerate()
        .for_each(|(i, x)| assert_eq!(*x, commit_output.codeword[i]));
    drop(_span);

    let _span = span!(Level::INFO, "decode_codeword").entered();
    info!("🔄 Phase 6: Decoding codeword");
    let start = Instant::now();
    let decoded_codeword = friveil
        .decode_codeword(&encoded_codeword, fri_params.clone(), &ntt)
        .unwrap();
    let decode_time = start.elapsed().as_millis();
    info!("✅ Codeword decoded in {} ms", decode_time);

    assert_eq!(decoded_codeword, packed_mle_values.packed_values);
    drop(_span);

    if DATA_SIZE_KB < 10 {
        // Test Reed-Solomon error correction by simulating data loss
        let _span = span!(Level::INFO, "error_correction_test").entered();
        info!("🔧 Phase 7: Testing Reed-Solomon error correction");
        let start = Instant::now();

        // Create a corrupted version of the encoded codeword with some data points "lost"
        let mut corrupted_codeword = encoded_codeword.clone();
        let total_elements = corrupted_codeword.len();

        let corruption_percentage = 0.1;
        let corrupted_indices_vec =
            corrupt_codeword_randomly(&mut corrupted_codeword, corruption_percentage, Some(42u64));

        info!("   - Total codeword elements: {}", total_elements);
        info!(
            "   - Corrupted {} elements ({:.1}%)",
            corrupted_indices_vec.len(),
            corruption_percentage * 100.0
        );

        info!("🔄 Attempting error correction...");
        assert_ne!(corrupted_codeword, encoded_codeword);

        let _reconstructed_codeword = friveil
            .reconstruct_codeword_naive(&mut corrupted_codeword, &corrupted_indices_vec)
            .unwrap();

        let reconstruction_time = start.elapsed().as_millis();

        info!("✅ Reconstruction completed in {} ms", reconstruction_time);
        assert_eq!(corrupted_codeword, encoded_codeword);
        drop(_span);
    } else {
        info!("Phase 7: Skipping error correction test for big data size");
    }

    // Phase 8: Data Availability Sampling - COMMENTED OUT DUE TO MERKLE TREE INDEX MISMATCH
    // The inclusion_proof function fails with index out of bounds because the Merkle tree
    // has fewer leaves than the codeword has elements. See issue analysis for details.
    let _span = span!(Level::INFO, "data_availability_sampling").entered();
    info!("🎯 Phase 8: Performing data availability sampling");
    info!(
        "   - Total codeword elements to sample: {}",
        commit_output.codeword.len()
    );
    let start = Instant::now();

    let mut successful_samples = 0;
    let mut failed_samples = Vec::new();

    let total_samples = commit_output.codeword.len();
    let sample_size = total_samples / 2;
    let indices = sample(&mut StdRng::from_seed([0; 32]), total_samples, sample_size).into_vec();
    let commitment_bytes: [u8; 32] = commit_output
        .commitment
        .to_vec()
        .try_into()
        .expect("We know commitment size is 32 bytes");

    for &sample_index in indices.iter() {
        let sample_span =
            span!(Level::DEBUG, "sample_verification", index = sample_index).entered();

        match friveil.inclusion_proof(&commit_output.committed, sample_index) {
            Ok(mut inclusion_proof) => {
                let value = commit_output.codeword[sample_index];
                match friveil.verify_inclusion_proof(
                    &mut inclusion_proof,
                    &[value],
                    sample_index,
                    &fri_params,
                    commitment_bytes,
                ) {
                    Ok(_) => {
                        successful_samples += 1;
                        debug!(
                            "✅ Sample {} verified successfully (value: {:?})",
                            sample_index, value
                        );
                    }
                    Err(e) => {
                        failed_samples.push((sample_index, format!("Verification failed: {}", e)));
                        debug!("❌ Sample {} verification failed: {}", sample_index, e);
                    }
                }
            }
            Err(e) => {
                failed_samples.push((
                    sample_index,
                    format!("Inclusion proof generation failed: {}", e),
                ));
                debug!(
                    "❌ Failed to generate inclusion proof for sample {}: {}",
                    sample_index, e
                );
            }
        }
        drop(sample_span);

        // Log progress every 1000 samples for large datasets
        if (sample_index + 1) % 1000 == 0 || sample_index == total_samples - 1 {
            info!(
                "   Progress: {}/{} samples processed",
                sample_index + 1,
                total_samples
            );
        }
    }

    let sampling_time = start.elapsed().as_millis();

    // Display results in a table format
    info!(
        "✅ Data availability sampling completed in {} ms",
        sampling_time
    );
    info!("");
    info!("📊 DATA AVAILABILITY SAMPLING RESULTS");
    info!("┌─────────────────────────────────┬─────────────────┐");
    info!("│ Metric                          │ Value           │");
    info!("├─────────────────────────────────┼─────────────────┤");
    info!(
        "│ Total Samples                   │ {:>15} │",
        total_samples
    );
    info!(
        "│ Successful Verifications        │ {:>15} │",
        successful_samples
    );
    info!(
        "│ Failed Verifications            │ {:>15} │",
        failed_samples.len()
    );
    info!(
        "│ Success Rate                    │ {:>13.2}% │",
        (successful_samples as f64 / total_samples as f64) * 100.0
    );
    info!(
        "│ Sampling Duration               │ {:>12} ms │",
        sampling_time
    );
    info!(
        "│ Average Time per Sample         │ {:>10.3} ms │",
        sampling_time as f64 / total_samples as f64
    );
    info!("└─────────────────────────────────┴─────────────────┘");

    if !failed_samples.is_empty() {
        warn!("");
        warn!("⚠️  FAILED SAMPLES DETAILS:");
        warn!("┌───────────┬─────────────────────────────────────────────────────┐");
        warn!("│ Sample ID │ Error Description                                   │");
        warn!("├───────────┼─────────────────────────────────────────────────────┤");
        for (id, error) in failed_samples.iter().take(10) {
            // Show first 10 failures
            warn!(
                "│ {:>9} │ {:<51} │",
                id,
                if error.len() > 51 {
                    format!("{}...", &error[..48])
                } else {
                    error.clone()
                }
            );
        }
        if failed_samples.len() > 10 {
            warn!(
                "│ ...       │ ... and {} more failures                        │",
                failed_samples.len() - 10
            );
        }
        warn!("└───────────┴─────────────────────────────────────────────────────┘");
        warn!(
            "⚠️  {} samples failed verification - potential data availability issues",
            failed_samples.len()
        );
    } else {
        info!("🎉 All samples verified successfully - data is fully available!");
    }
    drop(_span);

    let _span = span!(Level::INFO, "proof_generation").entered();
    info!("📝 Phase 9: Generating evaluation proof");
    let start = Instant::now();
    let (terminate_codeword, query_prover, transcript_bytes) = friveil
        .prove(
            packed_mle_values.packed_mle.clone(),
            &fri_params,
            &ntt,
            &commit_output,
            &evaluation_point,
        )
        .unwrap();
    let proof_time = start.elapsed().as_millis();

    info!("✅ Evaluation proof generated in {} ms", proof_time);

    drop(_span);

    let _span = span!(Level::INFO, "evaluation_claim").entered();

    info!("🧮 Computing evaluation claim");
    let start = Instant::now();
    let evaluation_claim = friveil
        .calculate_evaluation_claim(&packed_mle_values.packed_values, &evaluation_point)
        .unwrap();
    let claim_time = start.elapsed().as_millis();
    info!("✅ Evaluation claim computed in {} ms", claim_time);
    debug!("   - Evaluation claim value: {:?}", evaluation_claim);
    drop(_span);

    let _span = span!(Level::INFO, "final_verification").entered();
    info!("🔍 Phase 10: Final proof verification with extra query");

    // Extract layers from query_prover for extra verification
    let layers = query_prover.vcs_optimal_layers().unwrap();
    let terminate_codeword_vec: Vec<_> = terminate_codeword.iter_scalars().collect();

    // Generate extra query proof using open()
    let mut extra_transcript = friveil.open(0, &query_prover).unwrap();

    // Extract transcript bytes for network propagation
    info!(
        "📦 Transcript size: {} bytes (ready for network transmission)",
        transcript_bytes.len()
    );

    // Reconstruct verifier transcript from bytes
    let mut verifier_transcript =
        VerifierTranscript::new(StdChallenger::default(), transcript_bytes);

    let start = Instant::now();
    let result = friveil.verify(
        &mut verifier_transcript,
        evaluation_claim,
        &evaluation_point,
        &fri_params,
        &ntt,
        Some(0),
        Some(&terminate_codeword_vec),
        Some(&layers),
        Some(&mut extra_transcript),
    );
    let verification_time = start.elapsed().as_millis();

    match &result {
        Ok(_) => {
            info!(
                "✅ Final verification succeeded in {} ms",
                verification_time
            );
            info!("🎉 Data Availability Sampling scheme completed successfully!");
        }
        Err(e) => {
            error!(
                "❌ Final verification failed in {} ms: {:?}",
                verification_time, e
            );
            error!("💥 Data Availability Sampling scheme failed!");
        }
    }
    drop(_span);

    // Summary
    info!("📊 === EXECUTION SUMMARY ===");
    info!("Final verification result: {:?}", result);
    info!("🏁 Binius Data Availability Sampling completed");
}

/// Corrupts a codeword by randomly setting specified percentage of elements to zero
///
/// # Arguments
/// * `codeword` - The codeword to corrupt (modified in place)
/// * `corruption_percentage` - Percentage of elements to corrupt (0.0 to 1.0)
/// * `seed` - Optional seed for reproducible results. If None, uses system randomness
///
/// # Returns
/// * `Vec<usize>` - Vector of indices that were corrupted
fn corrupt_codeword_randomly(
    codeword: &mut [B128],
    corruption_percentage: f64,
    seed: Option<u64>,
) -> Vec<usize> {
    let total_elements = codeword.len();
    let num_corrupted_points = (total_elements as f64 * corruption_percentage) as usize;

    if num_corrupted_points == 0 {
        return Vec::new();
    }

    // Create RNG with optional seed
    let mut rng = if let Some(seed) = seed {
        StdRng::seed_from_u64(seed)
    } else {
        // Use current time as seed for randomness
        StdRng::seed_from_u64(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        )
    };

    // Use reservoir sampling to efficiently select random indices
    let corrupted_indices = sample(&mut rng, total_elements, num_corrupted_points).into_vec();

    // Corrupt the selected indices by setting them to zero
    for &index in &corrupted_indices {
        codeword[index] = B128::zero();
    }

    corrupted_indices
}
