use std::{fmt, collections::VecDeque};

pub struct Stack {
    borrows: VecDeque<StackItem>
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for item in &self.borrows {
            if item == *&self.borrows.back().unwrap() {
                write!(f, "{:?}", item)?;
            } else {
                write!(f, "{:?}, ", item)?;
            }

        }
        write!(f, "]")?;
        Ok(())
    }
}


#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct StackItem {
    tag: Tag,
    permission: Permission,
}

impl fmt::Debug for StackItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}", self.permission, self.tag)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Permission {
    Unique, // Grants unique mutable access.
    SharedReadWrite, // Grants shared mutable access.
    SharedReadOnly, // Grants shared read-only access.
}
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Tag {
    Tagged(PtrId),
    Untagged,
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Tagged(id) => write!(f, "<{}>", id),
            Tag::Untagged => write!(f, "<untagged>"),
        }
    }
}

pub type PtrId = u32;

impl Stack {
    pub fn new() -> Stack {
        Stack { borrows: VecDeque::new() }
    }

    pub fn new_ref(&mut self, tag: Tag, permission: Permission) {
        let new_item = StackItem::new(tag, permission);
        if self.borrows.contains(&new_item) {
            self.use_value(tag);
            return;
        }
        self.borrows.push_front(new_item);
    }

    pub fn use_value(&mut self, tag: Tag) {
        loop  {
            let last_item = self.borrows.pop_front();
            match last_item {
                Some(item) => {
                    if item.tag == tag {
                        self.borrows.push_front(item);
                        break;
                    }
                }
                None => {
                    println!("ERROR Tag {:?} does not have write access ERROR", tag);
                    break;
                }

            }
        }
    }

    pub fn read_value(&mut self, tag: Tag) {
        let mut index = 0;
        for item in self.borrows.clone().into_iter(){
            if item.tag == tag {
                return;
            }
            if item.permission == Permission::Unique {
                self.borrows.remove(index);
            }
            index += 0;
        }
        println!("ERROR Tag {:?} does not have READ access ERROR", tag);
    }
}

impl StackItem {
    pub fn new(tag: Tag, permission: Permission) -> StackItem {
        StackItem { tag, permission }
    }
}
