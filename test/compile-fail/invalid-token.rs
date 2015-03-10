fn a() -> int {
    return 0 ++ 1  //! ERROR(2:15): unexpected token: `+`, expected a prefix expression
}