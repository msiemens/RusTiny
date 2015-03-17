fn main() {
    let a: int = 2 + false;  //! ERROR(2:22): type mismatch: expected int, got bool
    let b: bool = false && 2;  //! ERROR(3:28): type mismatch: expected bool, got int
    let c: bool = 2 <= 'a';  //! ERROR(4:24): type mismatch: expected int, got char
    let d: int = 2 ^ false;  //! ERROR(5:22): type mismatch: expected int, got bool
    let e: bool = false ^ 2;  //! ERROR(6:27): type mismatch: expected bool, got int
    let f: char = 'a' ^ 'b';  //! ERROR(7:19): binary operation `^` cannot be applied to char
}