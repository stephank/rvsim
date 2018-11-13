use regex::Regex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env;
use std::fmt::{Write as FmtWrite};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::rc::Rc;

struct Variant {
    name: String,
    method: String,
    // name, extract, typ
    args: Vec<(String, String, String)>,
}

struct ParseNode {
    field: String,
    actions: HashMap<String, ParseAction>,
}

enum ParseAction {
    Finish(Rc<Variant>),
    Descend(ParseNode),
}

/** Build the parse tree. */
fn build_parse_tree(parse_tree: &mut ParseNode, matchers: &Vec<(&str, &str)>, first_field: &str, variant: Rc<Variant>) {
    let mut node = parse_tree;
    let mut prev_value = {
        let (field, value) = matchers[0];
        assert_eq!(field, first_field, "parser must start with {} field", first_field);
        value.to_owned()
    };
    for &(next_field, next_value) in &matchers[1..] {
        let prev_node = node;
        match prev_node.actions.entry(prev_value).or_insert_with(|| {
            ParseAction::Descend(ParseNode {
                field: next_field.to_owned(),
                actions: HashMap::new(),
            })
        }) {
            &mut ParseAction::Descend(ref mut next_node) => {
                assert_eq!(next_node.field, next_field, "parser field order must match");
                node = next_node;
            },
            &mut ParseAction::Finish(_) => {
                panic!("parser tried to descend into existing match");
            },
        }
        prev_value = next_value.to_owned();
    }
    match node.actions.entry(prev_value) {
        Entry::Vacant(entry) => {
            entry.insert(ParseAction::Finish(variant));
        },
        Entry::Occupied(_) => {
            panic!("conflicting field value in parser");
        },
    }
}

// Struct for selectively skipping rv32fd opcodes if rv32fd feature is disabled
struct SkipDisabled {
    #[cfg(not(feature = "rv32fd"))]
    only_rv32fd: bool,
}

impl SkipDisabled {
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "rv32fd"))]
            only_rv32fd: false,
        }
    }

    #[cfg(not(feature = "rv32fd"))]
    pub fn do_skip(&mut self, line: &str) -> bool {
        if line.starts_with("//f{") {
            self.only_rv32fd = true;
            true
        } else if line.starts_with("//f}") {
            self.only_rv32fd = false;
            true
        } else {
            self.only_rv32fd
        }
    }

    #[cfg(feature = "rv32fd")]
    pub fn do_skip(&mut self, _line: &str) -> bool {
        false
    }
}

