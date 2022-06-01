use rustc_middle::mir::{BasicBlock, BasicBlockData, Statement, Location};
use rustc_middle::mir::{Place, Rvalue};
use rustc_middle::mir::StatementKind::{Assign};
use rustc_middle::mir::Operand;
use rustc_middle::mir::Rvalue::{*};
use rustc_middle::mir::BorrowKind;
use rustc_middle::mir::ConstantKind;

// use crate::utils::print_mir;
use crate::stacked_borrows::{*};
use super::body_visitor::MirVisitor;

// Visitor trait implementation
impl<'tcx> MirVisitor<'tcx> {
    pub fn visit_basic_block_data(
        &mut self,
        block: BasicBlock,
        data: &BasicBlockData<'tcx>
    ) {
        println!("Block {:#?} --Start", block);
        let mut location = block.start_location();
        for statement in &data.statements {
            self.visit_statement(statement, location);
            location = location.successor_within_block();
        }
        if let Some(terminator) = &data.terminator {
            self.visit_terminator(terminator, location);
        }
        println!("Block {:#?} --End \n", block);
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
            }
            other => println!("Statement Kind not recognized {:?}", other)
        }
    }

    fn visit_assign(
        &mut self,
        place: &Place<'tcx>,
        rvalue: &Rvalue<'tcx>,
        location: Location
    ) {
        let variable = place.local.as_u32();
        let tag = self.place_to_tag(place);

        match rvalue {
            // Move or copy operand (x)
            Use(operand) => {
                print!("use ");
                self.visit_operand(operand, location);
                self.add_to_stack(place, tag);
                self.alias_graph.constant(variable);

            },
            // Reference (&x or &mut x)
            Ref(_region, borrow_kind, place) => {
                print!("ref ");
                match borrow_kind {
                    BorrowKind::Shared => { // Inmutable reference
                        self.stacked_borrows.read_value(self.place_to_tag(place));
                        self.stacked_borrows.new_ref(tag, Permission::SharedReadOnly);
                    }
                    _ => {  // Mutable reference
                        self.stacked_borrows.use_value(self.place_to_tag(place));
                        self.stacked_borrows.new_ref(tag, Permission::Unique);
                    }
                };
                self.alias_graph.points_to(variable, place.local.as_u32());
            },
            // Create a raw pointer (&raw const x)
            AddressOf(_mutability, place) => {
                print!("raw ");
                self.stacked_borrows.use_value(self.place_to_tag(place));
                self.stacked_borrows.new_ref(tag, Permission::SharedReadWrite);
                self.alias_graph.points_to(variable, place.local.as_u32());
            }
            // Creates an aggregate value, like a tuple or struct
            Aggregate(_kind,operands) => {
                print!("agg ");
                for operand in operands {
                    self.visit_operand(operand, location);
                }
                self.add_to_stack(place, tag);
            },
            Cast(_cast_kind, operand, _ty) => {
                print!("kst ");
                self.visit_operand(operand, location);
                self.add_to_stack(place, tag);
            },
            BinaryOp(_op, box_tuple) | CheckedBinaryOp(_op, box_tuple) => {
                let (operand1, operand2) = *box_tuple.clone();
                self.visit_operand(&operand1, location);
                self.visit_operand(&operand2, location);
                self.add_to_stack(place, tag);

            }
            other => println!("Rvalue kind not recognized {:?} ", other),
        }

        println!("{:#?} Assign {:?} = {:?} | {:#?}", location, place, rvalue, self.stacked_borrows);
    }

    pub fn visit_operand(
        &mut self,
        operand: &Operand<'tcx>,
        location: Location
    ) {
        match operand {
            Operand::Move(place) | Operand::Copy(place) => {
                if !place.projection.is_empty() {
                    self.stacked_borrows.use_value(self.place_to_tag(place));
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
