fn main() {
    let a: bool = false;

    a += 2;  //! ERROR(4:5): type mismatch: expected int, got bool
}