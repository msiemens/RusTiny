fn foo() {}


fn main() {
    foo(1, 2, 3);  //! ERROR(5:5): mismatching argument count: expected 0, got 3
}