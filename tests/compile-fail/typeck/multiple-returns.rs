fn foo() -> int {
    return 1;
    return 'a';  //! ERROR(3:12): type mismatch: expected int, got char
}

fn main() {}