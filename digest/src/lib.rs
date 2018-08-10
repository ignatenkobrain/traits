//! This crate provides traits for describing funcionality of cryptographic hash
//! functions.
//!
//! By default std functionality in this crate disabled. (e.g. method for
//! hashing `Read`ers) To enable it turn on `std` feature in your `Cargo.toml`
//! for this crate.
#![no_std]
pub extern crate generic_array;

#[cfg(feature = "std")]
extern crate std;
use generic_array::{GenericArray, ArrayLength};

mod digest;
mod errors;
#[cfg(feature = "dev")]
pub mod dev;

pub use errors::InvalidOutputSize;
pub use digest::Digest;

/// Trait for processing input data
pub trait Input {
    /// Digest input data. This method can be called repeatedly, e.g. for
    /// processing streaming messages.
    fn process(&mut self, buf: &[u8]);
}

/// Trait to indicate that digest function processes data in blocks of size
/// `BlockSize`. Main usage of this trait is for implementing HMAC generically.
pub trait BlockInput {
    type BlockSize: ArrayLength<u8>;
}

/// Trait for returning digest result with the fixed size
pub trait FixedOutput: Default {
    type OutputSize: ArrayLength<u8>;

    /// Retrieve result and consume hasher instance.
    fn fixed_result(self) -> GenericArray<u8, Self::OutputSize>;

    /// Retrieve result and reset hasher instance.
    ///
    /// Some implementations may provide more optmized implementations of this
    /// method compared to the default one.
    fn fixed_result_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        let mut hasher = Default::default();
        core::mem::swap(self, &mut hasher);
        hasher.fixed_result()
    }
}

/// Trait for returning digest result with the varaible size
pub trait VariableOutput: core::marker::Sized + Default {
    /// Create new hasher instance with given output size. Will return
    /// `Err(InvalidOutputSize)` in case if hasher can not work with the given
    /// output size. Will always return an error if output size equals to zero.
    fn new(output_size: usize) -> Result<Self, InvalidOutputSize>;

    /// Get output size of the hasher instance provided to the `new` method
    fn output_size(&self) -> usize;

    /// Retrieve result via closure and consume hasher.
    ///
    /// Closure is guaranteed to be called, length of the buffer passed to it
    /// will be equal to `output_size`.
    fn variable_result<F: FnOnce(&[u8])>(self, f: F);

    /// Retrieve result via closure and reset hasher.
    ///
    /// Closure is guaranteed to be called, length of the buffer passed to it
    /// will be equal to `output_size`.
    fn variable_result_reset<F: FnOnce(&[u8])>(&mut self, f: F) {
        let mut hasher = Default::default();
        core::mem::swap(self, &mut hasher);
        hasher.variable_result(f);
    }

    /// Retrieve result into vector and consume hasher instance.
    #[cfg(feature = "std")]
    fn vec_result(self, buffer: &mut [u8]) -> Vec<u8> {
        use std::vec::Vec;
        let mut buf = Vec::with_capacity(self.output_size());
        self.variable_result(|res| buf.extend_with(res));
    }
}

/// Trait for decribing readers which are used to extract extendable output
/// from the resulting state of hash function.
pub trait XofReader {
    /// Read output into the `buffer`. Can be called unlimited number of times.
    fn read(&mut self, buffer: &mut [u8]);
}

/// Trait which describes extendable output (XOF) of hash functions. Using this
/// trait you first need to get structure which implements `XofReader`, using
/// which you can read extendable output.
pub trait ExtendableOutput {
    type Reader: XofReader;

    /// Retrieve XOF reader and reset hasher instance.
    fn xof_result(&mut self) -> Self::Reader;
}

#[macro_export]
/// Implements `std::io::Write` trait for implementator of `Input`
macro_rules! impl_write {
    ($hasher:ident) => {
        #[cfg(feature = "std")]
        impl ::std::io::Write for $hasher {
            fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
                self.process(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> ::std::io::Result<()> {
                Ok(())
            }
        }
    }
}
