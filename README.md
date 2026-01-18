# compression_lib

A pure Rust compression library implementing multiple compression algorithms with a unified API.

## Features

- **Multiple Algorithms**: RLE, LZ77, and Huffman encoding
- **Unified API**: Common `Compressor` and `Decompressor` traits for all algorithms
- **Zero Unsafe Code**: Built with `#![forbid(unsafe_code)]`
- **No Dependencies**: Pure Rust implementation with no external runtime dependencies
- **Well Tested**: 99%+ test coverage with 111 unit tests

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
compression_lib = { path = "." }
```

## Quick Start

```rust
use compression_lib::{Rle, Lz77, Huffman, Compressor, Decompressor};

// Compress with RLE
let rle = Rle::new();
let data = b"aaaaaabbbbbcccc";
let compressed = rle.compress(data).unwrap();
let decompressed = rle.decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// Compress with LZ77
let lz77 = Lz77::new();
let compressed = lz77.compress(data).unwrap();
let decompressed = lz77.decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// Compress with Huffman
let huffman = Huffman::new();
let compressed = huffman.compress(data).unwrap();
let decompressed = huffman.decompress(&compressed).unwrap();
assert_eq!(decompressed, data);
```

## Algorithms

### RLE (Run-Length Encoding)

Run-Length Encoding replaces consecutive repeated bytes with a count and the byte value.

**Best for**: Data with long runs of repeated bytes (simple graphics, sparse data)

**Format**: Each run is encoded as `[count: u8][byte: u8]`

```rust
use compression_lib::{Rle, Compressor, Decompressor};

let rle = Rle::new();

// Highly compressible: repeated bytes
let data = vec![0xAA; 100];
let compressed = rle.compress(&data).unwrap();
assert!(compressed.len() < data.len()); // 2 bytes vs 100 bytes

// Roundtrip verification
let decompressed = rle.decompress(&compressed).unwrap();
assert_eq!(decompressed, data);
```

**Characteristics**:
- O(n) compression and decompression
- Maximum run length: 255 bytes
- Expansion possible for non-repeating data (2x worst case)

### LZ77 (Lempel-Ziv 77)

LZ77 uses a sliding window to find and reference repeated sequences in the data.

**Best for**: Text, source code, and data with repeated patterns

```rust
use compression_lib::{Lz77, Compressor, Decompressor};

// Default configuration
let lz77 = Lz77::new();

// Custom window size and lookahead buffer
let lz77_custom = Lz77::with_config(8192, 32);

let data = b"the quick brown fox jumps over the lazy dog. the quick brown fox";
let compressed = lz77.compress(data).unwrap();
let decompressed = lz77.decompress(&compressed).unwrap();
assert_eq!(decompressed, data.as_slice());
```

**Configuration**:
- `window_size`: Size of the search buffer (default: 4096)
- `lookahead_size`: Maximum match length (default: 18)

**Characteristics**:
- O(n * window_size) compression, O(n) decompression
- Good compression for repetitive data
- Includes 4-byte header for original length

### Huffman Encoding

Huffman coding assigns variable-length codes based on byte frequency, with shorter codes for more frequent bytes.

**Best for**: Data with skewed byte frequency distributions

```rust
use compression_lib::{Huffman, Compressor, Decompressor};

let huffman = Huffman::new();

// Text with repeated characters compresses well
let data = b"aaaaaaaaabbbbbcccdde";
let compressed = huffman.compress(data).unwrap();
let decompressed = huffman.decompress(&compressed).unwrap();
assert_eq!(decompressed, data.as_slice());
```

**Characteristics**:
- O(n log n) compression (tree building), O(n) decompression
- Optimal prefix-free encoding
- Includes serialized Huffman tree in output

## API Reference

### Traits

#### `Compressor`

```rust
pub trait Compressor {
    /// Compresses input bytes and returns compressed data.
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Returns the algorithm name.
    fn name(&self) -> &'static str;
}
```

#### `Decompressor`

```rust
pub trait Decompressor {
    /// Decompresses input bytes and returns original data.
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Returns the algorithm name.
    fn name(&self) -> &'static str;
}
```

#### `Codec`

A marker trait combining `Compressor + Decompressor`:

```rust
pub trait Codec: Compressor + Decompressor {}
```

### Error Handling

All operations return `Result<T, CompressionError>`:

```rust
use compression_lib::{Rle, Decompressor, CompressionError};

