fn main() {
    let a: int = if 2 == 0 {
        false  //! ERROR(3:9): type mismatch: expected int, got bool
    };
}