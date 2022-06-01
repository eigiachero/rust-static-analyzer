use rustc_middle::mir::{Place};
use rustc_middle::mir::Operand;
use rustc_middle::mir::ProjectionElem;
use rustc_middle::mir::Mutability::Mut;

// use crate::utils::print_mir;
use crate::stacked_borrows::{*};
use super::body_visitor::MirVisitor;

// Stacked Borrows helper functions
impl<'tcx> MirVisitor<'tcx> {
    pub fn place_to_tag(&self, place: &Place) -> Tag {
        Tag::Tagged(place.local.as_u32())
    }

    pub fn add_to_stack(&mut self, place: &Place, tag: Tag) {
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

    pub fn push_args(&mut self) {
        let mut index = 1;
        for _arg in &self.args {
            self.stacked_borrows.new_ref(Tag::Tagged(index), Permission::Unique);
            index += 1;
        }
    }

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
}