let rle = Rle::new();

// Invalid compressed data
let result = rle.decompress(&[1, 2, 3]); // Odd length is invalid for RLE
match result {
    Err(CompressionError::CorruptedData) => println!("Data is corrupted"),
    Err(e) => println!("Error: {}", e),
    Ok(_) => println!("Success"),
}
```

**Error Types**:
- `InvalidInput(String)` - Input data is invalid
- `DecompressionError(String)` - Decompression failed
- `BufferTooSmall` - Output buffer insufficient
- `InvalidHeader` - Compressed data has invalid header
- `CorruptedData` - Compressed data is corrupted

## Generic Programming

Use the `Codec` trait for algorithm-agnostic code:

```rust
use compression_lib::{Codec, Compressor, Decompressor, Rle, Lz77, Huffman, Result};

fn compress_with<C: Codec>(codec: &C, data: &[u8]) -> Result<Vec<u8>> {
    println!("Compressing with {}", Compressor::name(codec));
    let compressed = codec.compress(data)?;
    println!("Compressed {} -> {} bytes", data.len(), compressed.len());
    Ok(compressed)
}

fn roundtrip<C: Codec>(codec: &C, data: &[u8]) -> Result<bool> {
    let compressed = codec.compress(data)?;
    let decompressed = codec.decompress(&compressed)?;
    Ok(decompressed == data)
}

// Works with any codec
let rle = Rle::new();
let lz77 = Lz77::new();
let huffman = Huffman::new();

let data = b"test data";
assert!(roundtrip(&rle, data).unwrap());
assert!(roundtrip(&lz77, data).unwrap());
assert!(roundtrip(&huffman, data).unwrap());
```

## Choosing an Algorithm

| Algorithm | Best Use Case | Compression Ratio | Speed |
|-----------|--------------|-------------------|-------|
| **RLE** | Repeated bytes, simple graphics | Poor to Excellent* | Fastest |
| **LZ77** | Text, code, repeated patterns | Good | Medium |
| **Huffman** | Skewed byte distributions | Good | Medium |

*RLE compression ratio depends heavily on data characteristics. It excels with runs of repeated bytes but can expand random data.

**Recommendations**:
- **Simple data with runs**: Use RLE
- **Text and source code**: Use LZ77
- **Unknown data**: Try LZ77 or Huffman
- **Maximum compression**: Combine algorithms (e.g., LZ77 + Huffman)

## Building and Testing

```bash
# Build the library
cargo build

# Run tests
cargo test

# Run tests with coverage
cargo llvm-cov

# Run linter
cargo clippy

# Build documentation
cargo doc --open
```

## Project Structure

```
src/
├── lib.rs       # Public API and re-exports
├── error.rs     # Error types
├── traits.rs    # Compressor, Decompressor, Codec traits
├── rle.rs       # Run-Length Encoding
├── lz77.rs      # LZ77 compression
└── huffman.rs   # Huffman encoding
```

## Performance Notes

- All algorithms process data in a single pass where possible
- Memory usage is proportional to input size
- LZ77 window size can be tuned for memory/compression tradeoff
- Huffman builds a frequency table requiring a full scan of input

## Limitations

- Maximum input size: ~4GB (u32 length headers)
- RLE maximum run length: 255 bytes
- LZ77 maximum offset: 65535 bytes (u16)
- LZ77 maximum match length: 255 bytes (u8)
- No streaming API (full input required)

## License

MIT License

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. No clippy warnings: `cargo clippy`
3. Code is formatted: `cargo fmt`
4. New features include tests
5. Test coverage remains above 95%
