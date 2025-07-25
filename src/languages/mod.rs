use std::{fs, path::PathBuf, process::Command, sync::Mutex};

use duat_core::{
    context,
    text::{Text, txt},
};
use indoc::{formatdoc, indoc};
use libloading::Library;
use tree_sitter::Language;

use self::list::LANGUAGE_OPTIONS;

mod list;

/// Wether the parser for the given `filetype` is compiled
pub fn parser_is_compiled(filetype: &str) -> Result<bool, Text> {
    let options = LANGUAGE_OPTIONS
        .get(filetype)
        .ok_or_else(|| txt!("There is no tree-sitter grammar for [a].{filetype}[] files"))?;

    let lib = options.crate_name.replace("-", "_");
    let so_path = get_workspace_dir()?.join(format!("parsers/lib/lib{lib}.so"));

    Ok(so_path.try_exists()?)
}

pub fn get_language(filetype: &str) -> Result<Language, Text> {
    static LIBRARIES: Mutex<Vec<Library>> = Mutex::new(Vec::new());

    let parsers_dir = get_workspace_dir()?.join("parsers");
    let options = LANGUAGE_OPTIONS
        .get(filetype)
        .ok_or_else(|| txt!("There is no tree-sitter grammar for [a]{filetype}"))?;

    let lib_dir = parsers_dir.join("lib");
    let crate_dir = parsers_dir.join(format!("ts-{}", options.crate_name));
    let manifest_path = crate_dir.join("Cargo.toml");

    let lib = options.crate_name.replace("-", "_");
    let so_path = lib_dir.join(format!("lib{lib}.so"));

    if let Ok(lib) = unsafe { Library::new(so_path) } {
        context::debug!("Loading tree-sitter parser for [a]{filetype}");
        let language = unsafe {
            let (symbol, _) = options.symbols[0];
            let lang_fn = lib
                .get::<fn() -> Language>(symbol.to_lowercase().as_bytes())
                .map_err(|err| txt!("{err}"))?;

            lang_fn()
        };

        LIBRARIES.lock().unwrap().push(lib);

        Ok(language)
    } else if let Ok(true) = fs::exists(&manifest_path) {
        context::info!("Compiling tree-sitter parser for [a]{filetype}");
        let mut cargo = Command::new("cargo");

        cargo.args([
            "build",
            "--release",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "-Z",
            "unstable-options",
            "--artifact-dir",
            lib_dir.to_str().unwrap(),
        ]);

        let out = cargo.output()?;
        if out.status.success() {
            get_language(filetype)
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string().into())
        }
    } else {
        let lib_rs: String = options
            .symbols
            .iter()
            .map(|(symbol, is_function)| {
                let fn_name = symbol.to_lowercase();
                let language = if *is_function {
                    format!("ts::{symbol}()")
                } else {
                    format!("ts::{symbol}.into()")
                };

                formatdoc! {"
                    #[unsafe(no_mangle)]    
                    pub fn {fn_name}() -> tree_sitter::Language {{
                        {language}
                    }}
                "}
            })
            .collect();

        let crate_name = options.crate_name;
        let git = options.git;

        let cargo_toml = formatdoc! {r#"
            [package]
            name = "ts-{crate_name}"
            version = "0.1.0"
            edition = "2024"
            description = "Dynamic wrapper for tree-sitter-{crate_name}"

            [lib]
            name = "{lib}"
            crate-type = ["dylib"]

            [dependencies]
            tree-sitter = "*"

            [dependencies.ts]
            version = "*"
            git = "{git}"
            package = "tree-sitter-{crate_name}"
        "#};

        fs::create_dir_all(crate_dir.join("src"))?;
        fs::write(manifest_path, cargo_toml)?;
        fs::write(crate_dir.join("src/lib.rs"), lib_rs)?;

        get_language(filetype)
    }
}

fn get_workspace_dir() -> Result<PathBuf, Text> {
    const BASE_WORKSPACE_TOML: &str = indoc!(
        r#"
        [workspace]
        resolver = "2"
        members = ["*"]
        exclude = ["lib", "target"]"#
    );

    let workspace_dir = duat_core::plugin_dir("duat-treesitter")?;
    let parsers_dir = workspace_dir.join("parsers");

    if let Ok(false) | Err(_) = fs::exists(&parsers_dir) {
        fs::create_dir_all(workspace_dir.join("parsers"))?;
        fs::write(parsers_dir.join("Cargo.toml"), BASE_WORKSPACE_TOML)?
    }

    Ok(workspace_dir)
}

struct LanguageOptions {
    git: &'static str,
    symbols: &'static [(&'static str, bool)],
    crate_name: &'static str,
    _maintainers: &'static [&'static str],
}

impl LanguageOptions {
    fn pair(
        lang: &'static str,
        git: &'static str,
        _maintainers: &'static [&'static str],
    ) -> (&'static str, Self) {
        let options = Self {
            git,
            symbols: &[("LANGUAGE", false)],
            crate_name: crate_name(lang),
            _maintainers,
        };

        (lang, options)
    }

    fn pairs_with_symbol(
        lang: &'static str,
        git: &'static str,
        symbols: &'static [(&'static str, bool)],
        _maintainers: &'static [&'static str],
    ) -> (&'static str, LanguageOptions) {
        let options = Self {
            git,
            symbols,
            crate_name: crate_name(lang),
            _maintainers,
        };

        (lang, options)
    }

    fn pairs_with_symbol_and_crate(
        lang: &'static str,
        git: &'static str,
        symbols: &'static [(&'static str, bool)],
        crate_name: &'static str,
        _maintainers: &'static [&'static str],
    ) -> (&'static str, LanguageOptions) {
        let options = Self { git, symbols, crate_name, _maintainers };

        (lang, options)
    }
}

fn crate_name(lang: &'static str) -> &'static str {
    if lang.contains("_") {
        lang.replace("_", "-").leak()
    } else {
        lang
    }
}
