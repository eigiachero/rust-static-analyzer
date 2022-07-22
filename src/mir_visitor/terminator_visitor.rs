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
                            // Ignore if it's a macro or if the mir is not available
                            if !constant.span.from_expansion() && self.tcx.is_mir_available(*def_id) {
                                let body = self.tcx.optimized_mir(*def_id);
                                let mut visitor = MirVisitor::new(self.tcx, body, args);
                                visitor.visit_body(body);

                                println!("{:?}", Dot::with_config(&visitor.alias_graph.graph, &[Config::EdgeNoLabel]));
                                // self.alias_graph.extend(visitor.alias_graph.graph, arg_refs);
                            }
                        }
                    }
                }

                // Add result variable to stack
                if let Some((place, _)) = destination {
                    let tag = self.place_to_tag(&place);
                    if !place.is_indirect() { // place does not contain a Deref
                        self.stacked_borrows.new_ref(tag, Permission::Unique);
                        self.alias_graph.constant(place.local.as_u32());
                    }
                    self.stacked_borrows.use_value(tag);
                }
            },
            TerminatorKind::Assert {
                cond,
                ..
            } => {
                self.visit_operand(&cond, location);
            },
            TerminatorKind::SwitchInt {
                discr,
                switch_ty,
                targets
            } => {
                // println!("SwitchInt {:#?} {:#?} {:#?}", discr, switch_ty, targets);
                self.visit_operand(&discr, location);
            },
            TerminatorKind::Goto {
                target
            } => {
                // println!("goto {:#?}");
                // self.visit_basic_block_data(target, self.body.index(target));
            },
            TerminatorKind::Drop {
                place,
                target,
                unwind
            } => {
                self.stacked_borrows.clean();
            }
            TerminatorKind::Return
            | TerminatorKind::Resume
            | TerminatorKind::Unreachable
            => {},
            _ => {
                println!("Terminator Kind not recognized");
            }
        }
        println!("{:#?} Terminator {:#?} | {:#?}", location, terminator.kind, self.stacked_borrows);
    }
}
