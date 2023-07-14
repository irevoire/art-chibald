use std::fmt;

/*
Additionally, at the front of each inner node, a header of
constant size (e.g., 16 bytes) stores the node type, the number
of children, and the compressed path (cf. Section III-E)
*/
#[derive(Default)]
pub struct Node {
    nb_childrens: u64,
    path: Vec<u8>,
    inner: InnerNode,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = match std::str::from_utf8(&self.path) {
            Ok(s) => format!("`{}` ({:?})", s, self.path),
            Err(_) => format!("{:?}", self.path),
        };
        f.debug_struct("Node")
            .field("nb_childrens", &self.nb_childrens)
            .field("path", &path)
            .field("inner", &self.inner)
            .finish()
    }
}

impl Node {
    fn insert(mut self, input: &[u8], value: u64) -> (Self, Option<u64>) {
        let Node {
            mut nb_childrens,
            mut path,
            mut inner,
        } = self;

        let old_value = match inner {
            InnerNode::Empty => {
                path = input.to_vec();
                inner = InnerNode::SingleValueLeaf(value);
                None
            }

            InnerNode::SingleValueLeaf(v) => {
                // is it the same value?
                if input == path {
                    inner = InnerNode::SingleValueLeaf(value);
                    Some(v)
                } else {
                    let common_path: Vec<u8> = input
                        .iter()
                        .zip(&path)
                        .take_while(|(a, b)| a == b)
                        .map(|(a, _b)| *a)
                        .collect();

                    let original_node_path = &path[common_path.len()..];
                    let new_path = &input[common_path.len()..];

                    let mut node4 = Node4::default();
                    node4.keys[0] = Some(original_node_path[0]);
                    node4.values[0] =
                        Some(Box::new(Node::default().insert(original_node_path, v).0));

                    node4.keys[1] = Some(new_path[0]);
                    node4.values[1] = Some(Box::new(Node::default().insert(new_path, value).0));

                    // patch ourselves
                    path = common_path;
                    inner = InnerNode::Node4(node4);
                    None
                }
                // (InnerNode::Node4(value), None)
            }

            InnerNode::Node4(node) => {
                let (node, old_value) = node.insert(input, value);
                inner = node;
                old_value
            }
            // InnerNode::Node16(node) => node.insert(input),
            // InnerNode::Node48(node) => node.insert(input),
            // InnerNode::Node256(node) => node.insert(input),
            _ => todo!(),
        };
        nb_childrens = self
            .nb_childrens
            .saturating_add(u64::from(old_value.is_none()));
        (
            Node {
                nb_childrens,
                path,
                inner,
            },
            old_value,
        )
    }
}

#[derive(Default, Debug)]
pub enum InnerNode {
    #[default]
    Empty,

    SingleValueLeaf(u64),

    Node4(Node4),
    Node16(Node16),
    Node48(Node48),
    Node256(Node256),
}

/*
Node4: The smallest node type can store up to 4 child
pointers and uses an array of length 4 for keys and another
array of the same length for pointers. The keys and pointers
are stored at corresponding positions and the keys are sorted.
*/
#[derive(Default)]
pub struct Node4 {
    keys: [Option<u8>; 4],
    values: [Option<Box<Node>>; 4],
}

impl Node4 {
    fn insert(mut self, input: &[u8], value: u64) -> (InnerNode, Option<u64>) {
        let byte = input[0];
        let pos = self.keys.iter().position(|opt| *opt == Some(byte));

        if let Some(pos) = pos {
            // safe because we found the position above
            /*
            let inserted = self.values[pos]
                .as_mut()
                .unwrap()
                .insert(&input[1..], value);
            */
            todo!()
            // (InnerNode::Node4(self), inserted)
        } else if let Some(pos) = self.keys.iter().position(|opt| opt.is_none()) {
            // create a leaf
            todo!()
        } else {
            // move to a Node16 and add a new leaf
            todo!()
        }
    }
}

