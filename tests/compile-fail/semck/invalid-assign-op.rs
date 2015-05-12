fn main() {
    let a: int = 0;
    a &&= 2;  //! ERROR(3:9): unexpected token: `=`, expected a prefix expression
}