// LLVM:
//
//  %a = alloca i32
//  store i32 1, i32* %a
//  %0 = load i32* %a
//  %1 = mul i32 %0, 5
//  %2 = sub i32 %1, 2
//  ret void

fn main() {
    let a: int = 1;
    a * 5 - 2;
}
