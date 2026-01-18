//! A compression library implementing multiple compression algorithms.
//!
//! This library provides implementations of:
//! - RLE (Run-Length Encoding)
//! - LZ77 (Lempel-Ziv 77)
//! - Huffman coding
//!
//! # Example
//!
//! ```
//! use compression_lib::{Rle, Compressor, Decompressor};
//!
//! let rle = Rle::new();
//! let data = b"aaabbbccc";
//! let compressed = rle.compress(data).unwrap();
//! let decompressed = rle.decompress(&compressed).unwrap();
//! assert_eq!(decompressed, data);
//! ```

mod error;
mod huffman;
mod lz77;
mod rle;
mod traits;

pub use error::{CompressionError, Result};
pub use huffman::Huffman;
pub use lz77::Lz77;
pub use rle::Rle;
pub use traits::{Codec, Compressor, Decompressor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_export() {
        let rle = Rle::new();
        assert_eq!(Compressor::name(&rle), "RLE");
    }

    #[test]
    fn test_lz77_export() {
        let lz77 = Lz77::new();
        assert_eq!(Compressor::name(&lz77), "LZ77");
    }

    #[test]
    fn test_huffman_export() {
        let huffman = Huffman::new();
        assert_eq!(Compressor::name(&huffman), "Huffman");
    }

    #[test]
    fn test_compression_error_export() {
        let err = CompressionError::InvalidInput("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_traits_export() {
        fn accepts_compressor<T: Compressor>(_: &T) {}
        fn accepts_decompressor<T: Decompressor>(_: &T) {}
        fn accepts_codec<T: Codec>(_: &T) {}

        let rle = Rle::new();
        accepts_compressor(&rle);
        accepts_decompressor(&rle);
        accepts_codec(&rle);
    }

    #[test]
    fn test_all_codecs_roundtrip() {
        let data = b"hello world, this is a test of compression algorithms!";

        let rle = Rle::new();
        let compressed = rle.compress(data).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);

        let lz77 = Lz77::new();
        let compressed = lz77.compress(data).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);

        let huffman = Huffman::new();
        let compressed = huffman.compress(data).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data.as_slice());
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<Vec<u8>> {
            Ok(vec![1, 2, 3])
        }

        let result = returns_result();
        assert!(result.is_ok());
    }
}
