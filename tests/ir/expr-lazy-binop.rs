// LLVM:
//
// define internal i8 @foo() unnamed_addr #0 {
// entry-block:
//   br label %return
//
// ; preds = %entry-block
// return:
//   ret i8 2
// }
//
// define internal void @main() unnamed_addr #0 {
// entry-block:
//   %a = alloca i8
//   %0 = alloca i8
//   %1 = alloca i8
//   %2 = call i8 @foo()
//   store i8 %2, i8* %0
//   %3 = load i8* %0
//   %4 = icmp eq i8 5, %3
//   br i1 %4, label %join, label %before_rhs
//
// ; preds = %before_rhs, %entry-block
// join:
//   %5 = phi i1 [ %4, %entry-block ], [ %10, %before_rhs ]
//   %6 = zext i1 %5 to i8
//   store i8 %6, i8* %a
//   ret void
//
// ; preds = %entry-block
// before_rhs:
//   %7 = call i8 @foo()
//   store i8 %7, i8* %1
//   %8 = load i8* %1
//   %9 = add i8 %8, 3
//   %10 = icmp ne i8 %9, 7
//   br label %join
// }

fn foo() -> int {
    return 2;
}

fn main() {
    let a: bool = (5 == foo() || (foo() + 3) != 7);
}