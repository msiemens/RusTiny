fn foo() -> int {
    let a: int = 2;

    let b: int = if a + 3 == 7 {
        a
    } else {
        2
    };

    return b;
}


fn main() {
    let a: int = foo();
}