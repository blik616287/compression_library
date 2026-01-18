use crate::error::Result;

/// Trait for compression algorithms.
pub trait Compressor {
    /// Compresses the input bytes and returns the compressed data.
    ///
    /// # Errors
    ///
    /// Returns `CompressionError` if compression fails due to invalid input
    /// or other algorithm-specific issues.
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Returns the name of this compression algorithm.
    fn name(&self) -> &'static str;
}

/// Trait for decompression algorithms.
pub trait Decompressor {
    /// Decompresses the input bytes and returns the original data.
    ///
    /// # Errors
    ///
    /// Returns `CompressionError` if decompression fails due to corrupted
    /// data, invalid format, or other algorithm-specific issues.
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Returns the name of this decompression algorithm.
    fn name(&self) -> &'static str;
}

/// Trait combining both compression and decompression capabilities.
pub trait Codec: Compressor + Decompressor {}

impl<T: Compressor + Decompressor> Codec for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CompressionError;

    struct MockCodec;

    impl Compressor for MockCodec {
        fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
            if input.is_empty() {
                return Err(CompressionError::InvalidInput("empty input".to_string()));
            }
            Ok(input.to_vec())
        }

        fn name(&self) -> &'static str {
            "MockCodec"
        }
    }

    impl Decompressor for MockCodec {
        fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
            if input.is_empty() {
                return Err(CompressionError::DecompressionError(
                    "empty input".to_string(),
                ));
            }
            Ok(input.to_vec())
        }

        fn name(&self) -> &'static str {
            "MockCodec"
        }
    }

    #[test]
    fn test_compressor_trait() {
        let codec = MockCodec;
        let input = b"test data";
        let compressed = codec.compress(input).unwrap();
        assert_eq!(compressed, input);
    }

    #[test]
    fn test_compressor_name() {
        let codec = MockCodec;
        assert_eq!(Compressor::name(&codec), "MockCodec");
    }

    #[test]
    fn test_decompressor_trait() {
        let codec = MockCodec;
        let input = b"test data";
        let decompressed = codec.decompress(input).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_codec_roundtrip() {
        let codec = MockCodec;
        let input = b"hello world";
        let compressed = codec.compress(input).unwrap();
        let decompressed = codec.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_compressor_empty_input() {
        let codec = MockCodec;
        let result = codec.compress(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompressor_empty_input() {
        let codec = MockCodec;
        let result = codec.decompress(&[]);
        assert!(result.is_err());
    }

    fn accepts_codec<T: Codec>(codec: &T, data: &[u8]) -> Result<Vec<u8>> {
        let compressed = codec.compress(data)?;
        codec.decompress(&compressed)
    }

    #[test]
    fn test_codec_trait_bound() {
        let codec = MockCodec;
        let result = accepts_codec(&codec, b"test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"test");
    }
}
