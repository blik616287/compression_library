use crate::error::{CompressionError, Result};
use crate::traits::{Compressor, Decompressor};

const MAX_RUN_LENGTH: u8 = 255;

#[derive(Debug, Default, Clone, Copy)]
pub struct Rle;

impl Rle {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Compressor for Rle {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(input.len());
        let mut i = 0;

        while i < input.len() {
            let current_byte = input[i];
            let mut run_length: u8 = 1;

            while i + usize::from(run_length) < input.len()
                && input[i + usize::from(run_length)] == current_byte
                && run_length < MAX_RUN_LENGTH
            {
                run_length += 1;
            }

            output.push(run_length);
            output.push(current_byte);
            i += usize::from(run_length);
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "RLE"
    }
}

impl Decompressor for Rle {
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        if !input.len().is_multiple_of(2) {
            return Err(CompressionError::CorruptedData);
        }

        let mut output = Vec::new();

        for chunk in input.chunks_exact(2) {
            let count = chunk[0];
            let byte = chunk[1];

            if count == 0 {
                return Err(CompressionError::CorruptedData);
            }

            output.extend(std::iter::repeat_n(byte, usize::from(count)));
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "RLE"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_new() {
        let rle = Rle::new();
        assert_eq!(Compressor::name(&rle), "RLE");
    }

    #[test]
    fn test_rle_default() {
        let rle = Rle::default();
        assert_eq!(Compressor::name(&rle), "RLE");
    }

    #[test]
    fn test_compress_empty() {
        let rle = Rle::new();
        let result = rle.compress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_decompress_empty() {
        let rle = Rle::new();
        let result = rle.decompress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compress_single_byte() {
        let rle = Rle::new();
        let result = rle.compress(&[0x42]).unwrap();
        assert_eq!(result, vec![1, 0x42]);
    }

    #[test]
    fn test_decompress_single_byte() {
        let rle = Rle::new();
        let result = rle.decompress(&[1, 0x42]).unwrap();
        assert_eq!(result, vec![0x42]);
    }

    #[test]
    fn test_compress_repeated_bytes() {
        let rle = Rle::new();
        let input = vec![0xAA; 5];
        let result = rle.compress(&input).unwrap();
        assert_eq!(result, vec![5, 0xAA]);
    }

    #[test]
    fn test_decompress_repeated_bytes() {
        let rle = Rle::new();
        let result = rle.decompress(&[5, 0xAA]).unwrap();
        assert_eq!(result, vec![0xAA; 5]);
    }

    #[test]
    fn test_compress_alternating_bytes() {
        let rle = Rle::new();
        let input = vec![0xAA, 0xBB, 0xAA, 0xBB];
        let result = rle.compress(&input).unwrap();
        assert_eq!(result, vec![1, 0xAA, 1, 0xBB, 1, 0xAA, 1, 0xBB]);
    }

    #[test]
    fn test_compress_mixed_runs() {
        let rle = Rle::new();
        let input = vec![0xAA, 0xAA, 0xAA, 0xBB, 0xCC, 0xCC];
        let result = rle.compress(&input).unwrap();
        assert_eq!(result, vec![3, 0xAA, 1, 0xBB, 2, 0xCC]);
    }

    #[test]
    fn test_roundtrip_simple() {
        let rle = Rle::new();
        let input = b"hello";
        let compressed = rle.compress(input).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_repeated() {
        let rle = Rle::new();
        let input = b"aaaaaabbbcccccccc";
        let compressed = rle.compress(input).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_all_same() {
        let rle = Rle::new();
        let input = vec![0xFF; 100];
        let compressed = rle.compress(&input).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_binary_data() {
        let rle = Rle::new();
        let input: Vec<u8> = (0..=255).collect();
        let compressed = rle.compress(&input).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_compress_max_run_length() {
        let rle = Rle::new();
        let input = vec![0xAA; 300];
        let compressed = rle.compress(&input).unwrap();
        assert_eq!(compressed[0], 255);
        assert_eq!(compressed[1], 0xAA);
        assert_eq!(compressed[2], 45);
        assert_eq!(compressed[3], 0xAA);
    }

    #[test]
    fn test_decompress_invalid_odd_length() {
        let rle = Rle::new();
        let result = rle.decompress(&[1, 2, 3]);
        assert!(matches!(result, Err(CompressionError::CorruptedData)));
    }

    #[test]
    fn test_decompress_zero_count() {
        let rle = Rle::new();
        let result = rle.decompress(&[0, 0xAA]);
        assert!(matches!(result, Err(CompressionError::CorruptedData)));
    }

    #[test]
    fn test_compression_ratio_repeated() {
        let rle = Rle::new();
        let input = vec![0xAA; 100];
        let compressed = rle.compress(&input).unwrap();
        assert!(compressed.len() < input.len());
    }

    #[test]
    fn test_compression_ratio_non_repeated() {
        let rle = Rle::new();
        let input: Vec<u8> = (0..100).collect();
        let compressed = rle.compress(&input).unwrap();
        assert!(compressed.len() >= input.len());
    }

    #[test]
    fn test_compressor_name() {
        let rle = Rle::new();
        assert_eq!(Compressor::name(&rle), "RLE");
    }

    #[test]
    fn test_decompressor_name() {
        let rle = Rle::new();
        assert_eq!(Decompressor::name(&rle), "RLE");
    }

    #[test]
    fn test_rle_clone() {
        let rle = Rle::new();
        let cloned = rle;
        assert_eq!(Compressor::name(&cloned), "RLE");
    }

    #[test]
    fn test_rle_debug() {
        let rle = Rle::new();
        let debug_str = format!("{rle:?}");
        assert!(debug_str.contains("Rle"));
    }

    #[test]
    fn test_roundtrip_zeros() {
        let rle = Rle::new();
        let input = vec![0u8; 50];
        let compressed = rle.compress(&input).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_max_values() {
        let rle = Rle::new();
        let input = vec![255u8; 50];
        let compressed = rle.compress(&input).unwrap();
        let decompressed = rle.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }
}
