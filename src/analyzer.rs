use log::{info, error};
use rustc_middle::ty::{ TyCtxt };
use rustc_middle::mir::pretty::write_mir_pretty;
use rustc_hir::def_id::DefId;

pub fn analyze<'tcx>(tcx: TyCtxt) {
    for item in tcx.hir().items() {
        match item.kind {
            rustc_hir::ItemKind::Fn(_, _, _) => {
                print_mir(tcx, item.def_id.to_def_id());
            }
            _ => ()
        }
    }
}

pub fn print_mir<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) {
    println!("Printing MIR for {:?}", def_id);

    if tcx.is_mir_available(def_id) {
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        if let Err(_) = write_mir_pretty(tcx, Some(def_id), &mut handle) {
            error!(
                "Cannot print MIR: error while printing `{:?}`",
                def_id
            );
        }
    } else {
        info!("Cannot print MIR: no MIR for `{:?}`", def_id);
    }
}
