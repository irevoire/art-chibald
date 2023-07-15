use std::{
    fmt,
    mem::{swap, take},
};

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
        let old_value = match self.inner {
            InnerNode::Empty => {
                self.path = input.to_vec();
                self.inner = InnerNode::SingleValueLeaf(value);
                None
            }

            InnerNode::SingleValueLeaf(v) => {
                // is it the same value?
                if input == self.path {
                    self.inner = InnerNode::SingleValueLeaf(value);
                    Some(v)
                } else {
                    let common_path: Vec<u8> = input
                        .iter()
                        .zip(&self.path)
                        .take_while(|(a, b)| a == b)
                        .map(|(a, _b)| *a)
                        .collect();

                    let original_node_path = &self.path[common_path.len()..];
                    let new_path = &input[common_path.len()..];

                    let mut node4 = Node4::default();
                    let is_original_before_new = original_node_path > new_path;
                    let original_node_pos = is_original_before_new as usize;
                    let new_node_pos = !is_original_before_new as usize;
                    if !original_node_path.is_empty() {
                        node4.keys[original_node_pos] = EOption::Some(original_node_path[0]);
                    } else {
                        node4.keys[original_node_pos] = EOption::End;
                    }
                    node4.values[original_node_pos] =
                        Some(Box::new(Node::default().insert(original_node_path, v).0));

                    if !new_path.is_empty() {
                        node4.keys[new_node_pos] = EOption::Some(new_path[0]);
                    } else {
                        node4.keys[new_node_pos] = EOption::End;
                    }
                    node4.values[new_node_pos] =
                        Some(Box::new(Node::default().insert(new_path, value).0));
                    // patch ourselves
                    self.path = common_path;
                    self.inner = InnerNode::Node4(node4);
                    None
                }
                // (InnerNode::Node4(value), None)
            }

            InnerNode::Node4(ref mut node) => match input.strip_prefix(self.path.as_slice()) {
                Some([]) => {
                    if node.keys[0] == EOption::End {
                        let mut value_node = node.values[0].take().unwrap();
                        let value_node_v = value_node.inner.unwrap_leaf();
                        let old_value = *value_node_v;
                        *value_node_v = value;
                        node.values[0] = Some(value_node);
                        Some(old_value)
                    } else {
                        if self.nb_childrens == 4 {
                            let mut new_node = Node16::from(take(node));
                            new_node.keys.rotate_right(1);
                            new_node.keys[0] = EOption::End;
                            new_node.values.rotate_right(1);
                            new_node.values[0] =
                                Some(Box::new(Node::default().insert(&[], value).0));
                            self.inner = InnerNode::Node16(new_node);
                        } else {
                            node.keys.rotate_right(1);
                            node.keys[0] = EOption::End;
                            node.values.rotate_right(1);
                            node.values[0] = Some(Box::new(Node::default().insert(&[], value).0));
                        }
                        None
                    }
                }
                Some(s) => {
                    if let Some(pos) = node
                        .keys
                        .iter()
                        .position(|k| matches!(k, EOption::Some(b) if *b == s[0]))
                    {
                        let (new_node, old_value) =
                            node.values[pos].take().unwrap().insert(s, value);
                        node.values[pos] = Some(Box::new(new_node));
                        old_value
                    } else {
                        if self.nb_childrens == 4 {
                            let mut new_node = Node16::from(take(node));
                            let pos = new_node
                                .keys
                                .iter()
                                .position(|k| {
                                    *k == EOption::None
                                        || matches!(k, EOption::Some(b) if *b > s[0])
                                })
                                .unwrap();
                            new_node.keys[pos..].rotate_right(1);
                            new_node.keys[pos] = EOption::Some(s[0]);
                            new_node.values[pos..].rotate_right(1);
                            new_node.values[pos] =
                                Some(Box::new(Node::default().insert(s, value).0));
                            self.inner = InnerNode::Node16(new_node);
                        } else {
                            let pos = node
                                .keys
                                .iter()
                                .position(|k| {
                                    *k == EOption::None
                                        || matches!(k, EOption::Some(b) if *b > s[0])
                                })
                                .unwrap();
                            node.keys[pos..].rotate_right(1);
                            node.keys[pos] = EOption::Some(s[0]);
                            node.values[pos..].rotate_right(1);
                            node.values[pos] = Some(Box::new(Node::default().insert(s, value).0));
                        }
                        None
                    }
                }
                None => {
                    let common_path: Vec<u8> = input
                        .iter()
                        .zip(&self.path)
                        .take_while(|(a, b)| a == b)
                        .map(|(a, _b)| *a)
                        .collect();

                    let original_node_path = &self.path[common_path.len()..];
                    let new_path = &input[common_path.len()..];

                    let mut node4 = Node4::default();
                    let is_original_before_new = original_node_path > new_path;
                    let original_node_pos = is_original_before_new as usize;
                    let new_node_pos = !is_original_before_new as usize;
                    if !original_node_path.is_empty() {
                        node4.keys[original_node_pos] = EOption::Some(original_node_path[0]);
                    } else {
                        node4.keys[original_node_pos] = EOption::End;
                    }
                    self.path = original_node_path.to_vec();
                    node4.values[original_node_pos] = Some(Box::new(self));

                    if !new_path.is_empty() {
                        node4.keys[new_node_pos] = EOption::Some(new_path[0]);
                    } else {
                        node4.keys[new_node_pos] = EOption::End;
                    }
                    node4.values[new_node_pos] =
                        Some(Box::new(Node::default().insert(new_path, value).0));
                    // patch ourselves
                    self = Default::default();
                    self.path = common_path;
                    self.inner = InnerNode::Node4(node4);

                    None
                }
            },
            // InnerNode::Node16(node) => node.insert(input),
            // InnerNode::Node48(node) => node.insert(input),
            // InnerNode::Node256(node) => node.insert(input),
            _ => todo!(),
        };
        self.nb_childrens = self
            .nb_childrens
            .saturating_add(u64::from(old_value.is_none()));
        (self, old_value)
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

