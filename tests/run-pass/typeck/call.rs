fn foo(a: int, b: bool, c: char) -> char {
    if a == 2 || b {
        return c;
    } else {
        return 'd';
    }
}

fn main() {
    foo(1, false, 'a');
}

//! ERROR: err