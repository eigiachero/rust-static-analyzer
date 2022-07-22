use rustc_middle::mir::{Local, LocalDecl, LocalDecls, Body};
use rustc_middle::mir::Operand;
use rustc_middle::ty::{TyCtxt};

use crate::stacked_borrows::{*};
use crate::points_to::PointsToGraph;

pub struct MirVisitor<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub body: &'tcx Body<'tcx>,
    pub args: Vec<Operand<'tcx>>,
    pub local_declarations: LocalDecls<'tcx>,
    pub stacked_borrows: Stack,
    pub alias_graph: PointsToGraph,
}

// Basic Functions
impl<'tcx> MirVisitor<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, body:&'tcx Body<'tcx>, args: Vec<Operand<'tcx>>) -> Self {
        MirVisitor {
            tcx,
            body,
            args,
            local_declarations: LocalDecls::new(),
            stacked_borrows: Stack::new(),
            alias_graph: PointsToGraph::new()
        }
    }
}

// Visitor trait implementation
impl<'tcx> MirVisitor<'tcx> {
    pub fn visit_body(&mut self, body: &Body<'tcx>) {
        println!("Main body -- Start");
        // Visit local declarations
        let local_declarations = body.local_decls.clone();
        for (local, local_decl) in local_declarations.into_iter_enumerated() {
            self.visit_local_decl(local, &local_decl);
        }

        // Visit arguments and local declarations
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
        println!("Declaration {:?} {:?}: {:?}", _mutability, local, _ty);
    }
}
