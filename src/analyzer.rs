use crate::mir_visitor::body_visitor::{MirVisitor};
use rustc_middle::ty::{TyCtxt};
use petgraph::dot::{Dot, Config};


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
        let function_body = tcx.optimized_mir(entry_fn_id);
        let mut visitor = MirVisitor::new(tcx,&function_body, Vec::new());
        visitor.visit_body(function_body);

        println!("{:?}", Dot::with_config(&visitor.alias_graph.graph, &[Config::EdgeNoLabel]));
    }
}