impl InnerNode {
    #[track_caller]
    pub fn unwrap_leaf(&mut self) -> &mut u64 {
        if let Self::SingleValueLeaf(ref mut value) = self {
            value
        } else {
            panic!("Unwrapped a non-leaf in `unwrap_leaf`")
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub enum EOption<T> {
    End,
    #[default]
    None,
    Some(T),
}

impl<T> EOption<T> {
    pub fn is_none(&self) -> bool {
        matches!(self, EOption::None)
    }
}

/*
Node4: The smallest node type can store up to 4 child
pointers and uses an array of length 4 for keys and another
array of the same length for pointers. The keys and pointers
are stored at corresponding positions and the keys are sorted.
*/
#[derive(Default)]
pub struct Node4 {
    keys: [EOption<u8>; 4],
    values: [Option<Box<Node>>; 4],
}

impl fmt::Debug for Node4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = self
            .keys
            .iter()
            .map(|key| match key {
                EOption::Some(k) => format!("`{}`", *k as char),
                EOption::None => "___".to_string(),
                EOption::End => "END".to_string(),
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
    keys: [EOption<u8>; 16],
    values: [Option<Box<Node>>; 16],
}

impl From<Node4> for Node16 {
    fn from(mut value: Node4) -> Self {
        let mut keys: [EOption<u8>; 16] = Default::default();
        keys.iter_mut()
            .zip(value.keys.iter_mut())
            .for_each(|(l, r)| swap(l, r));
        let mut values: [Option<Box<Node>>; 16] = Default::default();
        values
            .iter_mut()
            .zip(value.values.iter_mut())
            .for_each(|(l, r)| swap(l, r));
        Self { keys, values }
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
                        keys: "[\"`a`\", \"`o`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`a` ([97])\", inner: SingleValueLeaf(43) }), Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), None, None]",
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
                        keys: "[\"END\", \"`o`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`` ([])\", inner: SingleValueLeaf(43) }), Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), None, None]",
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn reinsert_a_value_in_node4() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        art.insert(b"hell", 43);
        let ret = art.insert(b"hell", 44);
        insta::assert_debug_snapshot!(ret, @r###"
        Some(
            43,
        )
        "###);

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 2,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    Node {
                        keys: "[\"END\", \"`o`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`` ([])\", inner: SingleValueLeaf(44) }), Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), None, None]",
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_a_prefix_value_in_node4() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        art.insert(b"hella", 43);
        let ret = art.insert(b"hell", 44);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 3,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    Node {
                        keys: "[\"END\", \"`a`\", \"`o`\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`` ([])\", inner: SingleValueLeaf(44) }), Some(Node { nb_childrens: 1, path: \"`a` ([97])\", inner: SingleValueLeaf(43) }), Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), None]",
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_a_prefix_value_in_a_full_node4() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        art.insert(b"hella", 43);
        art.insert(b"helli", 44);
        art.insert(b"hellu", 45);
        let ret = art.insert(b"hell", 46);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 5,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node16(
                    Node16 {
                        keys: [
                            End,
                            Some(
                                97,
                            ),
                            Some(
                                105,
                            ),
                            Some(
                                111,
                            ),
                            Some(
                                117,
                            ),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`` ([])",
                                    inner: SingleValueLeaf(
                                        46,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`a` ([97])",
                                    inner: SingleValueLeaf(
                                        43,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`i` ([105])",
                                    inner: SingleValueLeaf(
                                        44,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`o` ([111])",
                                    inner: SingleValueLeaf(
                                        42,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`u` ([117])",
                                    inner: SingleValueLeaf(
                                        45,
                                    ),
                                },
                            ),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ],
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_a_value_in_a_full_node4() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        art.insert(b"hella", 43);
        art.insert(b"helli", 44);
        art.insert(b"hellu", 45);
        let ret = art.insert(b"hellyolo", 46);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 5,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node16(
                    Node16 {
                        keys: [
                            Some(
                                97,
                            ),
                            Some(
                                105,
                            ),
                            Some(
                                111,
                            ),
                            Some(
                                117,
                            ),
                            Some(
                                121,
                            ),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`a` ([97])",
                                    inner: SingleValueLeaf(
                                        43,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`i` ([105])",
                                    inner: SingleValueLeaf(
                                        44,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`o` ([111])",
                                    inner: SingleValueLeaf(
                                        42,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`u` ([117])",
                                    inner: SingleValueLeaf(
                                        45,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`yolo` ([121, 111, 108, 111])",
                                    inner: SingleValueLeaf(
                                        46,
                                    ),
                                },
                            ),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ],
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_several_values() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        art.insert(b"hella", 43);
        let ret = art.insert(b"helli", 44);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 3,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    Node {
                        keys: "[\"`a`\", \"`i`\", \"`o`\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 1, path: \"`a` ([97])\", inner: SingleValueLeaf(43) }), Some(Node { nb_childrens: 1, path: \"`i` ([105])\", inner: SingleValueLeaf(44) }), Some(Node { nb_childrens: 1, path: \"`o` ([111])\", inner: SingleValueLeaf(42) }), None]",
                    },
                ),
            },
        }
        "###);
    }

    #[test]
    fn insert_several_values_with_mismatched_prefix() {
        let mut art = Art::new();
        art.insert(b"hello", 42);
        art.insert(b"hella", 43);
        let ret = art.insert(b"hey", 44);
        insta::assert_debug_snapshot!(ret, @"None");

        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 1,
                path: "`he` ([104, 101])",
                inner: Node4(
                    Node {
                        keys: "[\"`l`\", \"`y`\", \"___\", \"___\"]",
                        values: "[Some(Node { nb_childrens: 2, path: \"`ll` ([108, 108])\", inner: Node4(Node { keys: \"[\\\"`a`\\\", \\\"`o`\\\", \\\"___\\\", \\\"___\\\"]\", values: \"[Some(Node { nb_childrens: 1, path: \\\"`a` ([97])\\\", inner: SingleValueLeaf(43) }), Some(Node { nb_childrens: 1, path: \\\"`o` ([111])\\\", inner: SingleValueLeaf(42) }), None, None]\" }) }), Some(Node { nb_childrens: 1, path: \"`y` ([121])\", inner: SingleValueLeaf(44) }), None, None]",
                    },
                ),
            },
        }
        "###);
    }
}
