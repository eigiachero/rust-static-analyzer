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
use cargo_metadata::{MetadataCommand};
// use rustc_span::source_map;
use static_alias_analyzer::analyzer::analyze;

use std::path;
use std::path::PathBuf;

fn main() {
    let config = create_compiler_config();
    run_compiler(config);
}

fn compile_time_sysroot() -> Option<String> {
    let home = option_env!("HOME");
    let toolchain = option_env!("RUST_CHANNEL");
    Some(match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/.rustup/toolchains/{}-x86_64-unknown-linux-gnu", home, toolchain),
        _ => option_env!("RUST_SYSROOT")
            .expect("To build Miri without rustup, set the `RUST_SYSROOT` env var at build time")
            .to_owned(),
    })
}

fn create_compiler_config() -> rustc_interface::Config {
    let meta = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap();

    let filename = meta.packages[0].targets[0].src_path.clone();
    let directory = meta.workspace_root;
    println!("{} {}", directory, filename);

    // "/home/$username/.rustup/toolchains/nightly-2022-01-01-x86_64-unknown-linux-gnu"
    let sysroot = compile_time_sysroot().expect("Cannot find sysroot");
    println!("{}", sysroot);

    let config = rustc_interface::Config {
        // Command line options
        opts: config::Options {
            maybe_sysroot: Some(path::PathBuf::from(sysroot)),
            ..config::Options::default()
        },
        // cfg! configuration in addition to the default ones
        crate_cfg: FxHashSet::default(), // FxHashSet<(String, Option<String>)>
        input: config::Input::File(PathBuf::from(filename)),
        input_path: Some(PathBuf::from(directory)),  // Option<PathBuf>
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
