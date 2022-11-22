use rustc_middle::mir::{BasicBlock, BasicBlockData, Statement, Location};
use rustc_middle::mir::{Place, Rvalue};
use rustc_middle::mir::StatementKind::{Assign, SetDiscriminant, StorageDead, StorageLive};
use rustc_middle::mir::Operand;
use rustc_middle::mir::Rvalue::{*};
use rustc_middle::mir::BorrowKind;
use rustc_middle::mir::ConstantKind;
use rustc_target::abi::VariantIdx;
use rustc_middle::ty::{ParamEnv, ParamEnvAnd, TyKind};

use crate::stacked_borrows::{*};
use super::body_visitor::MirVisitor;

// Visitor trait implementation
impl<'tcx> MirVisitor<'tcx> {
    pub fn visit_basic_block_data(
        &mut self,
        block: BasicBlock,
        data: &BasicBlockData<'tcx>
    ) {
        println!("Block {} {:#?} --Start\n", self.func_name, block);
        let mut location = block.start_location();
        // Visit each statement of the basic block
        for statement in &data.statements {
            self.visit_statement(statement, location);
            location = location.successor_within_block();
        }

        // Visit the basic block terminator if there is one
        if let Some(terminator) = &data.terminator {
            self.visit_terminator(terminator, location);
        }
        println!("\nBlock {} {:#?} --End \n", self.func_name, block);
    }

    fn visit_statement(
        &mut self,
        statement: &Statement<'tcx>,
        location: Location
    ) {
        match &statement.kind {
            Assign(assignment_box) => {
                let (place, rvalue) = &**assignment_box;
                self.visit_assign(place, rvalue, location);
            },
            SetDiscriminant {
                place,
                variant_index,
            } => self.visit_set_discriminant(place, *variant_index),
            StorageDead(local) | StorageLive(local) => self.visit_storage(*local),

            other => println!("Statement Kind not recognized {:?}", other)
        }
    }

    fn visit_storage(&mut self, local: rustc_middle::mir::Local) {
        let _variable = local.as_u32();
        // self.alias_graph.constant(variable);
    }

    fn visit_set_discriminant(
        &mut self,
        place: &Place<'tcx>,
        variant_index: VariantIdx,
    ) {
        self.add_to_stack(place);
    }

