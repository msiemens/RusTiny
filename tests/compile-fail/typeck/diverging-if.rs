fn main() {
    let a: int = if 2 == 0 {
        0
    } else {};  //! ERROR(4:13): type mismatch: expected int, got ()
}