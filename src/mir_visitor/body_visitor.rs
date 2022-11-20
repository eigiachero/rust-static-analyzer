use std::fmt::write;

use petgraph::visit::EdgeRef;
use rustc_middle::mir::{Local, LocalDecl, LocalDecls, Body};
use rustc_middle::mir::Operand;
use rustc_middle::ty::{TyCtxt};
use std::fmt::Write as FmtWrite;

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
        // Get function name
        let mut out = String::new();
        write!(&mut out, "{:?}", body.source.instance.def_id()).unwrap();
        let mut name: String = String::from(out.split("::").collect::<Vec<&str>>()[1]);
        name = name[0..1].to_uppercase() + &name[1..name.len()-1];

        println!("\n{} body -- Start\n", name);
        // Visit local declarations
        let local_declarations = body.local_decls.clone();
        for (local, local_decl) in local_declarations.into_iter_enumerated() {
            self.visit_local_decl(local, &local_decl);
        }
        //println!("{:?}",body.var_debug_info);

        // Visit arguments and local declarations
        self.push_args();
        self.local_declarations = body.local_decls.clone();

        // Visit function basic blocks
        let basic_blocks = body.basic_blocks().clone();
        for (block, data) in basic_blocks.into_iter_enumerated() {
            self.visit_basic_block_data(block, &data);
        }
        println!("{} body -- End", name);
    }

    // Function Declarations

    fn visit_local_decl(
        &mut self,
        local: Local,
        local_decl: &LocalDecl<'tcx>
    ) {
        let _ty = local_decl.ty;
        let _mutability = local_decl.mutability;

        if self.args.is_empty() { // Create unknown args
            self.stacked_borrows.new_ref(Tag::Tagged(local.as_u32()), Permission::Unique);
            self.alias_graph.constant(local.as_u32());
        }

        //println!("Declaration {:?} {:?}: {:?}\n", _mutability, local, _ty);
    }
}