impl fmt::Debug for Node4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = self
            .keys
            .iter()
            .map(|key| match key {
                Some(k) => format!("`{}`", *k as char),
                None => format!("___"),
            })
            .collect::<Vec<String>>();
        f.debug_struct("Node")
            .field("keys", &format!("{:?}", keys))
            .field("values", &format!("{:?}", self.values))
            .finish()
    }
}
/*
Node16: This node type is used for storing between 5 and
16 child pointers. Like the Node4, the keys and pointers
are stored in separate arrays at corresponding positions, but
both arrays have space for 16 entries. A key can be found
efficiently with binary search or, on modern hardware, with
parallel comparisons using SIMD instructions.
*/
#[derive(Debug)]
pub struct Node16 {
    keys: [Option<u8>; 16],
    values: [Option<Box<Node>>; 16],
}

impl Node16 {
    fn insert(&mut self, input: &[u8]) -> Option<u64> {
        todo!()
    }
}

/*
Node48: As the number of entries in a node increases,
searching the key array becomes expensive. Therefore, nodes
with more than 16 pointers do not store the keys explicitly.
Instead, a 256-element array is used, which can be indexed
with key bytes directly. If a node has between 17 and 48 child
pointers, this array stores indexes into a second array which
contains up to 48 pointers. This indirection saves space in
comparison to 256 pointers of 8 bytes, because the indexes
only require 6 bits (we use 1 byte for simplicity).
*/
#[derive(Debug)]
pub struct Node48 {
    keys: [Option<u8>; 256],
    values: [Option<Box<Node>>; 48],
}

impl Node48 {
    fn insert(&mut self, input: &[u8]) -> Option<u64> {
        todo!()
    }
}

/*
Node256: The largest node type is simply an array of 256
pointers and is used for storing between 49 and 256 entries.
With this representation, the next node can be found very
efficiently using a single lookup of the key byte in that array.
No additional indirection is necessary. If most entries are not
null, this representation is also very space efficient because
only pointers need to be stored.
*/
#[derive(Debug)]
pub struct Node256 {
    values: [Option<Box<Node>>; 256],
}

impl Node256 {
    fn insert(&mut self, input: &[u8]) -> Option<u64> {
        todo!()
    }
}

#[derive(Default, Debug)]
pub struct Art {
    root: Node,
}

impl Art {
    pub fn new() -> Art {
        Default::default()
    }

    /// Return `true` if the
    pub fn insert(&mut self, input: &[u8], value: u64) -> Option<u64> {
        let this = std::mem::take(&mut self.root);
        let old_value;
        (self.root, old_value) = this.insert(input, value);
        old_value
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_new() {
        let art = Art::new();
        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 0,
                path: "`` ([])",
                inner: Empty,
            },
        }
        "###);
    }

    #[test]
    fn single_insert() {
        let mut art = Art::new();
        let ret = art.insert(b"hello", 42);
        insta::assert_debug_snapshot!(ret, @"None");
        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 1,
                path: "`hello` ([104, 101, 108, 108, 111])",
                inner: SingleValueLeaf(
                    42,
                ),
            },
        }
        "###);
    }

    #[test]
    fn single_insert_and_replace() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        let ret = art.insert(b"hello", 43);
        insta::assert_debug_snapshot!(ret, @r###"
        Some(
            42,
        )
        "###);

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 1,
                path: "`hello` ([104, 101, 108, 108, 111])",
                inner: SingleValueLeaf(
                    43,
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_two_values_without_common_parts() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        let ret = art.insert(b"world", 43);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 2,
                path: "`` ([])",
                inner: Node4(
                    Node {
                        keys: "[\"`h`\", \"`w`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`hello` ([104, 101, 108, 108, 111])\", inner: SingleValueLeaf(42) }), Some(Node { nb_childrens: 1, path: \"`world` ([119, 111, 114, 108, 100])\", inner: SingleValueLeaf(43) }), None, None]",
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_two_values_with_common_parts() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        let ret = art.insert(b"hella", 43);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 2,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    Node {
                        keys: "[\"`o`\", \"`a`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), Some(Node { nb_childrens: 1, path: \"`a` ([97])\", inner: SingleValueLeaf(43) }), None, None]",
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_two_values_with_one_embedded_in_the_other() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        let ret = art.insert(b"hell", 43);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 2,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    Node {
                        keys: "[\"`o`\", \"`a`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), Some(Node { nb_childrens: 1, path: \"`a` ([97])\", inner: SingleValueLeaf(43) }), None, None]",
                    },
                ),
            },
        }
        "###);
    }
}
