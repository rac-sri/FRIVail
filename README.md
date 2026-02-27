# FRI-Vail: Data Availability Sampling (DAS) Library

A Rust implementation of Data Availability Sampling using the Binius polynomial commitment scheme with FRI (Fast Reed-Solomon Interactive Oracle Proofs) and Reed-Solomon error correction.

## Overview

This library provides a complete implementation of a data availability sampling scheme that allows:
- **Polynomial Commitment**: Commit to data using FRI-based vector commitments
- **Reed-Solomon Encoding**: Encode data with error correction capabilities  
- **Data Availability Sampling**: Efficiently verify data availability by sampling random positions
- **Error Correction**: Reconstruct corrupted data using Reed-Solomon codes
- **KZG Comparison**: Optional KZG commitment benchmarks for comparison (via `kzg` feature)

## Features

- ✅ Multilinear Extension (MLE) conversion from raw bytes
- ✅ FRI-based polynomial commitment scheme
- ✅ Reed-Solomon encoding/decoding with configurable rates
- ✅ Merkle tree-based vector commitments
- ✅ Inclusion proof generation and verification
- ✅ Naive error correction reconstruction
- ✅ Data availability sampling with configurable sample sizes
- ✅ **Parallel processing** support (via `parallel` feature)
- ✅ **KZG commitment** comparison benchmarks (via `kzg` feature)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
frivail = { git = "https://github.com/availproject/binius-das-poc" }
```

### Feature Flags

- `parallel` - Enables parallel processing using Rayon for improved performance
- `kzg` - Enables KZG commitment benchmarks for comparison with FRI commitments

```toml
[dependencies]
frivail = { git = "https://github.com/availproject/binius-das-poc", features = ["parallel"] }
```

## Quick Start

```rust
use frivail::{
    frivail::{B128, FriVailDefault},
    poly::Utils,
    traits::{FriVailSampling, FriVailUtils},
};

// 1. Generate or load your data
let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();

// 2. Convert to Multilinear Extension (MLE)
let packed_mle = Utils::<B128>::new()
    .bytes_to_packed_mle(&data)
    .expect("Failed to create MLE");

// 3. Initialize FRI-Vail
let fri_vail = FriVailDefault::new(
    1,                              // log_inv_rate: Reed-Solomon inverse rate
    128,                            // num_test_queries: FRI security parameter
    4,                              // arity: FRI folding arity (typically 2-4)
    packed_mle.total_n_vars,        // n_vars: number of variables
    80,                             // log_num_shares: Merkle tree parameter
);

// 4. Setup FRI context
let (fri_params, ntt) = fri_vail
    .initialize_fri_context(packed_mle.packed_mle.log_len())
    .expect("Failed to initialize FRI context");

// 5. Generate commitment
let commit_output = fri_vail
    .commit(
        packed_mle.packed_mle.clone(),
        fri_params.clone(),
        &ntt,
    )
    .expect("Failed to commit");

println!("Commitment: {:?}", commit_output.commitment);
```

## Core Workflows

### 1. Encoding and Decoding

```rust
use frivail::{
    frivail::{B128, FriVailDefault},
    poly::Utils,
    traits::FriVailUtils,
};

let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
let packed_mle = Utils::<B128>::new()
    .bytes_to_packed_mle(&data)
    .expect("Failed to create MLE");

let fri_vail = FriVailDefault::new(1, 128, 4, packed_mle.total_n_vars, 80);
let (fri_params, ntt) = fri_vail
    .initialize_fri_context(packed_mle.packed_mle.log_len())
    .expect("Failed to initialize FRI context");

// Encode data
let encoded_codeword = fri_vail
    .encode_codeword(&packed_mle.packed_values, fri_params.clone(), &ntt)
    .expect("Failed to encode");

// Decode data
let decoded_codeword = fri_vail
    .decode_codeword(&encoded_codeword, fri_params.clone(), &ntt)
    .expect("Failed to decode");

