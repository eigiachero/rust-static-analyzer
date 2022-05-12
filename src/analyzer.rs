use rustc_middle::mir::visit::Visitor;
use rustc_middle::ty::{TyCtxt};

// use crate::utils::print_mir;
use crate::mir_visitor::{MirVisitor};

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

        let mut visitor = MirVisitor::new(tcx, Vec::new());
        visitor.visit_body(tcx.optimized_mir(entry_fn_id));

        // visitor.visit_body(&tcx.mir_built(WithOptConstParam {
        //     did: entry_fn_id.as_local().unwrap(),
        //     const_param_did: Some(entry_fn_id),
        // }).steal());
    }
}
