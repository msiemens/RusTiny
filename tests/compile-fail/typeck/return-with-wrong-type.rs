fn foo() -> int {
    return;  //! ERROR(2:11): type mismatch: expected int, got ()
}

fn main() {}