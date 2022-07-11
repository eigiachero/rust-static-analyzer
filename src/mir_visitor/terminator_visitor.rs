use std::collections::HashMap;

use rustc_middle::mir::{Location, Terminator};
use rustc_middle::mir::Operand;
use rustc_middle::mir::terminator::TerminatorKind;
use rustc_middle::mir::ConstantKind;
use rustc_middle::ty::TyKind;


use petgraph::dot::{Dot, Config};
// use crate::utils::print_mir;
use crate::stacked_borrows::{*};
use super::body_visitor::MirVisitor;

// Visitor trait implementation
impl<'tcx> MirVisitor<'tcx> {
    pub fn visit_terminator(
        &mut self,
        terminator: &Terminator<'tcx>,
        location: Location
    ) {
        match terminator.kind.clone() {
            TerminatorKind::Call {
                func,
                args,
                destination,
                ..
            } => {

                println!("call {:#?}", &func);
                // To-do: analyze function profile, may-alias

                // Visit arg
                let mut index = 1;
                let mut arg_refs: HashMap<u32, u32> = HashMap::new();
                for arg in &args {
                    self.visit_operand(arg, location);
                    arg_refs.insert(index, self.operand_as_u32(arg));
                    index+=1;
                }

                // Check if there are 2 or more mutable arguments with alias
                let mutable_args: Vec<Operand> = args.clone().drain_filter(|arg| self.is_mutable(arg)).collect();
                if mutable_args.len() >= 2 {
                    println!("Caution: This function call contains two or more mutable arguments");
                    let (a, b) = (self.operand_as_u32(&mutable_args[0]), self.operand_as_u32(&mutable_args[1]));
                    if self.alias_graph.are_alias(a,b) {
                        println!("WARNING: Calling function with two mutable arguments that are alias");
                    }
                }

                // Visit inside function
                let constant = &func.constant().unwrap();
                if let ConstantKind::Ty(cnst) = constant.literal {
                    if cnst.ty.is_fn() {
                        println!("const ty {:#?}", cnst.ty);
                        if let TyKind::FnDef(def_id, subs_ref) = cnst.ty.kind() {
                            if !constant.span.from_expansion() { // Ignore if it's a macro
                                let mut visitor = MirVisitor::new(self.tcx, args);
                                visitor.visit_body(self.tcx.optimized_mir(*def_id));

                                println!("{:?}", Dot::with_config(&visitor.alias_graph.graph, &[Config::EdgeNoLabel]));
                                self.alias_graph.extend(visitor.alias_graph.graph, arg_refs);
                            }
                        }
                    }
                }

                // Add result variable to stack
                let (place, _) = destination.unwrap();
                let tag = self.place_to_tag(&place);
                if place.projection.is_empty() {
                    self.stacked_borrows.new_ref(tag, Permission::Unique);
                    self.alias_graph.constant(place.local.as_u32());
                }
                self.stacked_borrows.use_value(tag);


            },
            TerminatorKind::Assert {
                cond,
                ..
            } => {
                self.visit_operand(&cond, location);
            },
            TerminatorKind::Return => {},
            _ => {
                println!("Terminator Kind not recognized");
            }
        }
        println!("{:#?} Terminator {:#?} | {:#?}", location, terminator.kind, self.stacked_borrows);
    }
}
