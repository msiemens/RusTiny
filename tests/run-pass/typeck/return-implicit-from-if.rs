//! SKIP
fn foo(a: int, b: bool, c: char) -> char {
    if a == 2 || b {
        c
    } else {
        'd'
    }
}

fn main() {
    foo(1, false, 'a');
}
