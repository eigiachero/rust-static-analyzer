use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::{BasicBlock, BasicBlockData, Body, Statement, Location, Terminator};
use rustc_middle::mir::{Local, LocalDecl, LocalDecls, Place, Rvalue};
use rustc_middle::mir::StatementKind::{Assign};
use rustc_middle::mir::Operand;
use rustc_middle::mir::Rvalue::{*};
use rustc_middle::mir::BorrowKind;
use rustc_middle::ty::{TyCtxt};
use rustc_middle::mir::terminator::TerminatorKind;
use rustc_middle::mir::ProjectionElem;
use rustc_middle::mir::ConstantKind;
use rustc_middle::ty::TyKind;

// use crate::utils::print_mir;
use crate::stacked_borrows::{*};

pub struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    args: Vec<Operand<'tcx>>,
    local_declarations: LocalDecls<'tcx>,
    stacked_borrows: Stack,
}

// Basic Functions
impl<'tcx> MirVisitor<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, args: Vec<Operand<'tcx>>) -> Self {
        MirVisitor {
            tcx,
            args,
            local_declarations: LocalDecls::new(),
            stacked_borrows: Stack::new()
        }
    }
}

// Visitor trait implementation
impl<'tcx> Visitor<'tcx> for MirVisitor<'tcx> {
    fn visit_body(&mut self, body: &Body<'tcx>) {
        println!("Main body -- Start");
        // Visit local declarations
        let local_declarations = body.local_decls.clone();
        for (local, local_decl) in local_declarations.into_iter_enumerated() {
            self.visit_local_decl(local, &local_decl);
        }

        self.push_args();
        self.local_declarations = body.local_decls.clone();
        println!("\n");

        // Visit function basic blocks
        let basic_blocks = body.basic_blocks().clone();
        for (block, data) in basic_blocks.into_iter_enumerated() {
            self.visit_basic_block_data(block, &data);
        }
        println!("Main body -- End");
    }

    // Function Declarations

    fn visit_local_decl(
        &mut self,
        local: Local,
        local_decl: &LocalDecl<'tcx>
    ) {
        let _ty = local_decl.ty;
        let _mutability = local_decl.mutability;
        // println!("Declaration {:?} {:?}: {:?}", mutability, local, ty);
    }


    // Function Statements

    fn visit_basic_block_data(
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
        let tag = self.place_to_tag(place);

        match rvalue {
            // Move or copy operand (x)
            Use(operand) => {
                print!("use ");
                self.visit_operand(operand, location);
                self.add_to_stack(place, tag);
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
            },
            // Create a raw pointer (&raw const x)
            AddressOf(_mutability, place) => {
                print!("raw ");
                self.stacked_borrows.use_value(self.place_to_tag(place));
                self.stacked_borrows.new_ref(tag, Permission::SharedReadWrite);
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



    fn visit_operand(
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

    fn visit_terminator(
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

                for arg in &args {
                    self.visit_operand(arg, location);
                }

                if let Operand::Constant(boxed_constant) = &func {
                    let constant = *boxed_constant.clone();
                    if let ConstantKind::Ty(cnst) = constant.literal {
                        if cnst.ty.is_fn() {
                            // println!("const ty {:#?}", cnst.ty);
                            if let TyKind::FnDef(def_id, _) = cnst.ty.kind() {
                                let mut visitor = MirVisitor {
                                    tcx: self.tcx,
                                    args,
                                    local_declarations: LocalDecls::new(),
                                    stacked_borrows: Stack::new()
                                };
                                visitor.visit_body(self.tcx.optimized_mir(*def_id));
                            }
                        }
                    }
                }

                let (place, _) = destination.unwrap();
                let tag = self.place_to_tag(&place);
                if place.projection.is_empty() {
                    self.stacked_borrows.new_ref(tag, Permission::Unique);
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

// Stacked Borrows helper functions
impl<'tcx> MirVisitor<'tcx> {
    fn place_to_tag(&self, place: &Place) -> Tag {
        Tag::Tagged(place.local.as_u32())
    }

    fn add_to_stack(&mut self, place: &Place, tag: Tag) {
        if place.projection.is_empty() {
            self.stacked_borrows.new_ref(tag, Permission::Unique);
        } else {
            match place.projection[0] {
                ProjectionElem::Deref => {}
                _ => {self.stacked_borrows.new_ref(tag, Permission::Unique);}
            }
        }
        self.stacked_borrows.use_value(tag);
    }

    fn push_args(&mut self) {
        let mut index = 1;
        for _arg in &self.args {
            self.stacked_borrows.new_ref(Tag::Tagged(index), Permission::Unique);
            index += 1;
        }
    }
}