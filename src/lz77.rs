use crate::error::{CompressionError, Result};
use crate::traits::{Compressor, Decompressor};

const DEFAULT_WINDOW_SIZE: usize = 4096;
const DEFAULT_LOOKAHEAD_SIZE: usize = 18;
const MIN_MATCH_LENGTH: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token {
    offset: u16,
    length: u8,
    next: u8,
}

impl Token {
    const fn new_literal(byte: u8) -> Self {
        Self {
            offset: 0,
            length: 0,
            next: byte,
        }
    }

    const fn new_match(offset: u16, length: u8, next: u8) -> Self {
        Self {
            offset,
            length,
            next,
        }
    }

    const fn to_bytes(self) -> [u8; 4] {
        let offset_bytes = self.offset.to_le_bytes();
        [offset_bytes[0], offset_bytes[1], self.length, self.next]
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }
        Some(Self {
            offset: u16::from_le_bytes([bytes[0], bytes[1]]),
            length: bytes[2],
            next: bytes[3],
        })
    }
}

#[derive(Debug, Clone)]
pub struct Lz77 {
    window_size: usize,
    lookahead_size: usize,
}

impl Default for Lz77 {
    fn default() -> Self {
        Self::new()
    }
}

impl Lz77 {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            window_size: DEFAULT_WINDOW_SIZE,
            lookahead_size: DEFAULT_LOOKAHEAD_SIZE,
        }
    }

    #[must_use]
    pub const fn with_config(window_size: usize, lookahead_size: usize) -> Self {
        Self {
            window_size,
            lookahead_size,
        }
    }

    #[must_use]
    pub const fn window_size(&self) -> usize {
        self.window_size
    }

    #[must_use]
    pub const fn lookahead_size(&self) -> usize {
        self.lookahead_size
    }

    fn find_longest_match(&self, data: &[u8], position: usize) -> (usize, usize) {
        let search_start = position.saturating_sub(self.window_size);
        let lookahead_end = (position + self.lookahead_size).min(data.len());

        let mut best_offset = 0;
        let mut best_length = 0;

        for start in search_start..position {
            let mut length = 0;
            while position + length < lookahead_end
                && data[start + length] == data[position + length]
                && length < self.lookahead_size
            {
                length += 1;
            }

            if length >= MIN_MATCH_LENGTH && length > best_length {
                best_offset = position - start;
                best_length = length;
            }
        }

        (best_offset, best_length)
    }
}

impl Compressor for Lz77 {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut tokens = Vec::new();
        let mut position = 0;

        while position < input.len() {
            let (offset, length) = self.find_longest_match(input, position);

            if length >= MIN_MATCH_LENGTH {
                let next_pos = position + length;
                let next_byte = if next_pos < input.len() {
                    input[next_pos]
                } else {
                    0
                };

                let token = Token::new_match(
                    u16::try_from(offset).unwrap_or(u16::MAX),
                    u8::try_from(length).unwrap_or(u8::MAX),
                    next_byte,
                );
                tokens.push(token);

                position = if next_pos < input.len() {
                    next_pos + 1
                } else {
                    next_pos
                };
            } else {
                let token = Token::new_literal(input[position]);
                tokens.push(token);
                position += 1;
            }
        }

        let original_len = u32::try_from(input.len()).unwrap_or(u32::MAX);
        let mut output = Vec::with_capacity(4 + tokens.len() * 4);
        output.extend_from_slice(&original_len.to_le_bytes());
        for token in tokens {
            output.extend_from_slice(&token.to_bytes());
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "LZ77"
    }
}

impl Decompressor for Lz77 {
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        if input.len() < 4 {
            return Err(CompressionError::CorruptedData);
        }

        let original_len = u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;
        let token_data = &input[4..];

        if !token_data.len().is_multiple_of(4) {
            return Err(CompressionError::CorruptedData);
        }

        let mut output = Vec::with_capacity(original_len);

        for chunk in token_data.chunks_exact(4) {
            let token =
                Token::from_bytes(chunk).ok_or(CompressionError::CorruptedData)?;

            if token.length != 0 {
                let offset = usize::from(token.offset);
                let length = usize::from(token.length);

                if offset == 0 || offset > output.len() {
                    return Err(CompressionError::CorruptedData);
                }

                let start = output.len() - offset;
                for i in 0..length {
                    if output.len() >= original_len {
                        break;
                    }
                    let byte = output[start + i];
                    output.push(byte);
                }
            }

            if output.len() < original_len {
                output.push(token.next);
            }
        }

