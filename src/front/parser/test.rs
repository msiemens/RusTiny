#![cfg(test)]

use ast::*;
use front::{Lexer, Parser};


macro_rules! parser(
    ($source:expr) => (
        Parser::new(Lexer::new($source, "<test>"))
    )
);

macro_rules! ast_assert(
    ($pat:path { $($v:ident),* } == $cmp:expr) => (
        if let $pat { $(ref $v),* } = $cmp {
            ($($v),*)
        } else {
            panic!("Expected a {}, got ???", stringify!($pat));
        }
    )
);


#[test]
fn operator_precedence_call_prefix() {
    let ast = parser!("!a()").parse_expression();

    let (op, item) = ast_assert!(Expression::Prefix { op, item } == *ast);
    assert_eq!(*op, UnOp::Not);

    let (func, args) = ast_assert!(Expression::Call { func, args } == ***item);
    assert_eq!(args.len(), 0);

    let name = ast_assert!(Expression::Variable { name } == ***func);
    assert_eq!(&**name, "a");
}


#[test]
fn operator_precedence_prefix_exponent() {
    let ast = parser!("-1 ** 2").parse_expression();

    let (op, item) = ast_assert!(Expression::Prefix { op, item } == *ast);
    assert_eq!(*op, UnOp::Neg);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***item);
    assert_eq!(*op, BinOp::Pow);

}


#[test]
fn operator_precedence_exponent_product() {
    let ast = parser!("1 * 2 ** 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::Mul);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::Pow);
}


#[test]
fn operator_precedence_product_sum() {
    let ast = parser!("1 + 2 * 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::Add);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::Mul);
}


#[test]
fn operator_precedence_sum_shift() {
    let ast = parser!("1 + 2 << 3").parse_expression();

    let (op, lhs, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::Shl);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***lhs);
    assert_eq!(*op, BinOp::Add);
}


#[test]
fn operator_precedence_shift_bitand() {
    let ast = parser!("1 & 2 << 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::BitAnd);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::Shl);
}


#[test]
fn operator_precedence_bitand_bitxor() {
    let ast = parser!("1 ^ 2 & 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::BitXor);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::BitAnd);
}


#[test]
fn operator_precedence_bitxor_bitor() {
    let ast = parser!("1 | 2 ^ 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::BitOr);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::BitXor);
}


#[test]
fn operator_precedence_bitor_compare() {
    let ast = parser!("1 == 2 | 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::EqEq);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::BitOr);
}


#[test]
fn operator_precedence_compare_and() {
    let ast = parser!("1 && 2 == 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::And);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::EqEq);
}


#[test]
fn operator_precedence_and_or() {
    let ast = parser!("1 || 2 && 3").parse_expression();

    let (op, _, rhs) = ast_assert!(Expression::Infix { op, lhs, rhs } == *ast);
    assert_eq!(*op, BinOp::Or);

    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::And);
}


#[test]
fn operator_precedence_or_assignment() {
    let ast = parser!("a = 2 || 3").parse_expression();

    let (_, rhs) = ast_assert!(Expression::Assign { lhs, rhs } == *ast);
    let (op, _, _) = ast_assert!(Expression::Infix { op, lhs, rhs } == ***rhs);
    assert_eq!(*op, BinOp::Or);
}