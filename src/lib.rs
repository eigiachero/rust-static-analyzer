#![feature(rustc_private)]
#![allow(unused_variables)]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate log;

pub mod analyzer;
pub mod utils;
pub mod stacked_borrows;
pub mod mir_visitor;