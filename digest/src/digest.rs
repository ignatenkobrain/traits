use super::{Input, FixedOutput};
use generic_array::GenericArray;
use generic_array::typenum::Unsigned;

/// The `Digest` trait specifies an interface common for digest functions.
///
/// It's a convenience wrapper around `Input`, `FixedOutput` and `Default`
/// traits. It also provides additional convenience methods.
pub trait Digest: Input + FixedOutput + Default {
    /// Create new hasher instance
    fn new() -> Self {
        Self::default()
    }

    /// Digest input data. This method can be called repeatedly
    /// for use with streaming messages.
    fn input(&mut self, buf: &[u8]) {
        self.process(buf);
    }

    /// Retrieve result and consume hasher instance
    fn result(self) -> GenericArray<u8, Self::OutputSize> {
        self.fixed_result()
    }

    /// Retrieve result and reset hasher instance
    fn result_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        self.fixed_result_reset()
    }

    /// Get output size of the hasher
    fn output_size() -> usize {
        Self::OutputSize::to_usize()
    }

    /// Convinience function to compute hash of the `data`. It will handle
    /// hasher creation, data feeding and finalization.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// println!("{:x}", sha2::Sha256::digest(b"Hello world"));
    /// ```
    #[inline]
    fn digest(data: &[u8]) -> GenericArray<u8, Self::OutputSize> {
        let mut hasher = Self::default();
        hasher.input(data);
        hasher.fixed_result()
    }
}

impl<D: Input + FixedOutput + Default> Digest for D {}
