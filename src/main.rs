use glob::glob;
use path_clean::clean;
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

    build_import_export_map(path);

    return ExitCode::SUCCESS;
}

fn create_parser() -> Parser {
    let mut parser = Parser::new();

    parser
        .set_language(tree_sitter_typescript::language_tsx())
        .unwrap();

    return parser;
}

fn add_export(node: &Node, data: &str, vec: &mut Vec<String>) {
    if let Some(s) = data.get(node.range().start_byte..node.range().end_byte) {
        vec.push(s.to_owned());
    }
}

fn build_import_export_map(dir: &str) {
    let mut import_exports_map = HashMap::new();

    for path in glob(&[dir, "**/*.js"].join("/"))
        .expect("Failed to read glob pattern")
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
    {
        let (imports, exports) = collect_import_export_statements(&path);
        import_exports_map.insert(path.to_owned(), (imports, exports));
    }

    dbg!(import_exports_map);
}

fn collect_import_export_statements(path: &PathBuf) -> (Vec<(Vec<String>, String)>, Vec<String>) {
    println!("Analyzing exports in: {}", path.display());

    let mut imports = Vec::with_capacity(0);
    let mut exports = Vec::with_capacity(0);

    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to read file {}: {}", path.display(), e);
            return (imports, exports);
        }
    };

    let mut parser = create_parser();

    let tree = match parser.parse(&data, None) {
        Some(inner) => inner,
        None => {
            println!("Failed to parse the file {}", path.display());
            return (imports, exports);
        }
    };

    let mut cursor = tree.walk();

    // Programm       ->       Code
    // ^ --goto_first_child--> ^
    cursor.goto_first_child();

    'main: loop {
        if cursor.node().kind() == "import_statement" {
            let mut import_cursor = cursor.node().walk();
            import_cursor.goto_first_child();

            let mut source: Option<&str> = None;
            let mut identifiers = Vec::with_capacity(0);

            'inner: loop {
                if import_cursor.node().kind() == "import_clause" {
                    let mut import_clause_cursor = import_cursor.node().walk();
                    import_clause_cursor.goto_first_child();

                    if import_clause_cursor.node().child_count() > 0 {
                        let mut identifiers_cursor = import_clause_cursor.node().walk();
                        identifiers_cursor.goto_first_child();

                        'identifiers: loop {
                            if identifiers_cursor.node().kind() == "import_specifier" {
                                if let Some(identifier) = data.get(
                                    identifiers_cursor.node().range().start_byte
                                        ..identifiers_cursor.node().range().end_byte,
                                ) {
                                    identifiers.push(identifier.to_owned())
                                }
                            }

                            if !identifiers_cursor.goto_next_sibling() {
                                break 'identifiers;
                            }
                        }
                    } else {
                        identifiers.push("default".to_owned());
                    }
                } else if import_cursor.node().kind() == "string" {
                    source = data.get(
                        import_cursor.node().range().start_byte
                            ..import_cursor.node().range().end_byte,
                    );
                };

                if !import_cursor.goto_next_sibling() {
                    break 'inner;
                }
            }

            if let Some(s) = source {
                let s_cleaned = s.replace("\"", "").replace("'", "").to_owned();
                if s_cleaned.starts_with('.') {
                    if let Some(source_path) =
                        path.clone().with_file_name("").join(&s_cleaned).to_str()
                    {
                        imports.push((identifiers.to_owned(), clean(source_path).to_owned()));
                    }
                }
            }
        }
        if cursor.node().kind() == "export_statement" {
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
                            if let Some(exported_identifier) = export_clause_cursor.node().child(0)
                            {
                                add_export(&exported_identifier, &data, &mut exports);
                            };
                        }

                        if !export_clause_cursor.goto_next_sibling() {
                            break 'export_clause;
                        }
                    }
                } else if export_cursor.node().kind() == "function_declaration" {
                    // export function foo() {}
                    {
                        if let Some(exported_identifier) = export_cursor.node().child(1) {
                            add_export(&exported_identifier, &data, &mut exports);
                        };
                    }
                } else if export_cursor.node().kind() == "lexical_declaration" {
                    // export function foo() {}
                    if let Some(exported_variable_declarator) = export_cursor.node().child(1) {
                        if let Some(exported_identifier) = exported_variable_declarator.child(0) {
                            add_export(&exported_identifier, &data, &mut exports);
                        };
                    };
                } else if export_cursor.node().kind() == "type_alias_declaration" {
                    // export type Foo = {  }
                    let mut type_alias_declaration_cursor = export_cursor.node().walk();
                    type_alias_declaration_cursor.goto_first_child();
                    type_alias_declaration_cursor.goto_next_sibling();

                    if type_alias_declaration_cursor.node().kind() == "type_identifier" {
                        add_export(&type_alias_declaration_cursor.node(), &data, &mut exports);
                    }
                } else if export_cursor.node().kind() == "default" {
                    // export default foo
                    exports.push("default".to_owned());
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

    return (imports, exports);
}
