//! Node4: The smallest node type can store up to 4 child
//! pointers and uses an array of length 4 for keys and another
//! array of the same length for pointers. The keys and pointers
//! are stored at corresponding positions and the keys are sorted.

use crate::{node16::Node16, Cell, Node};

#[derive(Default)]
pub(crate) struct Node4 {
    pub keys: [Cell; 4],
    pub values: [Option<Box<Node>>; 4],
}

impl std::fmt::Debug for Node4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys = self
            .keys
            .iter()
            .map(Cell::to_string)
            .collect::<Vec<String>>();
        f.debug_struct("Node")
            .field("keys", &format!("{:?}", keys))
            .field("values", &format!("{:?}", self.values))
            .finish()
    }
}

impl Node4 {
    pub fn insert(&mut self, start: usize, path: &[u8], cell: Cell, value: u64) {
        self.keys[start..].rotate_right(1);
        self.keys[start] = cell;
        self.values[start..].rotate_right(1);
        self.values[start] = Some(Box::new(Node::default().insert(path, value).0));
    }

    pub fn promote(self, start: usize, path: &[u8], cell: Cell, value: u64) -> Node16 {
        let mut new_node = Node16::from(self);
        new_node.insert(start, path, cell, value);
        new_node
    }
}
