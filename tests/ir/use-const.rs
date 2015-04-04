// LLVM:
//
//  %a = alloca i8
//  store i8 0, i8* %a
//  %0 = load i8* %a
//  %1 = add i8 %0, 3
//  ret void

const CONST: int = 3;

fn main() {
    let a: int = 0;
    a + CONST;
}