        if output.len() != original_len {
            return Err(CompressionError::CorruptedData);
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "LZ77"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77_new() {
        let lz77 = Lz77::new();
        assert_eq!(lz77.window_size(), DEFAULT_WINDOW_SIZE);
        assert_eq!(lz77.lookahead_size(), DEFAULT_LOOKAHEAD_SIZE);
    }

    #[test]
    fn test_lz77_default() {
        let lz77 = Lz77::default();
        assert_eq!(lz77.window_size(), DEFAULT_WINDOW_SIZE);
    }

    #[test]
    fn test_lz77_with_config() {
        let lz77 = Lz77::with_config(1024, 32);
        assert_eq!(lz77.window_size(), 1024);
        assert_eq!(lz77.lookahead_size(), 32);
    }

    #[test]
    fn test_compress_empty() {
        let lz77 = Lz77::new();
        let result = lz77.compress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_decompress_empty() {
        let lz77 = Lz77::new();
        let result = lz77.decompress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compress_single_byte() {
        let lz77 = Lz77::new();
        let result = lz77.compress(&[0x42]).unwrap();
        assert_eq!(result.len(), 8); // 4 bytes header + 4 bytes token
    }

    #[test]
    fn test_roundtrip_simple() {
        let lz77 = Lz77::new();
        let input = b"hello";
        let compressed = lz77.compress(input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_repeated_pattern() {
        let lz77 = Lz77::new();
        let input = b"abcabcabcabc";
        let compressed = lz77.compress(input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_all_same() {
        let lz77 = Lz77::new();
        let input = vec![0xAA; 100];
        let compressed = lz77.compress(&input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_binary_data() {
        let lz77 = Lz77::new();
        let input: Vec<u8> = (0..=255).collect();
        let compressed = lz77.compress(&input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_long_repeated() {
        let lz77 = Lz77::new();
        let input = b"the quick brown fox jumps over the lazy dog. the quick brown fox jumps over the lazy dog.";
        let compressed = lz77.compress(input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.as_slice());
    }

    #[test]
    fn test_compression_reduces_size_for_repeated() {
        let lz77 = Lz77::new();
        // Use a large repeated pattern (over 200 chars) to ensure compression is beneficial
        let input = "abcdefghijklmnop".repeat(20);
        let compressed = lz77.compress(input.as_bytes()).unwrap();
        assert!(compressed.len() < input.len());
    }

    #[test]
    fn test_decompress_invalid_length() {
        let lz77 = Lz77::new();
        let result = lz77.decompress(&[1, 2, 3]);
        assert!(matches!(result, Err(CompressionError::CorruptedData)));
    }

    #[test]
    fn test_decompress_invalid_offset() {
        let lz77 = Lz77::new();
        let token = Token::new_match(100, 5, b'x');
        let token_bytes = token.to_bytes();
        let mut bytes = vec![1, 0, 0, 0]; // header: original length = 1
        bytes.extend_from_slice(&token_bytes);
        let result = lz77.decompress(&bytes);
        assert!(matches!(result, Err(CompressionError::CorruptedData)));
    }

    #[test]
    fn test_token_new_literal() {
        let token = Token::new_literal(b'a');
        assert_eq!(token.offset, 0);
        assert_eq!(token.length, 0);
        assert_eq!(token.next, b'a');
    }

    #[test]
    fn test_token_new_match() {
        let token = Token::new_match(10, 5, b'b');
        assert_eq!(token.offset, 10);
        assert_eq!(token.length, 5);
        assert_eq!(token.next, b'b');
    }

    #[test]
    fn test_token_roundtrip() {
        let token = Token::new_match(1000, 15, b'c');
        let bytes = token.to_bytes();
        let recovered = Token::from_bytes(&bytes).unwrap();
        assert_eq!(token, recovered);
    }

    #[test]
    fn test_token_from_bytes_too_short() {
        let result = Token::from_bytes(&[1, 2]);
        assert!(result.is_none());
    }

    #[test]
    fn test_compressor_name() {
        let lz77 = Lz77::new();
        assert_eq!(Compressor::name(&lz77), "LZ77");
    }

    #[test]
    fn test_decompressor_name() {
        let lz77 = Lz77::new();
        assert_eq!(Decompressor::name(&lz77), "LZ77");
    }

    #[test]
    fn test_lz77_clone() {
        let lz77 = Lz77::with_config(2048, 20);
        let cloned = lz77.clone();
        assert_eq!(lz77.window_size(), cloned.window_size());
        assert_eq!(lz77.lookahead_size(), cloned.lookahead_size());
    }

    #[test]
    fn test_lz77_debug() {
        let lz77 = Lz77::new();
        let debug_str = format!("{lz77:?}");
        assert!(debug_str.contains("Lz77"));
    }

    #[test]
    fn test_roundtrip_zeros() {
        let lz77 = Lz77::new();
        let input = vec![0u8; 50];
        let compressed = lz77.compress(&input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_alternating() {
        let lz77 = Lz77::new();
        let input: Vec<u8> = (0..100).map(|i| if i % 2 == 0 { 0xAA } else { 0xBB }).collect();
        let compressed = lz77.compress(&input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_small_window_size() {
        let lz77 = Lz77::with_config(16, 8);
        let input = b"abcdefghijklmnopabcdefghijklmnop";
        let compressed = lz77.compress(input).unwrap();
        let decompressed = lz77.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.as_slice());
    }

    #[test]
    fn test_find_longest_match_no_match() {
        let lz77 = Lz77::new();
        let data = b"abcdefgh";
        let (offset, length) = lz77.find_longest_match(data, 0);
        assert_eq!(offset, 0);
        assert_eq!(length, 0);
    }

    #[test]
    fn test_find_longest_match_with_match() {
        let lz77 = Lz77::new();
        let data = b"abcabc";
        let (offset, length) = lz77.find_longest_match(data, 3);
        assert_eq!(offset, 3);
        assert_eq!(length, 3);
    }

    #[test]
    fn test_decompress_zero_offset_with_length() {
        let lz77 = Lz77::new();
        let token = Token::new_match(0, 5, b'x');
        let token_bytes = token.to_bytes();
        let mut bytes = vec![1, 0, 0, 0]; // header: original length = 1
        bytes.extend_from_slice(&token_bytes);
        let result = lz77.decompress(&bytes);
        assert!(matches!(result, Err(CompressionError::CorruptedData)));
    }
}
