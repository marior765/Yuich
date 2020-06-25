// pub mod GC;
pub mod VM;

fn main() {
  let vm = VM::VM::init();
  vm.run();
}
