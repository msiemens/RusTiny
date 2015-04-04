// LLVM:
//
//  %a = alloca i8
//  store i8 2, i8* %a
//  %0 = load i8* %a
//  %1 = add i8 %0, 2
//  ret void

fn main() {
    let a: int = 2;
    a + 2;
}
