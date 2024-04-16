#![cfg(test)]

use crate::{parser::{Expr, Parser, Stmt}, scanner::Scanner, token::{Token, TokenType}};

#[test]
fn test_binary_expr() {
    let input = "10 + 12".to_string();
    let mut parser = get_parser(input);
    let expected: Vec<Stmt> = vec![
        Stmt::Expression(
            Box::new(
                Expr::Binary(
                    Box::new(Expr::Literal("10".to_string())),
                    Token::new(TokenType::Plus, "+".to_string(), 1),
                    Box::new(Expr::Literal("12".to_string()))
                    )
                )
            )
    ];

    assert_eq!(parser.parse().unwrap(), expected);
}

#[test]
fn test_literal_expr() {
    let input = "10".to_string();
    let mut parser = get_parser(input);
    let expected: Vec<Stmt> = vec![
        Stmt::Expression(Box::new(Expr::Literal("10".to_string())))
    ];

    assert_eq!(parser.parse().unwrap(), expected);
}

#[test]
fn test_grouping_expr() {
    let input = "(10)".to_string();
    let mut parser = get_parser(input);
    let expected: Vec<Stmt> = vec![
        Stmt::Expression(Box::new(Expr::Grouping(Box::new(Expr::Literal("10".to_string())))))
    ];

    assert_eq!(parser.parse().unwrap(), expected);
}

#[test]
fn test_unary_expr() {
    let input = "!true".to_string();
    let mut parser = get_parser(input);
    let expected: Vec<Stmt> = vec![
        Stmt::Expression(
            Box::new(
                Expr::Unary(
                    Token::new(TokenType::Bang, "!".to_string(), 1),
                    Box::new(Expr::Literal("true".to_string()))
                    )
                )
            )
    ];

    assert_eq!(parser.parse().unwrap(), expected);
}

#[test]
fn test_operator_precedence() {
    let input1 = "(10 + 12) / 2";
    let input2 = "10 + 12 / 2";
    let mut parser1 = get_parser(input1.to_string());
    let mut parser2 = get_parser(input2.to_string());

    assert_ne!(parser1.parse(), parser2.parse());
}

#[test]
fn test_ternary() {
    let input = "true ? 1 : 2".to_string();
    let mut parser = get_parser(input);

    let expected: Vec<Stmt> = vec![
        Stmt::Expression(
            Box::new(Expr::Ternary(
                Box::new(Expr::Literal("true".to_string())),
                Box::new(Expr::Literal("1".to_string())),
                Box::new(Expr::Literal("2".to_string())),
            ))
        )
    ];

    assert_eq!(parser.parse().unwrap(), expected);
}

fn get_parser(input: String) -> Parser {
    let mut scanner = Scanner::new(input);
    let tokens = scanner.scan_tokens();
    if let None = tokens {
        panic!("tokens are none.");
    }

    Parser::new(tokens.unwrap().to_vec())
}