    fn visit_assign(
        &mut self,
        place: &Place<'tcx>,
        rvalue: &Rvalue<'tcx>,
        location: Location
    ) {
        let variable = place.local.as_u32();
        let tag = self.place_to_tag(place);
        let variable_name = self.get_variable_name(variable);
        let mut operand_name = String::new();


        match rvalue {
            // Create or mutate variable (x or *x)
            Use(operand) => {
                print!("use ");
                self.visit_operand(operand, location);
                self.add_to_stack(place);
                if !place.is_indirect() { // is not a (&x)
                    self.alias_graph.constant(variable);
                }
                if let Operand::Move(_) = operand {
                    let operand_u32 = self.operand_as_u32(operand);
                    self.alias_graph.points_to(variable, operand_u32);
                    operand_name = format!("ref {}", self.get_variable_name(operand_u32));
                }
                if let Operand::Copy(_) = operand {
                    operand_name = format!("ref {}", self.get_variable_name(self.operand_as_u32(operand)));
                }

            },
            // Reference (&x or &mut x)
            Ref(_region, borrow_kind, place) => {
                print!("ref ");
                match borrow_kind {
                    BorrowKind::Shared | BorrowKind::Shallow => { // Inmutable reference
                        self.stacked_borrows.read_value(self.place_to_tag(place));
                        self.stacked_borrows.new_ref(tag, Permission::SharedReadOnly);
                    }
                    BorrowKind::Mut {allow_two_phase_borrow} => {  // Mutable reference
                        match allow_two_phase_borrow {
                            true => {self.stacked_borrows.use_value(self.place_to_tag(place));}
                            false => {self.stacked_borrows.read_value(self.place_to_tag(place));}
                        }
                        self.stacked_borrows.new_ref(tag, Permission::Unique);
                    }
                    _ => {  // Mutable reference
                        self.stacked_borrows.use_value(self.place_to_tag(place));
                        self.stacked_borrows.new_ref(tag, Permission::Unique);
                    }
                };
                self.alias_graph.points_to(variable, place.local.as_u32());
                operand_name = format!("ref {}", self.get_variable_name(place.local.as_u32()));
            },
            // Create a raw pointer (&raw const x)
            AddressOf(_mutability, place) => {
                print!("raw ");
                self.stacked_borrows.use_value(self.place_to_tag(place));
                self.stacked_borrows.new_ref(tag, Permission::SharedReadWrite);
                self.alias_graph.points_to(variable, place.local.as_u32());
                operand_name = format!("ref {}", self.get_variable_name(place.local.as_u32()));
            }
            // Creates an aggregate value, like a tuple or struct
            Aggregate(_kind,operands) => {
                print!("agg ");
                for operand in operands {
                    self.visit_operand(operand, location);
                }
                self.add_to_stack(place);
                self.alias_graph.constant(variable);
            },
            // Check cast kind equals type - Same size of T - raw pointers
            Cast(_cast_kind, operand, ty) => {
                print!("kst ");

                let mut operand_ty = operand.ty(&self.local_declarations,self.tcx);
                let mut cast_type = *ty;

                if let TyKind::RawPtr(type_and_mut) = operand_ty.kind() {
                    operand_ty = type_and_mut.ty;
                }

                if let TyKind::RawPtr(type_and_mut) = cast_type.kind() {
                    cast_type = type_and_mut.ty;
                }

                if operand_ty.is_trivially_sized(self.tcx) && cast_type.is_trivially_sized(self.tcx) {
                    let operand_query = ParamEnvAnd { param_env: ParamEnv::empty(), value: operand_ty };
                    let ty_query = ParamEnvAnd { param_env: ParamEnv::empty(), value: cast_type };

                    match (self.tcx.layout_of(operand_query), self.tcx.layout_of(ty_query)) {
                        (Ok(operand_layout), Ok(cast_layout)) => {
                            if operand_layout.size > cast_layout.size {
                                println!("WARNING: Casting from a layout with {} bits to {} bits", operand_layout.size.bytes(), cast_layout.size.bytes());
                                println!("from {:#?} to {:#?}", operand.ty(&self.local_declarations,self.tcx), ty);
                            }
                        },
                        other => println!("Error while calculating cast type sizes"),
                    }

                }

                self.visit_operand(operand, location);
                self.add_to_stack(place);
                self.alias_graph.constant(variable);
                operand_name = format!("ref {}", self.get_variable_name(self.operand_as_u32(operand)));
            },
            BinaryOp(_op, box_tuple) | CheckedBinaryOp(_op, box_tuple) => {
                print!("bin ");
                let (operand1, operand2) = *box_tuple.clone();
                self.visit_operand(&operand2, location);
                self.visit_operand(&operand1, location);
                self.add_to_stack(place);
                self.alias_graph.constant(variable);

            },
            UnaryOp(unary, operand) => {
                print!("un  ");
                self.visit_operand(&operand, location);
                self.add_to_stack(place);
                self.alias_graph.constant(variable);
            },
            // SizeOf(T) - AlignOf(T)
            NullaryOp(_null_op, _operand) => {
                print!("nul ");
                self.add_to_stack(place);
                self.alias_graph.constant(variable);
            },
            ShallowInitBox(operand, _ty) => {
                print!("box ");
                self.add_to_stack(place);
                self.alias_graph.points_to(variable, self.operand_as_u32(operand));
            },
            Discriminant(_place) => {
                print!("dsc ");
                self.add_to_stack(place);
                self.alias_graph.constant(variable);

            }
            other => println!("Rvalue kind not recognized {:?} ", other),
        }

        // println!("{:#?} Assign {} = {:?} {} | {:#?}", location, variable_name, rvalue, operand_name, self.stacked_borrows);
        println!("{:#?} Assign {} = {:?} {}", location, variable_name, rvalue, operand_name);
    }

    pub fn visit_operand(
        &mut self,
        operand: &Operand<'tcx>,
        location: Location
    ) {
        match operand {
            Operand::Move(place) => {
                // println!("M");
                if !place.is_indirect() { // is not a (&x)
                    if self.is_raw_ptr(place) {
                        self.stacked_borrows.use_raw(self.place_to_tag(place));
                    } else {
                        self.stacked_borrows.use_value(self.place_to_tag(place));
                    }
                }
            }
            Operand::Copy(place) => {
                // println!("C");
                if self.is_raw_ptr(place) {
                    self.stacked_borrows.read_raw(self.place_to_tag(place));
                } else {
                    self.stacked_borrows.read_value(self.place_to_tag(place));
                }
            }
            Operand::Constant(boxed_constant) => {
                let constant = *boxed_constant.clone();
                match constant.literal {
                    ConstantKind::Ty(_cnst) => {
                    },
                    ConstantKind::Val(_const_val, _ty) => {
                    }
                }
            }
        }
    }
}
