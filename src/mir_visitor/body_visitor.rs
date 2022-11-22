use std::collections::HashMap;

use rustc_middle::mir::{Local, LocalDecl, LocalDecls, Body};
use rustc_middle::mir::{Operand, VarDebugInfoContents};
use rustc_middle::ty::{TyCtxt};

use crate::stacked_borrows::{*};
use crate::points_to::PointsToGraph;

pub struct MirVisitor<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub body: &'tcx Body<'tcx>,
    pub args: Vec<Operand<'tcx>>,
    pub func_name: String,
    pub local_declarations: LocalDecls<'tcx>,
    pub variable_names: HashMap<u32, String>,
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
            func_name: String::new(),
            local_declarations: LocalDecls::new(),
            variable_names: HashMap::new(),
            stacked_borrows: Stack::new(),
            alias_graph: PointsToGraph::new()
        }
    }
}

// Visitor trait implementation
impl<'tcx> MirVisitor<'tcx> {
    pub fn visit_body(&mut self, body: &Body<'tcx>) {
        let name = MirVisitor::<'tcx>::get_body_func_name(body);
        self.func_name = name;
        println!("\n{} body -- Start\n", self.func_name);

        // Create a hashmap with variable real names
        for variable in &self.body.var_debug_info {
            if let VarDebugInfoContents::Place(val) = variable.value {
                self.variable_names.insert(val.local.as_u32(), String::from(variable.name.as_str()));
            }
        }
        self.stacked_borrows.names = self.variable_names.clone();

        // Visit local declarations
        let local_declarations = body.local_decls.clone();
        for (local, local_decl) in local_declarations.into_iter_enumerated() {
            self.visit_local_decl(local, &local_decl);
        }


        // Visit arguments and local declarations
        self.push_args();
        self.local_declarations = body.local_decls.clone();

        // Visit function basic blocks
        let basic_blocks = body.basic_blocks().clone();
        for (block, data) in basic_blocks.into_iter_enumerated() {
            self.visit_basic_block_data(block, &data);
        }
        println!("{} body -- End", self.func_name);
    }

    // Function Declarations

    fn visit_local_decl(
        &mut self,
        local: Local,
        local_decl: &LocalDecl<'tcx>
    ) {
        let _ty = local_decl.ty;
        let _mutability = local_decl.mutability;

        // println!("Declaration {:?} {}: {:?}", _mutability, self.get_variable_name(local.as_u32()), _ty);
    }
}
