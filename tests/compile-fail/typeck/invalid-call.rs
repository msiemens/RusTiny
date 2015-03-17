fn foo(a: int) {}

fn main() {
    foo(false)  //! ERROR(4:9): type mismatch: expected int, got bool
}