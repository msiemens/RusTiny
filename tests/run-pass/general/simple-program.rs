const CONST: int = 0;
static STATIC: int = 0;

fn mul(a: int, b: int) -> int {
    let result: int = 0;

    while b > 0 {
        result += if true { 1 } else { 0 };
        a -= 1 * (2 + 2);
        b -= 1;
    }

    return result;
}

fn main() {
    mul(3, 5);
}