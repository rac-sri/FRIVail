use binius_field::field::FieldOps;
use binius_field::{ExtensionField, PackedField};
use binius_math::FieldBuffer;
use binius_verifier::config::B1;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::marker::PhantomData;

/// Number of bytes per field element (128 bits = 16 bytes)
const BYTES_PER_ELEMENT: usize = 16;
/// Number of bits per field element
const BITS_PER_ELEMENT: usize = 128;

/// Utility struct for converting bytes to packed multilinear extensions
///
/// Generic over packed field type `P` which must support extension field operations
pub struct Utils<P> {
    /// Phantom data to hold the packed field type parameter
    _p: PhantomData<P>,
}

/// Packed Multilinear Extension representation
///
/// Contains both the packed field buffer (for efficient operations) and
/// the unpacked scalar values (for verification and testing)
pub struct PackedMLE<P>
where
    P: PackedField + ExtensionField<B1>,
    P::Scalar: From<u128> + ExtensionField<B1>,
{
    /// Packed field buffer optimized for polynomial operations (scalar representation)
    pub packed_mle: FieldBuffer<P>,
    /// Unpacked scalar values for easier access and verification
    pub packed_values: Vec<P::Scalar>,
    /// Total number of variables in the multilinear extension
    /// This is the sum of packed variables and scalar bit width
    pub total_n_vars: usize,
}

impl<P> Utils<P>
where
    P: PackedField + ExtensionField<B1>,
    P::Scalar: From<u128> + ExtensionField<B1>,
{
    /// Create a new utility instance
    ///
    /// Initializes with the logarithm of the scalar field degree
    pub fn new() -> Self {
        Self { _p: PhantomData }
    }

    /// Convert a byte chunk to a field element
    fn bytes_to_scalar(&self, chunk: &[u8]) -> P::Scalar {
        let mut bytes_array = [0u8; BYTES_PER_ELEMENT];
        bytes_array[..chunk.len()].copy_from_slice(chunk);
        P::Scalar::from(u128::from_le_bytes(bytes_array))
    }

    /// Convert raw bytes to a packed multilinear extension
    ///
    /// # Process:
    /// 1. Split bytes into 16-byte chunks (128-bit field elements)
    /// 2. Convert each chunk to a field element via u128
    /// 3. Pad to next power of 2 for MLE structure
    /// 4. Create FieldBuffer for efficient polynomial operations
    ///
    /// # Arguments
    /// * `data` - Raw byte slice to convert
    ///
    /// # Returns
    /// * `Ok(PackedMLE)` - Successfully converted MLE
    /// * `Err(String)` - Conversion error message
    ///
    /// # Example
    /// ```ignore
    /// let data = vec![0u8; 1024];
    /// let utils = Utils::<B128>::new();
    /// let mle = utils.bytes_to_packed_mle(&data)?;
    /// ```
    pub fn bytes_to_packed_mle(&self, data: &[u8]) -> Result<PackedMLE<P>, String> {
        // Calculate number of field elements needed
        // Note: Using BITS_PER_ELEMENT here (not BYTES) to match the original logic
        let num_elements = data.len().div_ceil(BITS_PER_ELEMENT);

        // Pad to next power of 2 for MLE structure requirements
        let padded_size = num_elements.next_power_of_two();
        let big_field_n_vars = padded_size.ilog2() as usize;
        let packed_size = 1 << big_field_n_vars;

        // Convert bytes to field elements
        // Uses parallel processing if the "parallel" feature is enabled
        #[cfg(feature = "parallel")]
        let mut packed_values: Vec<P::Scalar> = {
            data.par_chunks(BYTES_PER_ELEMENT)
                .map(|chunk| self.bytes_to_scalar(chunk))
                .collect()
        };

        // Sequential version for non-parallel builds
        #[cfg(not(feature = "parallel"))]
        let mut packed_values: Vec<P::Scalar> = {
            let mut values = Vec::with_capacity(num_elements);
            for chunk in data.chunks(BYTES_PER_ELEMENT) {
                values.push(self.bytes_to_scalar(chunk));
            }
            values
        };

        // Pad with zeros to reach power-of-2 size
        packed_values.resize(packed_size, P::Scalar::zero());

        let packed_mle = FieldBuffer::<P>::from_values(packed_values.as_slice());
        let total_n_vars = packed_mle.log_len();

        Ok(PackedMLE::<P> {
            packed_mle,
            packed_values,
            total_n_vars,
        })
    }
}
