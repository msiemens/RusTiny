// LLVM:
//
// entry-block:
//   %a = alloca i8
//   store i8 2, i8* %a
//   br label %while_cond
//
// while_exit:                                       ; preds = %while_cond
//   ret void
//
// while_cond:                                       ; preds = %while_body, %entry-block
//   %0 = load i8* %a
//   %1 = icmp ugt i8 %0, 0
//   br i1 %1, label %while_body, label %while_exit
//
// while_body:                                       ; preds = %while_cond
//   %2 = load i8* %a
//   %3 = sub i8 %2, 1
//   store i8 %3, i8* %a
//   br label %while_cond

fn main() {
    let a: int = 2;

    while a > 0 {
        a -= 1;
    }
}