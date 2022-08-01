use crate::mir_visitor::body_visitor::MirVisitor;
use petgraph::dot::{Config, Dot};
use rustc_middle::ty::TyCtxt;

pub fn analyze<'tcx>(tcx: TyCtxt, main_function_name: Option<String>) {
    let entry_fn_id = match main_function_name {
        Some(name) => {
            let mut def_id = None;
            for item in tcx.hir().items() {
                if let rustc_hir::ItemKind::Fn(_sign, _gen, body_id) = &item.kind {
                    if name == item.ident.name.as_str() {
                        def_id = Some(item.def_id.to_def_id());
                        break;
                    }
                }
            }
            match def_id {
                Some(id) => id,
                None => {
                    println!("Function name not found, using default entry function");
                    tcx.entry_fn(()).unwrap().0 // default
                }
            }
        }
        None => tcx.entry_fn(()).unwrap().0, // default
    };

    if tcx.is_mir_available(entry_fn_id) {
        let function_body = tcx.optimized_mir(entry_fn_id);
        let mut visitor = MirVisitor::new(tcx, &function_body, Vec::new());
        visitor.visit_body(function_body);

        println!(
            "{:?}",
            Dot::with_config(&visitor.alias_graph.graph, &[Config::EdgeNoLabel])
        );
    }
}
