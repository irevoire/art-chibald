//! Node16: This node type is used for storing between 5 and
//! 16 child pointers. Like the Node4, the keys and pointers
//! are stored in separate arrays at corresponding positions, but
//! both arrays have space for 16 entries. A key can be found
//! efficiently with binary search or, on modern hardware, with
//! parallel comparisons using SIMD instructions.

use crate::{node4::Node4, Cell, Node};

#[derive(Debug)]
pub(crate) struct Node16 {
    keys: [Cell; 16],
    values: [Option<Box<Node>>; 16],
}

impl From<Node4> for Node16 {
    fn from(mut value: Node4) -> Self {
        let mut keys: [Cell; 16] = Default::default();
        keys[..4].swap_with_slice(&mut value.keys);

        let mut values: [Option<Box<Node>>; 16] = Default::default();
        values[..4].swap_with_slice(&mut value.values);

        Self { keys, values }
    }
}

impl Node16 {
    pub fn insert(&mut self, start: usize, path: &[u8], cell: Cell, value: u64) {
        self.keys[start..].rotate_right(1);
        self.keys[start] = cell;
        self.values[start..].rotate_right(1);
        self.values[start] = Some(Box::new(Node::default().insert(path, value).0));
    }
}
