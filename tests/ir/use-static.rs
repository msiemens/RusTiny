// LLVM:
//
//  %0 = load i8* @_ZN6STATIC20h51bceb85c9b03c28eaaE
//  %1 = add i8 %0, 2
//  ret void

static STATIC: int = 3;

fn main() {
    STATIC + 2;
}
