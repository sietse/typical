use {
    crate::{identifier::Identifier, schema},
    std::{collections::BTreeMap, fmt::Write, path::PathBuf},
};

// This is the full list of TypeScript keywords, derived from:
//   https://github.com/microsoft/TypeScript/blob/2161e1852f4f627bbc7571c6b7284f419ec524c9
//   /src/compiler/types.ts#L113-L194
const _TYPESCRIPT_KEYWORDS: &[&str] = &[
    "abstract",
    "any",
    "as",
    "assert",
    "asserts",
    "async",
    "await",
    "bigint",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "constructor",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "global",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "instanceof",
    "interface",
    "intrinsic",
    "is",
    "keyof",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "of",
    "override",
    "package",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "return",
    "set",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "unique",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

// This struct represents a tree of schemas organized in a module hierarchy.
#[derive(Clone, Debug)]
struct Module {
    children: BTreeMap<Identifier, Module>,
    schema: schema::Schema,
}

// Generate TypeScript code from a schema and its transitive dependencies.
#[allow(clippy::too_many_lines)]
pub fn generate(
    typical_version: &str,
    schemas: &BTreeMap<schema::Namespace, (schema::Schema, PathBuf, String)>,
) -> String {
    // Construct a tree of modules and schemas. We start with an empty tree.
    let mut tree = Module {
        children: BTreeMap::new(),
        schema: schema::Schema {
            imports: BTreeMap::new(),
            declarations: BTreeMap::new(),
        },
    };

    // Populate the tree with all the schemas.
    for (namespace, (schema, _, _)) in schemas {
        insert_schema(&mut tree, namespace, schema);
    }

    // Write the code.
    let mut buffer = String::new();

    if !tree.children.is_empty() || !tree.schema.declarations.is_empty() {
        // The `unwrap` is safe because the `std::fmt::Write` impl for `String` is infallible.
        writeln!(
            &mut buffer,
            "\
// This file was automatically generated by Typical {}.
// Visit https://github.com/stepchowfun/typical for more information.

// eslint-disable

// NOTE: TypeScript support has not yet been implemented.",
            typical_version,
        )
        .unwrap();
    }

    buffer
}

// Insert a schema into a module.
fn insert_schema(module: &mut Module, namespace: &schema::Namespace, schema: &schema::Schema) {
    let mut iter = namespace.components.iter();

    if let Some(head) = iter.next() {
        if let Some(child) = module.children.get_mut(head) {
            insert_schema(
                child,
                &schema::Namespace {
                    components: iter.cloned().collect(),
                },
                schema,
            );
        } else {
            let mut child = Module {
                children: BTreeMap::new(),
                schema: schema::Schema {
                    imports: BTreeMap::new(),
                    declarations: BTreeMap::new(),
                },
            };

            insert_schema(
                &mut child,
                &schema::Namespace {
                    components: iter.cloned().collect(),
                },
                schema,
            );

            module.children.insert(head.clone(), child);
        }
    } else {
        module.schema = schema.clone();
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::{generate_typescript::generate, schema_loader::load_schemas, validator::validate},
        std::path::Path,
    };

    // This test doesn't work on Windows, for some reason.
    #[allow(clippy::too_many_lines)]
    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn generate_example() {
        let schemas = load_schemas(Path::new("integration_tests/types/main.t")).unwrap();
        validate(&schemas).unwrap();

        assert_eq!(
            generate("0.0.0", &schemas),
            "\
// This file was automatically generated by Typical 0.0.0.
// Visit https://github.com/stepchowfun/typical for more information.

// eslint-disable

// NOTE: TypeScript support has not yet been implemented.
",
        );
    }
}
