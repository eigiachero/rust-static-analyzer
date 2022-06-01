use rustc_middle::mir::{Location, Terminator};
use rustc_middle::mir::Operand;
use rustc_middle::mir::terminator::TerminatorKind;
use rustc_middle::mir::ConstantKind;
use rustc_middle::ty::TyKind;

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
                for arg in &args {
                    self.visit_operand(arg, location);
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
                if let Operand::Constant(boxed_constant) = &func {
                    let constant = *boxed_constant.clone();
                    if let ConstantKind::Ty(cnst) = constant.literal {
                        if cnst.ty.is_fn() {
                            println!("const ty {:#?}", cnst.ty);
                            if let TyKind::FnDef(def_id, subs_ref) = cnst.ty.kind() {
                                let mut visitor = MirVisitor::new(self.tcx, args);
                                visitor.visit_body(self.tcx.optimized_mir(*def_id));
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
