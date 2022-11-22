use rustc_middle::mir::{Place, Body};
use rustc_middle::mir::Operand;
use rustc_middle::mir::Mutability::Mut;
use std::fmt::Write as FmtWrite;

// use crate::utils::print_mir;
use crate::stacked_borrows::{*};
use super::body_visitor::MirVisitor;

impl<'tcx> MirVisitor<'tcx> {
    // Stacked Borrows helper functions
    pub fn place_to_tag(&self, place: &Place) -> Tag {
        Tag::Tagged(place.local.as_u32())
    }

    pub fn add_to_stack(&mut self, place: &Place) {
        let tag = self.place_to_tag(place);
        if !place.is_indirect() { // is not a (&x)
            let variable_decl = self.local_declarations.get(place.local).unwrap();
            let is_mutable = variable_decl.mutability == Mut;
            let is_mut_ref = variable_decl.ty.is_mutable_ptr();
            if is_mutable || is_mut_ref {
                self.stacked_borrows.new_ref(tag, Permission::Unique);
            } else {
                self.stacked_borrows.new_ref(tag, Permission::SharedReadOnly);
            }
        }
        self.stacked_borrows.use_value(tag);
    }

    pub fn is_raw_ptr(&self, place: &Place) -> bool {
        let variable_decl = self.local_declarations.get(place.local).unwrap();
        variable_decl.ty.is_unsafe_ptr()
    }

    pub fn push_args(&mut self) {
        let mut index = 1;
        for _arg in &self.args {
            self.stacked_borrows.new_ref(Tag::Tagged(index), Permission::Unique);
            self.alias_graph.constant(index);
            index += 1;
        }
    }

    // Points-to analisis helper functions
    pub fn is_mutable(&self, operand: &mut Operand) -> bool {
        match operand {
            Operand::Move(place) | Operand::Copy(place) => {
                let local_decl = self.local_declarations.get(place.local).unwrap();
                local_decl.mutability == Mut
            }
            Operand::Constant(boxed_constant) => {
                false
            }
        }
    }

    pub fn operand_as_u32(&self, operand: &Operand) -> u32 {
        match operand {
            Operand::Move(place) | Operand::Copy(place) => {
                place.local.as_u32()
            }
            Operand::Constant(boxed_constant) => {
                0
            }
        }
    }

    // Debugger help functions
    pub fn get_variable_name(&self, place: u32) -> String {
        let name = match self.variable_names.get(&place) {
            Some(name) => name.clone(),
            None => place.to_string()

        };
        name
    }

    pub fn get_body_func_name(body: &Body) -> String {
        let mut out = String::new();
        write!(&mut out, "{:?}", body.source.instance.def_id()).unwrap();
        let mut name: String = String::from(out.split("::").collect::<Vec<&str>>()[1]);
        name = name[0..1].to_uppercase() + &name[1..name.len()-1];
        name
    }
}