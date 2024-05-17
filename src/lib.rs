use std::{
    fmt::{self},
    mem::take,
};

use node256::Node256;
use node48::Node48;
use static_node::{Node16, Node4};

// mod node16;
mod node256;
// mod node4;
mod node48;
mod static_node;

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
                        node4.keys[original_node_pos] = Cell::Some(original_node_path[0]);
                    } else {
                        node4.keys[original_node_pos] = Cell::End;
                    }
                    node4.values[original_node_pos] =
                        Some(Box::new(Node::default().insert(original_node_path, v).0));

                    if !new_path.is_empty() {
                        node4.keys[new_node_pos] = Cell::Some(new_path[0]);
                    } else {
                        node4.keys[new_node_pos] = Cell::End;
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
                    if node.keys[0] == Cell::End {
                        let mut value_node = node.values[0].take().unwrap();
                        let value_node_v = value_node.inner.unwrap_leaf();
                        let old_value = *value_node_v;
                        *value_node_v = value;
                        node.values[0] = Some(value_node);
                        Some(old_value)
                    } else {
                        if self.nb_childrens == 4 {
                            let new_node = take(node).promote(0, &[], Cell::End, value);
                            self.inner = InnerNode::Node16(new_node);
                        } else {
                            node.insert(0, &[], Cell::End, value);
                        }
                        None
                    }
                }
                Some(s) => {
                    if let Some(pos) = node
                        .keys
                        .iter()
                        .position(|k| matches!(k, Cell::Some(b) if *b == s[0]))
                    {
                        let (new_node, old_value) =
                            node.values[pos].take().unwrap().insert(s, value);
                        node.values[pos] = Some(Box::new(new_node));
                        old_value
                    } else {
                        if self.nb_childrens == 4 {
                            let pos = node
                                .keys
                                .iter()
                                .position(|k| matches!(k, Cell::Some(b) if *b > s[0]))
                                .unwrap_or(4);

                            let new_node = take(node).promote(pos, s, Cell::Some(s[0]), value);
                            self.inner = InnerNode::Node16(new_node);
                        } else {
                            let pos = node
                                .keys
                                .iter()
                                .position(|k| {
                                    *k == Cell::None || matches!(k, Cell::Some(b) if *b > s[0])
                                })
                                .unwrap();
                            node.insert(pos, s, Cell::Some(s[0]), value);
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
                        node4.keys[original_node_pos] = Cell::Some(original_node_path[0]);
                    } else {
                        node4.keys[original_node_pos] = Cell::End;
                    }
                    self.path = original_node_path.to_vec();
                    node4.values[original_node_pos] = Some(Box::new(take(&mut self)));

                    if !new_path.is_empty() {
                        node4.keys[new_node_pos] = Cell::Some(new_path[0]);
                    } else {
                        node4.keys[new_node_pos] = Cell::End;
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
            InnerNode::Node16(ref mut node) => match input.strip_prefix(self.path.as_slice()) {
                Some([]) => {
                    if node.keys[0] == Cell::End {
                        let mut value_node = node.values[0].take().unwrap();
                        let value_node_v = value_node.inner.unwrap_leaf();
                        let old_value = *value_node_v;
                        *value_node_v = value;
                        node.values[0] = Some(value_node);
                        Some(old_value)
                    } else {
                        if self.nb_childrens == 4 {
                            let new_node = take(node).promote(0, &[], Cell::End, value);
                            self.inner = InnerNode::Node48(new_node);
                        } else {
                            node.insert(0, &[], Cell::End, value);
                        }
                        None
                    }
                }
                Some(s) => {
                    if let Some(pos) = node
                        .keys
                        .iter()
                        .position(|k| matches!(k, Cell::Some(b) if *b == s[0]))
                    {
                        let (new_node, old_value) =
                            node.values[pos].take().unwrap().insert(s, value);
                        node.values[pos] = Some(Box::new(new_node));
                        old_value
                    } else {
                        if self.nb_childrens == 4 {
                            let pos = node
                                .keys
                                .iter()
                                .position(|k| matches!(k, Cell::Some(b) if *b > s[0]))
                                .unwrap_or(4);

                            let new_node = take(node).promote(pos, s, Cell::Some(s[0]), value);
                            self.inner = InnerNode::Node48(new_node);
                        } else {
                            let pos = node
                                .keys
                                .iter()
                                .position(|k| {
                                    *k == Cell::None || matches!(k, Cell::Some(b) if *b > s[0])
                                })
                                .unwrap();
                            node.insert(pos, s, Cell::Some(s[0]), value);
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
                        node4.keys[original_node_pos] = Cell::Some(original_node_path[0]);
                    } else {
                        node4.keys[original_node_pos] = Cell::End;
                    }
                    self.path = original_node_path.to_vec();
                    node4.values[original_node_pos] = Some(Box::new(take(&mut self)));

                    if !new_path.is_empty() {
                        node4.keys[new_node_pos] = Cell::Some(new_path[0]);
                    } else {
                        node4.keys[new_node_pos] = Cell::End;
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
pub(crate) enum InnerNode {
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

#[derive(Default, PartialEq)]
pub enum Cell {
    End,
    #[default]
    None,
    Some(u8),
}

impl Cell {
    pub fn is_none(&self) -> bool {
        matches!(self, Cell::None)
    }

    pub fn unwrap(self) -> usize {
        match self {
            Cell::End | Cell::None => unreachable!("Called `.unwrap()` on {self:?}"),
            Cell::Some(i) => i as usize,
        }
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Some(k) => write!(f, "`{}`", *k as char),
            Cell::None => write!(f, "___"),
            Cell::End => write!(f, "END"),
        }
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
                    StaticNode {
                        keys: [
                            `h`,
                            `w`,
                            ___,
                            ___,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`hello` ([104, 101, 108, 108, 111])",
                                    inner: SingleValueLeaf(
                                        42,
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`world` ([119, 111, 114, 108, 100])",
                                    inner: SingleValueLeaf(
                                        43,
                                    ),
                                },
                            ),
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
                    StaticNode {
                        keys: [
                            `a`,
                            `o`,
                            ___,
                            ___,
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
                                    path: "`o` ([111])",
                                    inner: SingleValueLeaf(
                                        42,
                                    ),
                                },
                            ),
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
                    StaticNode {
                        keys: [
                            END,
                            `o`,
                            ___,
                            ___,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`` ([])",
                                    inner: SingleValueLeaf(
                                        43,
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
                    StaticNode {
                        keys: [
                            END,
                            `o`,
                            ___,
                            ___,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`` ([])",
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
                    StaticNode {
                        keys: [
                            END,
                            `a`,
                            `o`,
                            ___,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`` ([])",
                                    inner: SingleValueLeaf(
                                        44,
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
                                    path: "`o` ([111])",
                                    inner: SingleValueLeaf(
                                        42,
                                    ),
                                },
                            ),
                            None,
                        ],
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
                    StaticNode {
                        keys: [
                            END,
                            `a`,
                            `i`,
                            `o`,
                            `u`,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
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
    fn insert_4_values_with_prefix() {
        let mut art = Art::new();
        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 0,
                path: "`` ([])",
                inner: Empty,
            },
        }
        "###);
        art.insert(b"hello", 42);
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
        art.insert(b"hella", 43);
        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 2,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    StaticNode {
                        keys: [
                            `a`,
                            `o`,
                            ___,
                            ___,
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
                                    path: "`o` ([111])",
                                    inner: SingleValueLeaf(
                                        42,
                                    ),
                                },
                            ),
                            None,
                            None,
                        ],
                    },
                ),
            },
        }
        "###);
        art.insert(b"helli", 44);
        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 3,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    StaticNode {
                        keys: [
                            `a`,
                            `i`,
                            `o`,
                            ___,
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
                            None,
                        ],
                    },
                ),
            },
        }
        "###);
        art.insert(b"hellu", 45);
        insta::assert_debug_snapshot!(art, @r###"
        Art {
            root: Node {
                nb_childrens: 4,
                path: "`hell` ([104, 101, 108, 108])",
                inner: Node4(
                    StaticNode {
                        keys: [
                            `a`,
                            `i`,
                            `o`,
                            `u`,
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
                    StaticNode {
                        keys: [
                            `a`,
                            `i`,
                            `o`,
                            `u`,
                            `y`,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
                            ___,
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
                    StaticNode {
                        keys: [
                            `a`,
                            `i`,
                            `o`,
                            ___,
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
                            None,
                        ],
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
                    StaticNode {
                        keys: [
                            `l`,
                            `y`,
                            ___,
                            ___,
                        ],
                        values: [
                            Some(
                                Node {
                                    nb_childrens: 2,
                                    path: "`ll` ([108, 108])",
                                    inner: Node4(
                                        StaticNode {
                                            keys: [
                                                `a`,
                                                `o`,
                                                ___,
                                                ___,
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
                                                        path: "`o` ([111])",
                                                        inner: SingleValueLeaf(
                                                            42,
                                                        ),
                                                    },
                                                ),
                                                None,
                                                None,
                                            ],
                                        },
                                    ),
                                },
                            ),
                            Some(
                                Node {
                                    nb_childrens: 1,
                                    path: "`y` ([121])",
                                    inner: SingleValueLeaf(
                                        44,
                                    ),
                                },
                            ),
                            None,
                            None,
                        ],
                    },
                ),
            },
        }
        "###);
    }
}
