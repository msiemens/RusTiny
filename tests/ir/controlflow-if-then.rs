// LLVM:
//
// entry-block:
//   %a = alloca i8
//   store i8 2, i8* %a
//   %0 = load i8* %a
//   %1 = icmp eq i8 %0, 2
//   br i1 %1, label %then-block-21-, label %next-block
//
// then-block-21-:
//   %2 = load i8* %a
//   %3 = add i8 %2, 3
//   store i8 %3, i8* %a
//   br label %next-block
//
// next-block:
//   %4 = load i8* %a
//   br label %return
//
// return:
//   ret i8 %4

fn foo() -> int {
    let a: int = 2;

    if a == 2 {
        a += 3;
    };

    return a;
}


fn main() {
    let a: int = foo();
}