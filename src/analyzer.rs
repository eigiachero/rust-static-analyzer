use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::{BasicBlock, BasicBlockData, Body, Statement, Location, Terminator};
use rustc_middle::mir::{Local, LocalDecl, LocalDecls, Place, Rvalue};
// use rustc_middle::mir::Mutability::{Mut, Not};
use rustc_middle::mir::StatementKind::{Assign};
use rustc_middle::mir::Operand;
use rustc_middle::mir::Rvalue::{*};
use rustc_middle::mir::BorrowKind;
use rustc_middle::ty::{TyCtxt};
use rustc_middle::mir::terminator::TerminatorKind;
use rustc_middle::mir::ProjectionElem;
use rustc_middle::ty::WithOptConstParam;
use rustc_middle::mir::Field;
// use crate::utils::print_mir;
use crate::stacked_borrows::{*};

pub fn analyze<'tcx>(tcx: TyCtxt) {
    let entry_fn_id;
    match tcx.entry_fn(()) {
        Some((def_id, _fn_type)) => { entry_fn_id = def_id;}
        None => {
            println!("The program must have a main function");
            return;
        }
    }

    if tcx.is_mir_available(entry_fn_id) {
        // print_mir(tcx, entry_fn_id);
        let mut visitor = MirVisitor {
            tcx,
            local_declarations: LocalDecls::new(),
            stacked_borrows: Stack::new()
        };
        visitor.visit_body(tcx.optimized_mir(entry_fn_id));
        // visitor.visit_body(&tcx.mir_built(WithOptConstParam {
        //     did: entry_fn_id.as_local().unwrap(),
        //     const_param_did: Some(entry_fn_id),
        // }).steal());

    }
}

struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    local_declarations: LocalDecls<'tcx>,
    stacked_borrows: Stack,
}

impl<'tcx> Visitor<'tcx> for MirVisitor<'tcx> {
    fn visit_body(&mut self, body: &Body<'tcx>) {
        // println!("Body {:#?}", body);

        println!("Main body -- Start");
        let local_declarations = body.local_decls.clone();
        for (local, local_decl) in local_declarations.into_iter_enumerated() {
            self.visit_local_decl(local, &local_decl);
        }

        self.local_declarations = body.local_decls.clone();

        println!("\n");
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
        let ty = local_decl.ty;
        let mutability = local_decl.mutability;

        println!("Declaration {:?} {:?}: {:?}", mutability, local, ty);
    }


    // Function Statements

    fn visit_basic_block_data(
        &mut self,
        block: BasicBlock,
        data: &BasicBlockData<'tcx>
    ) {
        println!("Block {:#?} --Start", block);
        // println!("Data {:#?}", data);
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
        //println!("Statement {:#?} - {:#?}", statement, self.stacked_borrows);
    }

    fn visit_assign(
        &mut self,
        place: &Place<'tcx>,
        rvalue: &Rvalue<'tcx>,
        location: Location
    ) {
        let tag = Tag::Tagged(place.local.as_u32());

        match rvalue {
            Use(operand) => {
                print!("use ");
                self.visit_operand(operand, location);
                self.add_to_stack(place, tag);
            },
            Ref(_region, borrow_kind, place) => {
                print!("ref ");
                match borrow_kind {
                    BorrowKind::Shared => {
                        self.stacked_borrows.read_value(Tag::Tagged(place.local.as_u32()));
                        self.stacked_borrows.new_ref(tag, Permission::SharedReadOnly);
                    }
                    _ => {
                        self.stacked_borrows.use_value(Tag::Tagged(place.local.as_u32()));
                        self.stacked_borrows.new_ref(tag, Permission::Unique);
                    }
                };
            },
            AddressOf(_mutability, place) => {
                print!("raw ");
                self.stacked_borrows.use_value(Tag::Tagged(place.local.as_u32()));
                self.stacked_borrows.new_ref(tag, Permission::SharedReadWrite);
            }
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
            BinaryOp(_bin_op, box_tuple) => {
                let (operand1, operand2) = *box_tuple.clone();
                self.visit_operand(&operand1, location);
                self.visit_operand(&operand2, location);
                self.add_to_stack(place, tag);

            }
            CheckedBinaryOp(_bin_op, box_tuple) => {
                let (operand1, operand2) = *box_tuple.clone();
                self.visit_operand(&operand1, location);
                self.visit_operand(&operand2, location);
                self.add_to_stack(place, tag);

            }
            other => println!("Rvalue kind not recognized {:?} ", other),
        }

        println!("{:#?} Assign {:?} = {:?} | {:#?}", location, place, rvalue, self.stacked_borrows);
        // println!("{:#?} Assign {:?} = {:?}", location, place, rvalue);
    }



    fn visit_operand(
        &mut self,
        operand: &Operand<'tcx>,
        location: Location
    ) {
        match operand {
            Operand::Move(place) => {
                // println!("Move TAG: {:?}", place.local.as_u32());
                if !place.projection.is_empty() {
                    self.stacked_borrows.use_value(Tag::Tagged(place.local.as_u32()));
                }
            }
            Operand::Copy(place) => {
                // println!("Copy TAG: {:?}", place.local.as_u32());
                if !place.projection.is_empty() {
                    self.stacked_borrows.use_value(Tag::Tagged(place.local.as_u32()));
                }
            }
            Operand::Constant(_) => {}
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
                cleanup,
                from_hir_call,
                fn_span
            } => {
                for arg in args {
                    self.visit_operand(&arg, location);
                }

                let (place, _) = destination.unwrap();
                let tag = Tag::Tagged(place.local.as_u32());
                if place.projection.is_empty() {
                    self.stacked_borrows.new_ref(tag, Permission::Unique);
                }
                self.stacked_borrows.use_value(tag);


            },
            TerminatorKind::Assert {
                cond,
                expected,
                msg,
                target,
                cleanup,
            } => {
                self.visit_operand(&cond, location);
            },
            TerminatorKind::Return => {},
            _ => {
                println!("Terminator Kind not recognized");
            }
        }
        // println!("{:#?} Terminator {:#?} | {:#?}", location, terminator.kind, self.stacked_borrows);
        println!("{:#?} Terminator {:#?}", location, terminator.kind);
    }
}

impl<'tcx> MirVisitor<'tcx> {
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
}