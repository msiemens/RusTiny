// LLVM:
//
//  %a = alloca i32
//  store i32 1, i32* %a
//  %0 = load i32* %a
//  %1 = add i32 %0, 2
//  store i32 %1, i32* %a
//  ret void

fn main() {
    let a: int = 1;
    a += 2;
}
