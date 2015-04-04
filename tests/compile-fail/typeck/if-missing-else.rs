fn foo() -> int {
    let a: int = 2;

    //! ERROR(5:26): missing else clause
    return if a + 3 == 7 {
        a
    };
}


fn main() {
    let a: int = foo();
}