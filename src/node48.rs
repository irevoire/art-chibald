//! Node48: As the number of entries in a node increases,
//! searching the key array becomes expensive. Therefore, nodes
//! with more than 16 pointers do not store the keys explicitly.
//! Instead, a 256-element array is used, which can be indexed
//! with key bytes directly. If a node has between 17 and 48 child
//! pointers, this array stores indexes into a second array which
//! contains up to 48 pointers. This indirection saves space in
//! comparison to 256 pointers of 8 bytes, because the indexes
//! only require 6 bits (we use 1 byte for simplicity).

use crate::{static_node::Node16, Node};

#[derive(Debug)]
pub struct Node48 {
    keys: [Option<u8>; 256],
    values: [Option<Box<Node>>; 48],
}

impl From<Node16> for Node48 {
    fn from(value: Node16) -> Self {
        let mut keys: [Option<u8>; 256] = std::array::from_fn(|_| None);
        let mut values: [Option<Box<Node>>; 48] = std::array::from_fn(|_| None);

        value
            .keys
            .into_iter()
            .zip(value.values)
            .enumerate()
            .for_each(|(i, (key, value))| {
                keys[key.unwrap()] = Some(i as u8);
                values[i] = value;
            });

        Self { keys, values }
    }
}
