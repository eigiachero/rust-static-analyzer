use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::{BasicBlock, BasicBlockData, Body, Statement, Location, Terminator, Local, LocalDecl, Operand, TerminatorKind};
use rustc_middle::ty::{TyCtxt};
use crate::utils::print_mir;

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
        };
        visitor.visit_body(tcx.optimized_mir(entry_fn_id));

    }
}

struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> Visitor<'tcx> for MirVisitor<'tcx> {
    fn visit_body(&mut self, body: &Body<'tcx>) {
        // println!("Body {:#?}", body);

        println!("Main body -- Start");
        let local_declarations = body.local_decls.clone();
        for (local, local_decl) in local_declarations.into_iter_enumerated() {
            self.visit_local_decl(local, &local_decl);
        }

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
        println!("Statement {:#?} - location {:#?}", statement, location);
    }

    fn visit_terminator(
        &mut self,
        terminator: &Terminator<'tcx>,
        location: Location
    ) {
        println!("Terminator {:#?} - location {:#?}", terminator.kind, location);
    }
}