pub fn build() {
    println!("# generating cpu code");

    let mut variants = vec![];
    let mut parse_tree = ParseNode {
        field: "opcode".to_owned(),
        actions: HashMap::new(),
    };
    let mut parse_tree_c = ParseNode {
        field: "cquad".to_owned(),
        actions: HashMap::new(),
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Simple regex to parse function arguments, skipping `self`.
    // Notably, this also strips off a `_` prefix for unused arguments.
    let args_re = Regex::new(r"_?([_a-z0-9]+)\s*:\s*([_a-z0-9]+)\s*[,)]").unwrap();

    // Read `interp.in.rs` by line, keeping the previous line aruond.
    let reader = BufReader::new(File::open("src/cpu/interp.in.rs").unwrap());
    let mut prev = String::new();
    let mut skipper = SkipDisabled::new();
    for line in reader.lines() {
        let line = line.unwrap().trim().to_owned();

        if skipper.do_skip(&line) {
            continue;
        }

        if prev.starts_with("//% ") && line.starts_with("fn ") {
            // Extract the method name and argument names.
            let method = line[3..].splitn(2, '(').next().unwrap().to_owned();
            let args = args_re.captures_iter(&line)
                .map(|c| (c[1].to_owned(), c[1].to_owned(), c[2].to_owned()))
                .collect::<Vec<_>>();

            // Camelcase the method name to create the `Op` enum variant.
            let name = method.split('_').map(|s| {
                format!("{}{}", s[..1].to_uppercase(), &s[1..])
            }).collect::<Vec<_>>().join("");

            // Create the variant.
            let variant = Rc::new(Variant { name, method, args });
            variants.push(Rc::clone(&variant));

            // Parse the matchers in the comment.
            let matchers = prev[4..].split_whitespace().map(|s| {
                let mut split = s.splitn(2, '=');
                let field = split.next().unwrap();
                let value = split.next().unwrap();
                (field, value)
            }).collect::<Vec<_>>();

            build_parse_tree(&mut parse_tree, &matchers, "opcode", variant);
        }

        // Parsing for decompression
        if prev.starts_with("//% ") && line.starts_with("//    ") {
            // Parse the matchers in the comment.
            let matchers = prev[4..].split_whitespace().map(|s| {
                let mut split = s.splitn(2, '=');
                let field = split.next().unwrap();
                let value = split.next().unwrap();
                (field, value)
            }).collect::<Vec<_>>();

            // Parse the metadata in the comment.
            let meta = line[6..].split_whitespace().map(|s| {
                let mut split = s.splitn(2, '=');
                let field = split.next().unwrap();
                let value = split.next().unwrap();
                (field, value)
            }).collect::<Vec<_>>();

            assert_eq!(meta[0].0, "name", "rv32c description must start with instruction name");
            assert_eq!(meta[1].0, "decomp", "second part of rv32c description must be decompressed instruction name");

            // Camelcase the method name to create the `Op` enum variant.
            let name = meta[1].1.split('_').map(|s| {
                format!("{}{}", s[..1].to_uppercase(), &s[1..])
            }).collect::<Vec<_>>().join("");

            let args = meta[2..].iter().map(|(a,b)| { ( a.to_string(), b.to_string(), String::new() ) }).collect::<Vec<_>>();
            let variant = Rc::new(Variant { name, method:meta[1].1.to_string(), args });

            build_parse_tree(&mut parse_tree_c, &matchers, "cquad", variant);
        }

        prev = line;
    }

    // Generate `Op` variants source code.
    let mut variants_src = String::new();
    for variant in &variants {
        let &Variant { ref name, ref args, .. } = &**variant;
        if args.is_empty() {
            writeln!(variants_src, "    {},", name).unwrap();
        } else {
            let field_src = args.iter().map(|&(ref name, _, ref typ)| {
                format!("{}: {}", name, typ)
            }).collect::<Vec<_>>().join(", ");
            writeln!(variants_src, "    {} {{ {} }},", name, field_src).unwrap();
        }
    }

    // Generate `Op::parse` source code.
    fn node_parse_src(node: &ParseNode, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        let mut src = format!("{}match {}(instr) {{\n", spaces, node.field);
        let mut have_default = false;
        let mut items = node.actions.iter().collect::<Vec<_>>();
        items.sort_by_key(|(k,_)| {*k});

        for (value, action) in items {
            if value == "_" {
                src.push_str(&format!("{}    _ => {{\n", spaces));
                have_default = true;
            } else {
                src.push_str(&format!("{}    0b{} => {{\n", spaces, value));
            }
            match action {
                &ParseAction::Descend(ref child) => {
                    src.push_str(&node_parse_src(child, indent + 8));
                },
                &ParseAction::Finish(ref variant) => {
                    if variant.method == "illegal" {
                        src.push_str(&format!("{}        None\n", spaces));
                    } else {
                        src.push_str(&format!("{}        Some(Op::{} {{\n", spaces, variant.name));
                        for &(ref name, ref extract, _) in &variant.args {
                            src.push_str(&format!("{}            {}: {}(instr),\n", spaces, name, extract));
                        }
                        src.push_str(&format!("{}        }})\n", spaces));
                    }
                },
            }
            src.push_str(&format!("{}    }},\n", spaces));
        }
        if !have_default {
            src.push_str(&format!("{}    _ => None,\n", spaces));
        }
        src.push_str(&format!("{}}}\n", spaces));
        src
    }
    let parse_src = node_parse_src(&parse_tree, 8);

    // Generate `Op::parse_c` source code.
    let parse_c_src = node_parse_src(&parse_tree_c, 8);

    // Generate `Interp` dispatch source code.
    let mut dispatch_src = String::new();
    let spaces = " ".repeat(12);
    for variant in &variants {
        let &Variant { ref name, ref method, ref args } = &**variant;
        let params = args.iter().map(|&(ref name, _, _)| name.as_str())
            .collect::<Vec<_>>().join(", ");
        let pattern = if params.is_empty() {
            "".to_owned()
        } else {
            format!(" {{ {} }}", params)
        };
        writeln!(dispatch_src, "{}Op::{}{} => self.{}({}),",
            spaces, name, pattern, method, params).unwrap();
    }

    // Generate the `op.rs`.
    let reader = BufReader::new(File::open("src/cpu/op.in.rs").unwrap());
    let mut file = File::create(out_path.join("op.rs")).unwrap();
    for line in reader.lines() {
        let line = line.unwrap();
        match line.trim() {
            "//% variants" => file.write_all(variants_src.as_bytes()),
            "//% parse" => file.write_all(parse_src.as_bytes()),
            "//% parse_c" => file.write_all(parse_c_src.as_bytes()),
            _ => writeln!(file, "{}", line),
        }.unwrap();
    }

    // Generate the `interp.rs`.
    let reader = BufReader::new(File::open("src/cpu/interp.in.rs").unwrap());
    let mut file = File::create(out_path.join("interp.rs")).unwrap();
    let mut skipper = SkipDisabled::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let line_trim = line.trim();

        if skipper.do_skip(&line_trim) {
            continue;
        }

        match line_trim {
            "//% dispatch" => file.write_all(dispatch_src.as_bytes()),
            _ => writeln!(file, "{}", line),
        }.unwrap();
    }
}
