#![cfg(test)]

use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

#[test]
fn test_print_statement() {
    let input = "print 10 + 12".to_string();
    let mut scanner = Scanner::new(input);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens.to_vec());
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(statements);

    assert_eq!(result, Some(()));
}

#[test]
fn test_runtime_error() {
    let input = "print -false".to_string();
    let mut scanner = Scanner::new(input);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens.to_vec());
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(statements);

    assert_eq!(result, None);
}

#[test]
fn test_block_statements() {
    let input = "
        var x = 45;
        {
            var y = 45;
            print x + y;
        }
        print x;
        print y;
    ".to_string();

    let mut scanner = Scanner::new(input);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens.to_vec());
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(statements);

    assert_eq!(result, None);
}
