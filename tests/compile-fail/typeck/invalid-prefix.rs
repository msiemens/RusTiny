fn main() {
    let a: char = 'a';
    let b: bool = !a;  //! ERROR(3:20): unary operation `!` cannot be applied to char
}