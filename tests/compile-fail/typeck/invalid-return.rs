fn foo() -> int {
    return 'a';  //! ERROR(2:12): type mismatch: expected int, got char
}

fn main() {}