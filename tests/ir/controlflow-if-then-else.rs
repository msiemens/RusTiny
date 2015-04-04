// LLVM:
// define internal i8 @bar() unnamed_addr #0 {
// entry-block:
//   ret i8 2
// }
//
// ; Function Attrs: uwtable
// define internal i8 @foo() unnamed_addr #0 {
// entry-block:
//   %a = alloca i8
//   %0 = alloca i8
//   %1 = call i8 @bar()
//   store i8 %1, i8* %0
//   %2 = load i8* %0
//   %3 = icmp eq i8 %2, 2
//   br i1 %3, label %then-block-25-, label %else-block
//
// then-block-25-:
//   store i8 5, i8* %a
//   br label %join
//
// else-block:
//   store i8 7, i8* %a
//   br label %join
//
// join:
//   %4 = load i8* %a
//   br label %return
//
// return:
//   ret i8 %4
// }

fn bar() -> int {
    2
}

fn foo() -> int {
    let a: int = if bar() == 2 {
        5
    } else {
        7
    };

    return a;
}


fn main() {
    let a: int = foo();
}