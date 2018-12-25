use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Default)]
pub struct Node {
    pub label: String,
    pub nodes: Vec<Node>,
}

impl Node {
    pub fn new(label: impl Into<String>, nodes: Vec<Node>) -> Node {
        Node {
            label: label.into(),
            nodes,
        }
    }
}

//impl fmt::Display for Node {
//    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//        archy(self)
//    }
//}

#[derive(Debug)]
pub struct ArchyOpts {
    pub unicode: bool
}

impl ArchyOpts {
    pub fn new() -> ArchyOpts {
        Default::default()
    }
    pub fn unicode(mut self, value: bool) -> ArchyOpts {
        self.unicode = value;
        self
    }
}

impl Default for ArchyOpts {
    fn default() -> Self {
        ArchyOpts {
            unicode: true
        }
    }
}

pub fn archy(input: &Node, prefix: &str, opts: &ArchyOpts) -> String {
    let get_char = |input| {
        match input {
            "│" => if opts.unicode { "│" } else { "|" },
            "└" => if opts.unicode { "└" } else { "`" },
            "├" => if opts.unicode { "├" } else { "+" },
            "─" => if opts.unicode { "─" } else { "-" },
            "┬" => if opts.unicode { "┬" } else { "-" },
            _ => "-"
        }
    };
    let lines = input.label.lines().collect::<Vec<&str>>();
    let node_len = input.nodes.len();
    let suf = if node_len > 0 { get_char("│") } else { " " };
    let splitter = format!("\n{}{} ", prefix, suf);
    let joined = lines.join(splitter.as_str());

    let child = input.nodes.iter().enumerate().map(|(ix, node)| {
        let last = ix == (node_len - 1);
        let more = node.nodes.len() > 0;
        let prefix_ = format!("{}{} ", prefix, if last { " " } else { get_char("│") });
        let next_string = &archy(&node, prefix_.as_str(), &opts);
        let next_string_indices = next_string.char_indices();

        let target_num = prefix.char_indices().count() + 2;

        let next_output = next_string_indices.skip(target_num)
            .map(|(i, char)| char.to_string())
            .collect::<Vec<String>>()
            .join("");

        vec![
            prefix,
            if last { get_char("└") } else { get_char("├") },
            get_char("─"),
            if more { get_char("┬") } else { get_char("─") },
            " ",
            next_output.as_str()
        ].join("")
    }).collect::<Vec<String>>();

    format!("{}{}\n{}", prefix, joined, child.join(""))
}

#[test]
fn single_node() {
    let node = Node::new("Hello", vec![]);
    let opts = ArchyOpts::new().unicode(false);
    assert_eq!("Hello\n", archy(&node, "", &opts));
}

#[test]
fn multi_node() {
    let node = Node::new("first", vec![
        Node::new("second", vec![]),
        Node::new("third", vec![]),
    ]);
    let opts = ArchyOpts::new();
    assert_eq!("first
├── second
└── third\n", archy(&node, "", &opts));
}

#[test]
fn multi_level() {
    let node = Node::new("first", vec![
        Node::new("second", vec![
            Node::new("third", vec![]),
        ]),
    ]);
    let opts = ArchyOpts::new();
    assert_eq!("first
└─┬ second
  └── third\n", archy(&node, "", &opts));
}

#[test]
fn multi_line_label() {
    let node = Node::new("first", vec![
        Node::new("second", vec![
            Node::new("third\nwith\nmultiple\nlines", vec![]),
            Node::new("forth", vec![]),
        ]),
    ]);
    let opts = ArchyOpts::new();
    println!("{}", archy(&node, "", &opts));
    assert_eq!("first
└─┬ second
  ├── third
  │   with
  │   multiple
  │   lines
  └── forth\n", archy(&node, "", &opts));
}