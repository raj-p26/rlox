use std::fmt;

use crate::{token::{ Token, TokenType }, Lox};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(String),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary(left, op, right) => {
                write!(f, "({} {} {})", *left, op.lexeme, *right)
            },
            Expr::Grouping(expression) => {
                write!(f, "(group {})", *expression)
            },
            Expr::Literal(lit) => {
                write!(f, "{}", lit)
            },
            Expr::Unary(op, right) => {
                write!(f, "({} {})", op.lexeme, *right)
            },
            Expr::Ternary(cond, left, right) => {
                write!(f, "({} ? {} : {})", *cond, *left, *right)
            },
            Expr::Variable(var) => {
                write!(f, "{}", var.lexeme)
            },
            Expr::Assign(name, expr) => {
                write!(f, "({} = {})", name.lexeme, *expr)
            },
            _ => todo!()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Let(Token, Option<Box<Expr>>),
    Block(Vec<Box<Stmt>>),
    If(Expr, Box<Stmt>, Box<Option<Stmt>>),
    While(Expr, Box<Stmt>),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => {
                write!(f, "{}", *expr)
            },
            Stmt::Print(expr) => {
                write!(f, "print({})", *expr)
            },
            Stmt::Let(name, expr) => {
                if let None = expr {
                    write!(f, "var {};", name.lexeme)
                } else {
                    write!(f, "var {} = {};", name.lexeme, (expr.as_ref()).unwrap())
                }
            },
            Stmt::Block(stmts) => {
                write!(f, "{{\n").unwrap();
                for stmt in stmts {
                    write!(f,"\t{}\n", *stmt).unwrap();
                }

                write!(f, "}}")
            },
            Stmt::If(cond, consequence, alternative) => {
                write!(f, "if ({}) {{\n\t", cond).unwrap();
                write!(f, "{}\n}}", consequence).unwrap();
                if (*alternative).is_some() {
                write!(f, " else {{\n\t{}", alternative.as_ref().as_ref().unwrap()).unwrap();
                }
                write!(f, "\n}}")
            },
            _ => todo!()
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let stmt = self.declaration();
            if let None = stmt {
                return None;
            }
            statements.push(stmt.unwrap());
        }
        return Some(statements);
    }

    fn declaration(&mut self) -> Option<Stmt> {
        // if self.match_tokens(&[TokenType::Fun]) {
        //     return self.function("function");
        // }
        if self.match_tokens(&[TokenType::Var]) {
            return self.var_declaration();
        }

        let stmt = self.statement();
        if let None = stmt {
            self.synchronize();
            return None;
        }
        return Some(stmt.unwrap());
    }

    fn block(&mut self) -> Option<Vec<Box<Stmt>>> {
        let mut statements: Vec<Box<Stmt>> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let declaration = self.declaration();
            if let None = declaration {
                return None;
            }
            statements.push(Box::new(declaration.unwrap()));
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.".to_string());

        return Some(statements);
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        let ident = self.consume(TokenType::Identifier, "Expect identifier after 'var'".to_string());
        if let None = ident {
            return None;
        }
        let ident = ident.unwrap();

        let mut initializer: Option<Box<Expr>> = None;
        if self.match_tokens(&[TokenType::Equal]) {
            let expr = self.expression();
            if let None = expr {
                return None;
            }
            let expr = expr.unwrap();
            initializer = Some(Box::new(expr));
        }

        while self.peek().token_type == TokenType::Semicolon {
            self.advance();
        }
        return Some(Stmt::Let(ident, initializer));
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_tokens(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_tokens(&[TokenType::LeftBrace]) {
            let block = self.block();
            if let None = block {
                return None;
            }

            return Some(Stmt::Block(block.unwrap()));
        }

        if self.match_tokens(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_tokens(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_tokens(&[TokenType::For]) {
            return self.for_statement();
        }

        return self.expression_statement();
    }

    fn for_statement(&mut self) -> Option<Stmt> {
        if let None = self.consume(
            TokenType::LeftParen, "Expect '(' after 'for'.".to_string()
        ) {
            return None;
        }

        #[allow(unused_assignments)]
        let mut initializer: Option<Stmt> = None;
        if self.match_tokens(&[TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_tokens(&[TokenType::Var]) {
            initializer = self.var_declaration();
        } else {
            initializer = self.expression_statement();
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon) {
            condition = self.expression();
        }

        if let None = self.consume(
            TokenType::Semicolon, "Expect ';' after condition.".to_string()
        ) {
            return None;
        }

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = self.expression();
        }

        if let None = self.consume(
            TokenType::RightParen, "Expect ')' after for clauses.".to_string()
        ) {
            return None;
        }
        let body_outer = self.statement();
        if let None = body_outer {
            return None;
        }
        let mut body_inner = body_outer.unwrap();
        if let Some(inc) = increment {
            body_inner = Stmt::Block(
                vec![
                Box::new(body_inner),
                Box::new(Stmt::Expression(Box::new(inc)))
                ]
            );
        }

        let cond: Expr = condition.unwrap_or(Expr::Literal("true".to_string()));
        body_inner = Stmt::While(cond, Box::new(body_inner));

        if let Some(init_expr) = initializer {
            body_inner = Stmt::Block(vec![Box::new(init_expr), Box::new(body_inner)]);
        }

        return Some(body_inner);
    }

    fn while_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'".to_string());
        let condition = self.expression();
        if let None = condition { return None; }
        let condition = condition.unwrap();
        self.consume(TokenType::RightParen, "Expect ')' after condition.".to_string());

        let body = self.statement();
        if let None = body { return None; }
        let body = Box::new(body.unwrap());

        return Some(Stmt::While(condition, body));
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.".to_string());
        let condition = self.expression();
        if let None = condition { return None; }
        let condition = condition.unwrap();

        self.consume(TokenType::RightParen, "Expect ')' after if condition.".to_string());
        // self.consume(TokenType::LeftBrace, "Expect '{' after if.".to_string());
        let then_branch = self.statement();
        if let None = then_branch {
            return None;
        }
        let then_branch = then_branch.unwrap();
        let mut else_branch = None;
        if self.match_tokens(&[TokenType::Else]) {
            else_branch = self.statement();
            if let None = else_branch {
                return None;
            }
        }
        Some(Stmt::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch)
        ))
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let value = self.expression();
        while self.peek().token_type == TokenType::Semicolon {
            self.advance();
        }

        if let None = value {
            return None;
        }

        return Some(Stmt::Print(Box::new(value.unwrap())));
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let value = self.expression();
        if self.peek().token_type == TokenType::Semicolon {
            self.advance();
        }

        if let None = value {
            return None;
        }

        return Some(Stmt::Expression(Box::new(value.unwrap())));
    }

    fn expression(&mut self) -> Option<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr> {
        let expr = self.or();
        if let None = expr {
            return None;
        }

        if self.match_tokens(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment();
            if let None = value {
                return None;
            }
            let value = value.unwrap();

            if let Some(Expr::Variable(var)) = expr {
                return Some(Expr::Assign(var, Box::new(value)));
            }

            Lox::report(equals.line, format!("at '{}'", equals.lexeme), "Invalid assignment Target.".to_string());
            return None;
        }

        return expr;
    }

    fn or(&mut self) -> Option<Expr> {
        let expr = self.and();
        if let None = expr {
            return None;
        }

        let mut expr = expr.unwrap();
        while self.match_tokens(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and();
            if let None = right { break; }
            let right = right.unwrap();

            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        return Some(expr);
    }

    fn and(&mut self) -> Option<Expr> {
        let expr = self.ternary();
        if let None = expr {
            return None;
        }
        let mut expr = expr.unwrap();

        while self.match_tokens(&[TokenType::And]) {
            let op = self.previous();
            let right = self.equality();
            if let None = right { break; }
            let right = right.unwrap();

            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }

        return Some(expr);
    }

    fn ternary(&mut self) -> Option<Expr> {
        let condition = self.equality();
        if let None = condition {
            return None;
        }

        if let false = self.match_tokens(&[TokenType::Qmark]) {
            return condition;
        }

        let left = self.primary();
        if let None = left {
            Lox::error(self.tokens[self.current].line, "Expect a expression after ?".to_string());
            return None;
        }
        let left = left.unwrap();

        if let false = self.match_tokens(&[TokenType::Colon]) {
            Lox::error(
                self.tokens[self.current].line,
                format!("Expect : after ternary expression. got '{:?}' instead.", self.tokens[self.current])
                );
            return None;
        }

        let right = self.ternary();
        if let None = right {
            Lox::error(
                self.tokens[self.current].line,
                format!("Expect a expression after :. got '{:?}' instead.", self.tokens[self.current])
                );
            return None;
        }
        let right = right.unwrap();

        return Some(Expr::Ternary(Box::new(condition.unwrap()), Box::new(left), Box::new(right)));
    }

    fn equality(&mut self) -> Option<Expr> {
        let expr = self.comparison();
        if let None = expr {
            return None;
        }
        let mut expr = expr.unwrap();

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            if right.is_none() {
                break;
            }
            expr = Expr::Binary(
                Box::new(expr),
                operator,
                Box::new(right.unwrap())
            );
        }

        return Some(expr);
    }

    fn comparison(&mut self) -> Option<Expr> {
        let expr = self.term();
        if let None = expr {
            return None;
        }
        let mut expr = expr.unwrap();

        while self.match_tokens(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous();
            let right = self.term();
            if right.is_none() {
                break;
            }

            expr = Expr::Binary(
                Box::new(expr),
                operator,
                Box::new(right.unwrap())
            );
        }

        return Some(expr);
    }

    fn term(&mut self) -> Option<Expr> {
        let expr = self.factor();
        if let None = expr {
            return None;
        }
        let mut expr = expr.unwrap();

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor();
            if right.is_none() {
                return None;
            }

            expr = Expr::Binary(
                Box::new(expr),
                operator,
                Box::new(right.unwrap())
            );
        }

        return Some(expr);
    }

    fn factor(&mut self) -> Option<Expr> {
        let expr = self.unary();
        if let None = expr {
            return None;
        }
        let mut expr = expr.unwrap();

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            if right.is_none() {
                break;
            }

            expr = Expr::Binary(
                Box::new(expr),
                operator,
                Box::new(right.unwrap())
            );
        }

        return Some(expr);
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            if right.is_none() {
                return None;
            }

            return Some(Expr::Unary(operator, Box::new(right.unwrap())));
        }

        return self.primary();
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.match_tokens(&[TokenType::False]) {
            return Some(Expr::Literal("false".to_string()));
        }

        if self.match_tokens(&[TokenType::True]) {
            return Some(Expr::Literal("true".to_string()));
        }

        if self.match_tokens(&[TokenType::Nil]) {
            return Some(Expr::Literal("nil".to_string()));
        }

        if self.match_tokens(&[TokenType::Number, TokenType::String]) {
            return Some(Expr::Literal(self.previous().lexeme));
        }

        if self.match_tokens(&[TokenType::LeftParen]) {
            let expr = self.expression().unwrap();
            self.consume(TokenType::RightParen, "Expect ')' after expression.".to_string());

            return Some(Expr::Grouping(Box::new(expr)));
        }

        if self.match_tokens(&[TokenType::Identifier]) {
            return Some(Expr::Variable(self.previous()));
        }

        Lox::error(
            self.tokens[self.current].line,
            format!("Expect expression. got {} instead", self.tokens[self.current].lexeme)
            );
        return None;
    }

    fn consume(&mut self, type_: TokenType, msg: String) -> Option<Token> {
        if self.check(type_) { return Some(self.advance()); }
        Lox::error(self.tokens[self.current].line, msg);
        return None;
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                // TokenType::Fun
                TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print => return,
                // | TokenType::Return => return,
                _ => self.advance(),
            };
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for token in types {
            if self.check(*token) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn check(&mut self, type_: TokenType) -> bool {
        if self.is_at_end() { return false; }

        return self.peek().token_type == type_;
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.current += 1; }

        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
