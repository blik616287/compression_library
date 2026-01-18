use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use crate::error::{CompressionError, Result};
use crate::traits::{Compressor, Decompressor};

#[derive(Debug, Clone, Eq, PartialEq)]
struct HuffmanNode {
    frequency: usize,
    data: NodeData,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum NodeData {
    Leaf(u8),
    Internal {
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.frequency.cmp(&self.frequency)
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HuffmanNode {
    const fn new_leaf(byte: u8, frequency: usize) -> Self {
        Self {
            frequency,
            data: NodeData::Leaf(byte),
        }
    }

    fn new_internal(left: Self, right: Self) -> Self {
        let frequency = left.frequency + right.frequency;
        Self {
            frequency,
            data: NodeData::Internal {
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }

    fn build_codes(&self, prefix: Vec<bool>, codes: &mut HashMap<u8, Vec<bool>>) {
        match &self.data {
            NodeData::Leaf(byte) => {
                if prefix.is_empty() {
                    codes.insert(*byte, vec![false]);
                } else {
                    codes.insert(*byte, prefix);
                }
            }
            NodeData::Internal { left, right } => {
                let mut left_prefix = prefix.clone();
                left_prefix.push(false);
                left.build_codes(left_prefix, codes);

                let mut right_prefix = prefix;
                right_prefix.push(true);
                right.build_codes(right_prefix, codes);
            }
        }
    }
}

fn build_frequency_table(data: &[u8]) -> HashMap<u8, usize> {
    let mut freq = HashMap::new();
    for &byte in data {
        *freq.entry(byte).or_insert(0) += 1;
    }
    freq
}

fn build_huffman_tree(freq_table: &HashMap<u8, usize>) -> Option<HuffmanNode> {
    if freq_table.is_empty() {
        return None;
    }

    let mut heap: BinaryHeap<HuffmanNode> = freq_table
        .iter()
        .map(|(&byte, &freq)| HuffmanNode::new_leaf(byte, freq))
        .collect();

    while heap.len() > 1 {
        let left = heap.pop()?;
        let right = heap.pop()?;
        heap.push(HuffmanNode::new_internal(left, right));
    }

    heap.pop()
}

fn serialize_tree(node: &HuffmanNode, output: &mut Vec<u8>) {
    match &node.data {
        NodeData::Leaf(byte) => {
            output.push(1);
            output.push(*byte);
        }
        NodeData::Internal { left, right } => {
            output.push(0);
            serialize_tree(left, output);
            serialize_tree(right, output);
        }
    }
}

fn deserialize_tree(data: &[u8], pos: &mut usize) -> Result<HuffmanNode> {
    if *pos >= data.len() {
        return Err(CompressionError::CorruptedData);
    }

    let node_type = data[*pos];
    *pos += 1;

    if node_type == 1 {
        if *pos >= data.len() {
            return Err(CompressionError::CorruptedData);
        }
        let byte = data[*pos];
        *pos += 1;
        Ok(HuffmanNode::new_leaf(byte, 0))
    } else {
        let left = deserialize_tree(data, pos)?;
        let right = deserialize_tree(data, pos)?;
        Ok(HuffmanNode::new_internal(left, right))
    }
}

fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(bits.len().div_ceil(8));
    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (7 - i);
            }
        }
        bytes.push(byte);
    }
    bytes
}

fn bytes_to_bits(bytes: &[u8], num_bits: usize) -> Vec<bool> {
    let mut bits = Vec::with_capacity(num_bits);
    for &byte in bytes {
        for i in 0..8 {
            if bits.len() >= num_bits {
                break;
            }
            bits.push((byte >> (7 - i)) & 1 == 1);
        }
    }
    bits
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Huffman;

impl Huffman {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Compressor for Huffman {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let freq_table = build_frequency_table(input);
        let tree = build_huffman_tree(&freq_table)
            .ok_or_else(|| CompressionError::InvalidInput("cannot build tree".to_string()))?;

        let mut codes = HashMap::new();
        tree.build_codes(Vec::new(), &mut codes);

        let mut bits = Vec::new();
        for &byte in input {
            let code = codes.get(&byte).ok_or(CompressionError::CorruptedData)?;
            bits.extend(code);
        }

        let mut output = Vec::new();

        serialize_tree(&tree, &mut output);

        let original_len = u32::try_from(input.len()).unwrap_or(u32::MAX);
        output.extend_from_slice(&original_len.to_le_bytes());

        let num_bits = u32::try_from(bits.len()).unwrap_or(u32::MAX);
        output.extend_from_slice(&num_bits.to_le_bytes());

        let encoded_bytes = bits_to_bytes(&bits);
        output.extend_from_slice(&encoded_bytes);

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "Huffman"
    }
}

impl Decompressor for Huffman {
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut pos = 0;
        let tree = deserialize_tree(input, &mut pos)?;

        if pos + 8 > input.len() {
            return Err(CompressionError::CorruptedData);
        }

        let original_len = u32::from_le_bytes([
            input[pos],
            input[pos + 1],
            input[pos + 2],
            input[pos + 3],
        ]) as usize;
        pos += 4;

        let num_bits = u32::from_le_bytes([
            input[pos],
            input[pos + 1],
            input[pos + 2],
            input[pos + 3],
        ]) as usize;
        pos += 4;

        let encoded_bytes = &input[pos..];
        let bits = bytes_to_bits(encoded_bytes, num_bits);

        let mut output = Vec::with_capacity(original_len);
        let mut current_node = &tree;
        let mut bit_idx = 0;

        while output.len() < original_len && bit_idx < bits.len() {
            match &current_node.data {
                NodeData::Leaf(byte) => {
                    output.push(*byte);
                    current_node = &tree;
                }
                NodeData::Internal { left, right } => {
                    current_node = if bits[bit_idx] { right } else { left };
                    bit_idx += 1;
                }
            }
        }

        if let NodeData::Leaf(byte) = &current_node.data
            && output.len() < original_len
        {
            output.push(*byte);
        }

        if output.len() != original_len {
            return Err(CompressionError::CorruptedData);
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "Huffman"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_new() {
        let huffman = Huffman::new();
        assert_eq!(Compressor::name(&huffman), "Huffman");
    }

    #[test]
    fn test_huffman_default() {
        let huffman = Huffman::default();
        assert_eq!(Compressor::name(&huffman), "Huffman");
    }

    #[test]
    fn test_compress_empty() {
        let huffman = Huffman::new();
        let result = huffman.compress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_decompress_empty() {
        let huffman = Huffman::new();
        let result = huffman.decompress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_roundtrip_single_byte() {
        let huffman = Huffman::new();
        let input = &[0x42];
        let compressed = huffman.compress(input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_simple() {
        let huffman = Huffman::new();
        let input = b"hello";
        let compressed = huffman.compress(input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.as_slice());
    }

    #[test]
    fn test_roundtrip_repeated() {
        let huffman = Huffman::new();
        let input = b"aaaaaabbbbcccc";
        let compressed = huffman.compress(input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.as_slice());
    }

    #[test]
    fn test_roundtrip_all_same() {
        let huffman = Huffman::new();
        let input = vec![0xAA; 100];
        let compressed = huffman.compress(&input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_binary_data() {
        let huffman = Huffman::new();
        let input: Vec<u8> = (0..=255).collect();
        let compressed = huffman.compress(&input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_long_text() {
        let huffman = Huffman::new();
        let input = b"the quick brown fox jumps over the lazy dog";
        let compressed = huffman.compress(input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.as_slice());
    }

    #[test]
    fn test_compression_reduces_size_for_repeated() {
        let huffman = Huffman::new();
        let input = vec![0xAA; 1000];
        let compressed = huffman.compress(&input).unwrap();
        assert!(compressed.len() < input.len());
    }

    #[test]
    fn test_frequency_table() {
        let data = b"aabbc";
        let freq = build_frequency_table(data);
        assert_eq!(freq.get(&b'a'), Some(&2));
        assert_eq!(freq.get(&b'b'), Some(&2));
        assert_eq!(freq.get(&b'c'), Some(&1));
    }

    #[test]
    fn test_frequency_table_empty() {
        let freq = build_frequency_table(&[]);
        assert!(freq.is_empty());
    }

    #[test]
    fn test_build_huffman_tree_empty() {
        let freq = HashMap::new();
        let tree = build_huffman_tree(&freq);
        assert!(tree.is_none());
    }

    #[test]
    fn test_build_huffman_tree_single() {
        let mut freq = HashMap::new();
        freq.insert(b'a', 5);
        let tree = build_huffman_tree(&freq).unwrap();
        assert_eq!(tree.frequency, 5);
    }

    #[test]
    fn test_huffman_node_new_leaf() {
        let node = HuffmanNode::new_leaf(b'x', 10);
        assert_eq!(node.frequency, 10);
        assert!(matches!(node.data, NodeData::Leaf(b'x')));
    }

    #[test]
    fn test_huffman_node_new_internal() {
        let left = HuffmanNode::new_leaf(b'a', 5);
        let right = HuffmanNode::new_leaf(b'b', 3);
        let internal = HuffmanNode::new_internal(left, right);
        assert_eq!(internal.frequency, 8);
    }

    #[test]
    fn test_huffman_node_ordering() {
        let node1 = HuffmanNode::new_leaf(b'a', 10);
        let node2 = HuffmanNode::new_leaf(b'b', 5);
        assert!(node2 > node1);
    }

    #[test]
    fn test_bits_to_bytes() {
        let bits = vec![true, false, true, false, true, false, true, false];
        let bytes = bits_to_bytes(&bits);
        assert_eq!(bytes, vec![0b10101010]);
    }

    #[test]
    fn test_bits_to_bytes_partial() {
        let bits = vec![true, true, true];
        let bytes = bits_to_bytes(&bits);
        assert_eq!(bytes, vec![0b11100000]);
    }

    #[test]
    fn test_bytes_to_bits() {
        let bytes = vec![0b10101010];
        let bits = bytes_to_bits(&bytes, 8);
        assert_eq!(bits, vec![true, false, true, false, true, false, true, false]);
    }

    #[test]
    fn test_bytes_to_bits_partial() {
        let bytes = vec![0b11100000];
        let bits = bytes_to_bits(&bytes, 3);
        assert_eq!(bits, vec![true, true, true]);
    }

    #[test]
    fn test_serialize_deserialize_tree() {
        let left = HuffmanNode::new_leaf(b'a', 5);
        let right = HuffmanNode::new_leaf(b'b', 3);
        let tree = HuffmanNode::new_internal(left, right);

        let mut serialized = Vec::new();
        serialize_tree(&tree, &mut serialized);

        let mut pos = 0;
        let deserialized = deserialize_tree(&serialized, &mut pos).unwrap();

        assert_eq!(tree.frequency, 8);
        match deserialized.data {
            NodeData::Internal { left, right } => {
                assert!(matches!(left.data, NodeData::Leaf(b'a')));
                assert!(matches!(right.data, NodeData::Leaf(b'b')));
            }
            _ => panic!("Expected internal node"),
        }
    }

    #[test]
    fn test_deserialize_tree_corrupted() {
        let result = deserialize_tree(&[], &mut 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_tree_truncated_leaf() {
        let data = vec![1];
        let result = deserialize_tree(&data, &mut 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_corrupted_short() {
        let huffman = Huffman::new();
        let result = huffman.decompress(&[1, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_compressor_name() {
        let huffman = Huffman::new();
        assert_eq!(Compressor::name(&huffman), "Huffman");
    }

    #[test]
    fn test_decompressor_name() {
        let huffman = Huffman::new();
        assert_eq!(Decompressor::name(&huffman), "Huffman");
    }

    #[test]
    fn test_huffman_clone() {
        let huffman = Huffman::new();
        let cloned = huffman;
        assert_eq!(Compressor::name(&cloned), "Huffman");
    }

    #[test]
    fn test_huffman_debug() {
        let huffman = Huffman::new();
        let debug_str = format!("{huffman:?}");
        assert!(debug_str.contains("Huffman"));
    }

    #[test]
    fn test_roundtrip_zeros() {
        let huffman = Huffman::new();
        let input = vec![0u8; 50];
        let compressed = huffman.compress(&input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_max_values() {
        let huffman = Huffman::new();
        let input = vec![255u8; 50];
        let compressed = huffman.compress(&input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_alternating() {
        let huffman = Huffman::new();
        let input: Vec<u8> = (0..100).map(|i| if i % 2 == 0 { 0xAA } else { 0xBB }).collect();
        let compressed = huffman.compress(&input).unwrap();
        let decompressed = huffman.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_build_codes_single_symbol() {
        let node = HuffmanNode::new_leaf(b'x', 10);
        let mut codes = HashMap::new();
        node.build_codes(Vec::new(), &mut codes);
        assert!(codes.contains_key(&b'x'));
        assert!(!codes.get(&b'x').unwrap().is_empty());
    }

    #[test]
    fn test_node_partial_ord() {
        let node1 = HuffmanNode::new_leaf(b'a', 10);
        let node2 = HuffmanNode::new_leaf(b'b', 5);
        assert!(node1.partial_cmp(&node2).is_some());
    }
}
