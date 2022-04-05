#![feature(rustc_private)]

extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;

use rustc_errors::registry;
use rustc_hash::{FxHashMap, FxHashSet};
use rustc_session::config;
use rustc_span::source_map;
use static_alias_analyzer::analyzer::analyze;

use std::path;
use std::process;
use std::str;

fn main() {
    let config = create_compiler_config();
    run_compiler(config);
}

fn create_compiler_config() -> rustc_interface::Config {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = str::from_utf8(&out.stdout).unwrap().trim();
    let config = rustc_interface::Config {
        // Command line options
        opts: config::Options {
            maybe_sysroot: Some(path::PathBuf::from(sysroot)),
            ..config::Options::default()
        },
        // cfg! configuration in addition to the default ones
        crate_cfg: FxHashSet::default(), // FxHashSet<(String, Option<String>)>
        input: config::Input::Str {
            name: source_map::FileName::Custom("main.rs".to_string()),
            input: "static HELLO: &str = \"Hello, world!\"; fn main() { println!(\"{}\", HELLO); }"
                .to_string(),
        },
        input_path: None,  // Option<PathBuf>
        output_dir: None,  // Option<PathBuf>
        output_file: None, // Option<PathBuf>
        file_loader: None, // Option<Box<dyn FileLoader + Send + Sync>>
        diagnostic_output: rustc_session::DiagnosticOutput::Default,
        stderr: None,                    // Option<Arc<Mutex<Vec<u8>>>>
        lint_caps: FxHashMap::default(), // FxHashMap<lint::LintId, lint::Level>
        parse_sess_created: None, //Option<Box<dyn FnOnce(&mut ParseSess) + Send>>
        register_lints: None, // Option<Box<dyn Fn(&Session, &mut LintStore) + Send + Sync>>
        override_queries: None, // Option<fn(&Session, &mut ty::query::Providers<'_>, &mut ty::query::Providers<'_>)>
        registry: registry::Registry::new(&rustc_error_codes::DIAGNOSTICS),
        make_codegen_backend: None,
    };
    config
}

fn run_compiler(config: rustc_interface::Config) {
    rustc_interface::run_compiler(config, |compiler| {
        compiler.enter(|queries| {
            // Analyze the program and inspect the types of definitions.
            queries.global_ctxt().unwrap().take().enter(|tcx| {
                analyze(tcx);
            })
        });
    });
}
