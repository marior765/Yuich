pub mod VM;

fn main() {
  let vm = VM::VM::init();
  vm.run();
}
