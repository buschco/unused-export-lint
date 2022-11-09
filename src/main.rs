use glob::glob;
use std::env;
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    process::ExitCode,
};
use tree_sitter::{Node, Parser};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return ExitCode::FAILURE;
    }

    let path = &args[1];

    build_export_map(path);

    return ExitCode::SUCCESS;
}

fn create_parser() -> Parser {
    let mut parser = Parser::new();

    parser
        .set_language(tree_sitter_typescript::language_tsx())
        .unwrap();

    return parser;
}

fn print_source(node: &Node, data: &str, path: &PathBuf) {
    let src_range = node.range();

    let src = match data.get(src_range.start_byte..src_range.end_byte) {
        Some(inner) => inner,
        None => {
            println!(
                "Failed to get source code at pos ln: {} col: {} from {}",
                src_range.start_byte,
                src_range.end_byte,
                path.display()
            );
            return;
        }
    };

    println!("{}", src);
}

fn add_node_to_vec(node: &Node, data: &str, vec: &mut Vec<String>) {
    match data.get(node.range().start_byte..node.range().end_byte) {
        Some(s) => {
            vec.push(s.to_owned());
        }
        _ => (),
    }
}

fn build_export_map(path: &String) {
    let mut export_map = HashMap::new();

    for entry in glob(&[path, "**/*.js"].join("/")).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let exports = find_exports(&path);
                export_map.insert(path, exports);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    dbg!(export_map);
}

fn find_exports(path: &PathBuf) -> Vec<String> {
    println!("Analyzing exports in: {}", path.display());

    let mut vec = Vec::with_capacity(0);

    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to read file {}: {}", path.display(), e);
            return vec;
        }
    };

    let mut parser = create_parser();

    let tree = match parser.parse(&data, None) {
        Some(inner) => inner,
        None => {
            println!("Failed to parse the file {}", path.display());
            return vec;
        }
    };

    let mut cursor = tree.walk();

    // Programm       ->       Code
    // ^ --goto_first_child--> ^
    cursor.goto_first_child();

    'main: loop {
        if cursor.node().kind() == "export_statement" {
            print!("Found Export Statement -> ");
            print_source(&cursor.node(), &data, &path);
            let mut export_cursor = cursor.node().walk();

            export_cursor.goto_first_child();

            'inner: loop {
                if export_cursor.node().kind() == "export_clause" {
                    // export { foo, bar }
                    // export type { foo, bar }
                    let mut export_clause_cursor = export_cursor.node().walk();
                    export_clause_cursor.goto_first_child();

                    'export_clause: loop {
                        if export_clause_cursor.node().kind() == "export_specifier" {
                            match export_clause_cursor.node().child(0) {
                                Some(exported_identifier) => {
                                    add_node_to_vec(&exported_identifier, &data, &mut vec);
                                }
                                _ => (),
                            };
                        } else if export_clause_cursor.node().kind() == "export_specifier" {
                            match export_clause_cursor.node().child(0) {
                                Some(exported_identifier) => {
                                    add_node_to_vec(&exported_identifier, &data, &mut vec);
                                }
                                _ => (),
                            };
                        }
                        if !export_clause_cursor.goto_next_sibling() {
                            break 'export_clause;
                        }
                    }
                } else if export_cursor.node().kind() == "function_declaration" {
                    // export function foo() {}
                    {
                        match export_cursor.node().child(1) {
                            Some(exported_identifier) => {
                                add_node_to_vec(&exported_identifier, &data, &mut vec);
                            }
                            _ => (),
                        };
                    }
                } else if export_cursor.node().kind() == "lexical_declaration" {
                    // export function foo() {}
                    match export_cursor.node().child(1) {
                        Some(exported_variable_declarator) => {
                            match exported_variable_declarator.child(0) {
                                Some(exported_identifier) => {
                                    add_node_to_vec(&exported_identifier, &data, &mut vec);
                                }
                                _ => (),
                            };
                        }
                        _ => (),
                    };
                } else if export_cursor.node().kind() == "type_alias_declaration" {
                    //export type Foo = {  }
                    let mut type_alias_declaration_cursor = export_cursor.node().walk();
                    type_alias_declaration_cursor.goto_first_child();
                    type_alias_declaration_cursor.goto_next_sibling();

                    if type_alias_declaration_cursor.node().kind() == "type_identifier" {
                        add_node_to_vec(&type_alias_declaration_cursor.node(), &data, &mut vec);
                    }
                } else if export_cursor.node().kind() == "identifier" {
                    //export default foo
                    vec.push("default".to_owned());
                }

                if !export_cursor.goto_next_sibling() {
                    break 'inner;
                }
            }
        }

        if !cursor.goto_next_sibling() {
            break 'main;
        }
    }

    return vec;
}
