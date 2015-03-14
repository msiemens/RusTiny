fn foo(a: int) {
    let a: bool = false;  //! ERROR(2:9): cannot redeclare `a`
}

fn main() {}