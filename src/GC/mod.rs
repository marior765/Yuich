// use super::VM::VM;
use std::rc::Rc;

const STACK_SIZE: usize = 256;

enum ObjectType {
  INT,
  PAIR,
}

struct Object {
  obj_type: ObjectType,
  value: u16,
  head: Rc<Object>,
  tail: Rc<Object>,
}

// impl Object {
//   pub fn new(obj_type: ObjectType) -> Self {
//     Object {
//       obj_type
//     }
//   }
// }

struct VM {
  stack: [Object; STACK_SIZE],
  stack_size: i16,
}

impl VM {
  pub fn new_obj(&self, obj_type: ObjectType) -> Object {
    Object {
      obj_type,
      value: 0,
      head: Rc::new(self.pop()),
      tail: Rc::new(self.pop()),
    }
  }

  pub fn init(&self) -> Self {
    VM {
      stack: [self.new_obj(ObjectType::INT); STACK_SIZE],
      stack_size: 0,
    }
  }

  pub fn push(&self, value: Object) {
    self.stack_size += 1;
    self.stack[self.stack_size as usize] = value;
  }
  pub fn pop(&self) -> Object {
    self.stack[--self.stack_size as usize]
  }
}

// struct Pair {
//   head: Box<Object>,
//   tail: Box<Object>,
// }

// enum InnerUnion {
//   INT(i32),
//   PAIR(Pair),
// }

// struct Object {
//   obj_type: ObjectType,
//   inner: InnerUnion,
// }
