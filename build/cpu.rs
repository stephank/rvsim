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
    args: Vec<(String, String)>,
}

struct ParseNode {
    field: String,
    actions: HashMap<String, ParseAction>,
}

enum ParseAction {
    Finish(Rc<Variant>),
    Descend(ParseNode),
}

pub fn build() {
    println!("# generating cpu code");

    let mut variants = vec![];
    let mut parse_tree = ParseNode {
        field: "opcode".to_owned(),
        actions: HashMap::new(),
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Simple regex to parse function arguments, skipping `self`.
    // Notably, this also strips off a `_` prefix for unused arguments.
    let args_re = Regex::new(r"_?([_a-z0-9]+)\s*:\s*([_a-z0-9]+)\s*[,)]").unwrap();

    // Read `interp.in.rs` by line, keeping the previous line aruond.
    let reader = BufReader::new(File::open("src/cpu/interp.in.rs").unwrap());
    let mut prev = String::new();
    for line in reader.lines() {
        let line = line.unwrap().trim().to_owned();

        if prev.starts_with("//% ") && line.starts_with("fn ") {
            // Extract the method name and argument names.
            let method = line[3..].splitn(2, '(').next().unwrap().to_owned();
            let args = args_re.captures_iter(&line)
                .map(|c| (c[1].to_owned(), c[2].to_owned()))
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

            // Build the parse tree.
            let mut node = &mut parse_tree;
            let mut prev_value = {
                let (field, value) = matchers[0];
                assert_eq!(field, "opcode", "parser must start with opcode field");
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

        prev = line;
    }

    // Generate `Op` variants source code.
    let mut variants_src = String::new();
    for variant in &variants {
        let &Variant { ref name, ref args, .. } = &**variant;
        if args.is_empty() {
            writeln!(variants_src, "    {},", name).unwrap();
        } else {
            let field_src = args.iter().map(|&(ref name, ref typ)| {
                format!("{}: {}", name, typ)
            }).collect::<Vec<_>>().join(", ");
            writeln!(variants_src, "    {} {{ {} }},", name, field_src).unwrap();
        }
    }

    // Generate `Op::parse` source code.
    fn node_parse_src(node: &ParseNode, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        let mut src = format!("{}match {}(instr) {{\n", spaces, node.field);
        for (value, action) in &node.actions {
            src.push_str(&format!("{}    0b{} => {{\n", spaces, value));
            match action {
                &ParseAction::Descend(ref child) => {
                    src.push_str(&node_parse_src(child, indent + 8));
                },
                &ParseAction::Finish(ref variant) => {
                    src.push_str(&format!("{}        Some(Op::{} {{\n", spaces, variant.name));
                    for &(ref name, _) in &variant.args {
                        src.push_str(&format!("{}            {}: {}(instr),\n", spaces, name, name));
                    }
                    src.push_str(&format!("{}        }})\n", spaces));
                },
            }
            src.push_str(&format!("{}    }},\n", spaces));
        }
        src.push_str(&format!("{}    _ => None,\n", spaces));
        src.push_str(&format!("{}}}\n", spaces));
        src
    }
    let parse_src = node_parse_src(&parse_tree, 8);

    // Generate `Interp` dispatch source code.
    let mut dispatch_src = String::new();
    let spaces = " ".repeat(12);
    for variant in &variants {
        let &Variant { ref name, ref method, ref args } = &**variant;
        let params = args.iter().map(|&(ref name, _)| name.as_str())
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
            _ => writeln!(file, "{}", line),
        }.unwrap();
    }

    // Generate the `interp.rs`.
    let reader = BufReader::new(File::open("src/cpu/interp.in.rs").unwrap());
    let mut file = File::create(out_path.join("interp.rs")).unwrap();
    for line in reader.lines() {
        let line = line.unwrap();
        match line.trim() {
            "//% dispatch" => file.write_all(dispatch_src.as_bytes()),
            _ => writeln!(file, "{}", line),
        }.unwrap();
    }
}
