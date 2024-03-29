use std::{fmt, collections::{VecDeque, HashMap}};

pub struct Stack {
    borrows: VecDeque<StackItem>,
    pub names: HashMap<u32, String>,
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
        Stack { borrows: VecDeque::new(), names: HashMap::new() }
    }

    pub fn clean(&mut self) {
        self.borrows.clear();
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
                    println!("ERROR Tag {:?} does not have WRITE access", self.get_tag_name(tag));
                    break;
                }

            }
        }
    }

    pub fn use_raw(&mut self, tag: Tag) {
        if self.borrows.contains(&StackItem { tag, permission: Permission::SharedReadWrite }) {
            loop  {
                let last_item = self.borrows.pop_front();
                match last_item {
                    Some(item) => {
                        if item.permission == Permission::SharedReadWrite {
                            self.borrows.push_front(item);
                            break;
                        }
                    }
                    None => {
                        println!("ERROR Tag {:?} does not have WRITE access", self.get_tag_name(tag));
                        break;
                    }

                }
            }
        } else {
            println!("ERROR Tag {:?} does not have WRITE access", self.get_tag_name(tag));
        }
    }

    pub fn read_raw(&mut self, tag: Tag) {
        if !self.borrows.contains(&StackItem { tag, permission: Permission::SharedReadWrite }) {
            println!("ERROR Tag {:?} does not have WRITE access", self.get_tag_name(tag));
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
        println!("ERROR Tag {:?} does not have READ access", self.get_tag_name(tag));
    }

    pub fn is_live(&self, tag: Tag) -> bool {
        let mut result = false;
        for item in &self.borrows {
            if item.tag == tag {
                result = true;
                break;
            }
        }
        result
    }

    pub fn get_tag_name(&self, tag: Tag) -> String {
        let mut name = format!("{:?}", tag);
        if let Tag::Tagged(id) = tag {
            if let Some(x) = self.names.get(&id){
                name = x.clone();
            }
        }
        name
    }
}

impl StackItem {
    pub fn new(tag: Tag, permission: Permission) -> StackItem {
        StackItem { tag, permission }
    }
}