assert_eq!(decoded_codeword, packed_mle.packed_values);
```

### 2. Error Correction

```rust
use frivail::{
    frivail::{B128, FriVailDefault},
    poly::Utils,
    traits::FriVailSampling,
};
use rand::{SeedableRng, rngs::StdRng, seq::index::sample};

let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
let packed_mle = Utils::<B128>::new()
    .bytes_to_packed_mle(&data)
    .expect("Failed to create MLE");

let fri_vail = FriVailDefault::new(1, 128, 4, packed_mle.total_n_vars, 80);
let (fri_params, ntt) = fri_vail
    .initialize_fri_context(packed_mle.packed_mle.log_len())
    .expect("Failed to initialize FRI context");

let encoded_codeword = fri_vail
    .encode_codeword(&packed_mle.packed_values, fri_params.clone(), &ntt)
    .expect("Failed to encode");

// Simulate data corruption
let mut corrupted = encoded_codeword.clone();
let corruption_rate = 0.1; // 10% corruption
let num_corrupted = (corrupted.len() as f64 * corruption_rate) as usize;

let mut rng = StdRng::seed_from_u64(42);
let corrupted_indices = sample(&mut rng, corrupted.len(), num_corrupted).into_vec();

for &idx in &corrupted_indices {
    corrupted[idx] = B128::zero();
}

// Reconstruct corrupted data
fri_vail
    .reconstruct_codeword_naive(&mut corrupted, &corrupted_indices)
    .expect("Failed to reconstruct");

assert_eq!(corrupted, encoded_codeword);
```

### 3. Data Availability Sampling

```rust
use frivail::{
    frivail::{B128, FriVailDefault},
    poly::Utils,
    traits::{FriVailSampling, FriVailUtils},
};
use rand::{SeedableRng, rngs::StdRng, seq::index::sample};

let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
let packed_mle = Utils::<B128>::new()
    .bytes_to_packed_mle(&data)
    .expect("Failed to create MLE");

let fri_vail = FriVailDefault::new(1, 128, 4, packed_mle.total_n_vars, 80);
let (fri_params, ntt) = fri_vail
    .initialize_fri_context(packed_mle.packed_mle.log_len())
    .expect("Failed to initialize FRI context");

let commit_output = fri_vail
    .commit(
        packed_mle.packed_mle.clone(),
        fri_params.clone(),
        &ntt,
    )
    .expect("Failed to commit");

// Sample random positions
let total_samples = commit_output.codeword.len();
let sample_size = total_samples / 2;
let indices = sample(&mut StdRng::from_seed([0; 32]), total_samples, sample_size).into_vec();

let commitment_bytes: [u8; 32] = commit_output
    .commitment
    .to_vec()
    .try_into()
    .expect("Commitment is 32 bytes");

// Verify each sample
for &sample_index in &indices {
    // Generate inclusion proof
    let mut inclusion_proof = fri_vail
        .inclusion_proof(&commit_output.committed, sample_index)
        .expect("Failed to generate proof");

    let value = commit_output.codeword[sample_index];

    // Verify inclusion proof
    fri_vail
        .verify_inclusion_proof(
            &mut inclusion_proof,
            &[value],
            sample_index,
            &fri_params,
            commitment_bytes,
        )
        .expect("Verification failed");
}
```

### 4. Proof Generation and Verification

```rust
use frivail::{
    frivail::{B128, FriVailDefault},
    poly::Utils,
    traits::{FriVailSampling, FriVailUtils},
};

let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
let packed_mle = Utils::<B128>::new()
    .bytes_to_packed_mle(&data)
    .expect("Failed to create MLE");

let fri_vail = FriVailDefault::new(1, 128, 4, packed_mle.total_n_vars, 80);
let (fri_params, ntt) = fri_vail
    .initialize_fri_context(packed_mle.packed_mle.log_len())
    .expect("Failed to initialize FRI context");

let commit_output = fri_vail
    .commit(
        packed_mle.packed_mle.clone(),
        fri_params.clone(),
        &ntt,
    )
    .expect("Failed to commit");

// Generate evaluation point
let evaluation_point = fri_vail
    .calculate_evaluation_point_random()
    .expect("Failed to generate evaluation point");

