//! Node4: The smallest node type can store up to 4 child
//! pointers and uses an array of length 4 for keys and another
//! array of the same length for pointers. The keys and pointers
//! are stored at corresponding positions and the keys are sorted.

use crate::{node48::Node48, Cell, Node};

#[derive(Debug)]
pub(crate) struct StaticNode<const SIZE: usize> {
    pub keys: [Cell; SIZE],
    pub values: [Option<Box<Node>>; SIZE],
}

impl<const SIZE: usize> Default for StaticNode<SIZE> {
    fn default() -> Self {
        Self {
            keys: std::array::from_fn(|_| Cell::None),
            values: std::array::from_fn(|_| None),
        }
    }
}

pub(crate) type Node4 = StaticNode<4>;
pub(crate) type Node16 = StaticNode<16>;

impl From<StaticNode<4>> for StaticNode<16> {
    fn from(mut value: StaticNode<4>) -> Self {
        let mut keys: [Cell; 16] = Default::default();
        keys[..4].swap_with_slice(&mut value.keys);

        let mut values: [Option<Box<Node>>; 16] = Default::default();
        values[..4].swap_with_slice(&mut value.values);

        Self { keys, values }
    }
}

impl<const SIZE: usize> StaticNode<SIZE> {
    pub fn insert(&mut self, start: usize, path: &[u8], cell: Cell, value: u64) {
        self.keys[start..].rotate_right(1);
        self.keys[start] = cell;
        self.values[start..].rotate_right(1);
        self.values[start] = Some(Box::new(Node::default().insert(path, value).0));
    }
}

impl Node4 {
    pub fn promote(self, start: usize, path: &[u8], cell: Cell, value: u64) -> Node16 {
        let mut new_node = Node16::from(self);
        new_node.insert(start, path, cell, value);
        new_node
    }
}

impl Node16 {
    pub fn promote(self, start: usize, path: &[u8], cell: Cell, value: u64) -> Node48 {
        // let mut new_node = Node48::from(self);
        // new_node.insert(start, path, cell, value);
        // new_node
        todo!()
    }
}
