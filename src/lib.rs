#![feature(rustc_private)]
#![allow(unused_variables)]
#![feature(drain_filter)]

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
pub mod points_to;

pub mod mir_visitor {
    pub mod block_visitor;
    pub mod body_visitor;
    pub mod terminator_visitor;
    pub mod helper;
}