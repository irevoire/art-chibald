//! Node256: The largest node type is simply an array of 256
//! pointers and is used for storing between 49 and 256 entries.
//! With this representation, the next node can be found very
//! efficiently using a single lookup of the key byte in that array.
//! No additional indirection is necessary. If most entries are not
//! null, this representation is also very space efficient because
//! only pointers need to be stored.

use crate::Node;

#[derive(Debug)]
pub struct Node256 {
    values: [Option<Box<Node>>; 256],
}
