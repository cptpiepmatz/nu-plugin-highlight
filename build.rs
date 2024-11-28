use std::path::PathBuf;

use bat::assets::HighlightingAssets;
use patch_apply::Patch;
use syntect::parsing::SyntaxDefinition;

const NUSHELL_SYNTAX: &str = include_str!("./syntaxes/nushell/nushell.sublime-syntax");
const NUSHELL_PATCH: &str = include_str!("./syntaxes/patches/nushell.sublime-syntax.patch");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=syntaxes/nushell/nushell.sublime-syntax");
    println!("cargo:rerun-if-changed=syntaxes/patches/nushell.sublime-syntax.patch");
    println!("cargo:rerun-if-env-changed=OUT_DIR");

    let syntax_set = HighlightingAssets::from_binary()
        .get_syntax_set()
        .unwrap()
        .clone();
    let mut syntax_set_builder = syntax_set.into_builder();

    let patch = Patch::from_single(NUSHELL_PATCH).unwrap();
    let syntax = NUSHELL_SYNTAX.to_string();
    let syntax = patch_apply::apply(syntax, patch);
    let syntax = SyntaxDefinition::load_from_str(&syntax, true, Some("nushell")).unwrap();

    syntax_set_builder.add(syntax);
    let syntax_set = syntax_set_builder.build();

    let out_path = std::env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_path).join("syntax_set.bin");
    syntect::dumps::dump_to_uncompressed_file(&syntax_set, out_path).unwrap();
}
