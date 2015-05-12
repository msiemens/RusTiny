fn a() -> int {
    return 0 + + 1  //! ERROR(2:16): unexpected token: `+`, expected a prefix expression
}