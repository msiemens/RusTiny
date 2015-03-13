fn foo() {
    1 = false;  //! ERROR(2:5): left-hand side of assignment is not a variable
}

fn main() {}