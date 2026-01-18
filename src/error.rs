use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionError {
    InvalidInput(String),
    DecompressionError(String),
    BufferTooSmall,
    InvalidHeader,
    CorruptedData,
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::DecompressionError(msg) => write!(f, "Decompression error: {msg}"),
            Self::BufferTooSmall => write!(f, "Buffer too small for output"),
            Self::InvalidHeader => write!(f, "Invalid compression header"),
            Self::CorruptedData => write!(f, "Corrupted compressed data"),
        }
    }
}

impl std::error::Error for CompressionError {}

pub type Result<T> = std::result::Result<T, CompressionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_invalid_input() {
        let err = CompressionError::InvalidInput("test message".to_string());
        assert_eq!(err.to_string(), "Invalid input: test message");
    }

    #[test]
    fn test_error_display_decompression_error() {
        let err = CompressionError::DecompressionError("failed".to_string());
        assert_eq!(err.to_string(), "Decompression error: failed");
    }

    #[test]
    fn test_error_display_buffer_too_small() {
        let err = CompressionError::BufferTooSmall;
        assert_eq!(err.to_string(), "Buffer too small for output");
    }

    #[test]
    fn test_error_display_invalid_header() {
        let err = CompressionError::InvalidHeader;
        assert_eq!(err.to_string(), "Invalid compression header");
    }

    #[test]
    fn test_error_display_corrupted_data() {
        let err = CompressionError::CorruptedData;
        assert_eq!(err.to_string(), "Corrupted compressed data");
    }

    #[test]
    fn test_error_clone() {
        let err = CompressionError::InvalidInput("test".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_error_debug() {
        let err = CompressionError::BufferTooSmall;
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("BufferTooSmall"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(CompressionError::CorruptedData);
        assert!(result.is_err());
    }
}
