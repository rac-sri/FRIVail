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
pub struct Utils<P> {
    _p: PhantomData<P>,
}

/// Packed Multilinear Extension representation
pub struct PackedMLE<P>
where
    P: PackedField + ExtensionField<B1>,
    P::Scalar: From<u128> + ExtensionField<B1>,
{
    pub packed_mle: FieldBuffer<P>,
    pub packed_values: Vec<P::Scalar>,
    pub total_n_vars: usize,
}

impl<P> Utils<P>
where
    P: PackedField + ExtensionField<B1>,
    P::Scalar: From<u128> + ExtensionField<B1>,
{
    /// Create a new utility instance
    ///
    /// # Returns
    /// New Utils instance
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
    /// # Arguments
    /// * `data` - Raw bytes to convert
    ///
    /// # Returns
    /// Packed multilinear extension representation
    ///
    /// # Errors
    /// When conversion fails
    pub fn bytes_to_packed_mle(&self, data: &[u8]) -> Result<PackedMLE<P>, String> {
        let num_elements = data.len().div_ceil(BITS_PER_ELEMENT);

        let padded_size = num_elements.next_power_of_two();
        let big_field_n_vars = padded_size.ilog2() as usize;
        let packed_size = 1 << big_field_n_vars;
        #[cfg(feature = "parallel")]
        let mut packed_values: Vec<P::Scalar> = {
            data.par_chunks(BYTES_PER_ELEMENT)
                .map(|chunk| self.bytes_to_scalar(chunk))
                .collect()
        };

        #[cfg(not(feature = "parallel"))]
        let mut packed_values: Vec<P::Scalar> = {
            let mut values = Vec::with_capacity(num_elements);
            for chunk in data.chunks(BYTES_PER_ELEMENT) {
                values.push(self.bytes_to_scalar(chunk));
            }
            values
        };

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
