use super::bundled_crate::{BundledCrate, Exposure};

const CRATE_PATH: &str = "crate::";
const BUNDLE_IMPORT: &str = "#[allow(unused_imports)]\nuse crate::__bundled::*;\n";

/// Rewrites one source file of `bundled` so it compiles as `crate::<module>` of the bundle:
/// `crate::` paths gain the module, and the sibling crates come into scope by name.
pub(crate) fn rewrite(source: &str, bundled: &BundledCrate, exported_macros: &[&str]) -> String {
    let replaced = bundled
        .replacements
        .iter()
        .fold(source.to_owned(), |text, (from, to)| text.replace(from, to));
    let prefixed = exported_macros.iter().fold(
        prefix_crate_paths(&replaced, bundled.module),
        |text, name| {
            text.replace(
                &format!("crate::{}::{name}!", bundled.module),
                &format!("crate::{name}!"),
            )
        },
    );
    import_bundle(&prefixed)
}

/// Brings the sibling crates into scope by name, right after the file's inner attributes.
pub(crate) fn import_bundle(source: &str) -> String {
    let header = header_length(source);
    format!("{}{BUNDLE_IMPORT}{}", &source[..header], &source[header..])
}

/// Names of the `#[macro_export]` macros in `source`; they live at the bundle root.
pub(crate) fn exported_macros(source: &str) -> impl Iterator<Item = &str> {
    source.split("#[macro_export]").skip(1).filter_map(|after| {
        let name = after
            .trim_start()
            .strip_prefix("macro_rules!")?
            .trim_start();
        name.split(|character: char| !is_identifier(character))
            .next()
    })
}

/// Brings the sibling crates into scope inside the facade's inline `pub mod .. {` blocks.
pub(crate) fn import_bundle_in_inline_modules(source: &str) -> String {
    source
        .split_inclusive('\n')
        .flat_map(|line| {
            let opens_module =
                line.trim_start().starts_with("pub mod ") && line.trim_end().ends_with('{');
            [line, if opens_module { BUNDLE_IMPORT } else { "" }]
        })
        .collect()
}

/// The facade's declarations of every bundled crate, plus the `__bundled` module re-exporting them.
pub(crate) fn module_declarations(crates: &[BundledCrate]) -> String {
    let mut declarations = String::new();
    let mut reexports = String::new();
    for bundled in crates {
        let gate = bundled
            .feature
            .map(|feature| format!("#[cfg(feature = \"{feature}\")]\n"))
            .unwrap_or_default();
        let hidden = match bundled.exposure {
            Exposure::Documented => "",
            Exposure::Hidden => "#[doc(hidden)]\n",
        };
        let module = bundled.module;
        declarations.push_str(&format!(
            "\n{gate}{hidden}#[allow(missing_docs)]\n#[path = \"../{module}/src/lib.rs\"]\npub mod {module};\n"
        ));
        reexports.push_str(&format!("{gate}pub use crate::{module};\n"));
    }
    format!("{declarations}\n#[doc(hidden)]\npub mod __bundled {{\n{reexports}}}\n")
}

fn prefix_crate_paths(source: &str, module: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut copied = 0;
    for (index, _) in source.match_indices(CRATE_PATH) {
        if source[..index].ends_with(is_identifier) {
            continue;
        }
        let end = index + CRATE_PATH.len();
        output.push_str(&source[copied..end]);
        output.push_str(module);
        output.push_str("::");
        copied = end;
    }
    output.push_str(&source[copied..]);
    output
}

/// Byte length of the leading comments, blank lines and inner attributes, which must stay first.
fn header_length(source: &str) -> usize {
    let mut length = 0;
    let mut open_brackets = 0_usize;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let inner_attribute = open_brackets > 0 || trimmed.starts_with("#![");
        let outer_doc = trimmed.starts_with("///") && !trimmed.starts_with("////");
        let comment = trimmed.starts_with("//") && !outer_doc;
        if !(inner_attribute || trimmed.is_empty() || comment) {
            break;
        }
        if inner_attribute {
            open_brackets = (open_brackets + line.matches('[').count())
                .saturating_sub(line.matches(']').count());
        }
        length += line.len();
    }
    length
}

fn is_identifier(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: BundledCrate = BundledCrate {
        module: "view",
        directory: "view",
        feature: None,
        exposure: Exposure::Documented,
        replacements: &[],
    };

    #[test]
    fn crate_paths_gain_the_module_and_the_import_follows_inner_attributes() {
        let source = "//! Docs.\n#![deny(\n    missing_docs\n)]\n\n/// Item docs.\nuse crate::cx::Cx;\npub(in crate::cx) fn f() -> $crate::A {}\nfn g(_: my_crate::B) {}\n";
        assert_eq!(
            rewrite(source, &VIEW, &[]),
            "//! Docs.\n#![deny(\n    missing_docs\n)]\n\n#[allow(unused_imports)]\nuse crate::__bundled::*;\n/// Item docs.\nuse crate::view::cx::Cx;\npub(in crate::view::cx) fn f() -> $crate::view::A {}\nfn g(_: my_crate::B) {}\n"
        );
    }

    #[test]
    fn exported_macros_stay_at_the_bundle_root() {
        let source = "#[macro_export]\nmacro_rules! section { () => { $crate::Key }; }\ncrate::section! {}\n";
        let names: Vec<&str> = exported_macros(source).collect();
        assert_eq!(names, ["section"]);
        assert!(
            rewrite(source, &VIEW, &names)
                .ends_with("$crate::view::Key }; }\ncrate::section! {}\n")
        );
    }

    #[test]
    fn inline_modules_of_the_facade_import_the_bundle() {
        assert_eq!(
            import_bundle_in_inline_modules("pub mod prelude {\n    pub use geometry::Rect;\n}\n"),
            format!("pub mod prelude {{\n{BUNDLE_IMPORT}    pub use geometry::Rect;\n}}\n")
        );
    }
}