// Generate proof
let mut verifier_transcript = fri_vail
    .prove(
        packed_mle.packed_mle.clone(),
        fri_params.clone(),
        &ntt,
        &commit_output,
        &evaluation_point,
    )
    .expect("Failed to generate proof");

// Calculate evaluation claim
let evaluation_claim = fri_vail
    .calculate_evaluation_claim(&packed_mle.packed_values, &evaluation_point)
    .expect("Failed to calculate claim");

// Verify proof
fri_vail
    .verify(
        &mut verifier_transcript,
        evaluation_claim,
        &evaluation_point,
        &fri_params,
        &ntt,
        None,
        None,
        None,
        None,
    )
    .expect("Verification failed");
```

## Configuration Parameters

### Reed-Solomon Parameters
- **`log_inv_rate`**: Logarithm of inverse rate (e.g., 1 means 2x expansion)
- Higher values = more redundancy = better error correction

### FRI Parameters
- **`num_test_queries`**: Number of queries for FRI protocol (security parameter)
- Typical values: 64-128 for good security
- **`arity`**: Arity for FRI folding strategy (typically 2-4)
- Higher arity = fewer folding rounds but larger proof sizes

### Merkle Tree Parameters
- **`log_num_shares`**: Controls Merkle tree structure
- Affects commitment size and proof generation time

## Benchmarks

The library includes comprehensive benchmarks using Divan for:
- FRI commitment generation (4MB, 8MB, 16MB, 32MB)
- FRI proof generation (4MB, 8MB, 16MB, 32MB)
- KZG commitment comparison (when `kzg` feature enabled)
- Different redundancy factors (2x, 4x expansion)

Run benchmarks with:

```bash
# Basic benchmarks
cargo bench

# With parallel processing
cargo bench --features parallel

# With KZG comparison
cargo bench --features kzg

# All features
cargo bench --all-features
```

### Sample Benchmark Results

On a typical modern CPU, you can expect:
- **4MB FRI commitment**: ~50-100ms
- **16MB FRI commitment**: ~200-400ms  
- **32MB FRI commitment**: ~500-1000ms
- **Proof generation**: Similar timing to commitment

*Note: Actual performance varies based on hardware and configuration.*

## Running the Example

```bash
# Run the full demo
cargo run

# Run tests
cargo test

# Run specific test
cargo test test_codeword_decode
cargo test test_error_correction_reconstruction
cargo test test_data_availability_sampling
```

## Architecture

```
┌─────────────────┐
│   Raw Data      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Packed MLE     │  (Multilinear Extension)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  FRI Context    │  (NTT, Parameters)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Commitment     │  (Merkle Root)
└────────┬────────┘
         │
         ├──────────────────┐
         │                  │
         ▼                  ▼
┌─────────────────┐  ┌─────────────────┐
│  Encode/Decode  │  │  Sampling       │
└─────────────────┘  └─────────────────┘
         │                  │
         ▼                  ▼
┌─────────────────┐  ┌─────────────────┐
│  Reconstruction │  │  Verification   │
└─────────────────┘  └─────────────────┘
```

## Performance Characteristics

- **Commitment Time**: O(n log n) where n is data size
- **Encoding Time**: O(n log n) with NTT
- **Sampling Verification**: O(log n) per sample
- **Reconstruction**: O(k log n) where k is number of corrupted elements

## Testing

The library includes comprehensive tests:

- `test_frivail_new`: Basic initialization
- `test_field_conversion_methods`: Field arithmetic
- `test_calculate_evaluation_point_random`: Evaluation point generation
- `test_initialize_fri_context`: FRI context setup
- `test_commit_and_inclusion_proofs`: Commitment and proofs
- `test_codeword_decode`: Encoding/decoding cycle
- `test_error_correction_reconstruction`: Error correction
- `test_data_availability_sampling`: Full DAS workflow
- `test_calculate_evaluation_claim`: Evaluation claims
- `test_full_prove_verify_workflow`: End-to-end proving
- `test_invalid_verification_fails`: Negative testing
