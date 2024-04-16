use crate::{environment::Environment, parser::{Expr, Stmt}, token::{Token, TokenType}, Lox};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Option<()> {
        for stmt in statements {
            let res = self.execute(stmt);
            if let None = res {
                return None;
            }
        }

        Some(())
    }

    fn execute(&mut self, expr: Stmt) -> Option<()> {
        match expr {
            Stmt::Expression(e) => {
                let result = self.evaluate(*e);
                if let None = result {
                    return None;
                }
                return Some(());

            },
            Stmt::Print(e) => {
                let res = self.evaluate(*e);
                if let None = res {
                    return None;
                }

                println!("{}", res.unwrap());

                return Some(())
            },
            Stmt::Let(op, expr) => {
                let res = self.let_statement(op, expr);
                if let None = res {
                    return None;
                }

                return Some(())
            },
            Stmt::Block(statements) => {
                let env = Environment::with_enclosing(self.environment.clone());
                let res = self.execute_block(statements, env);
                if let None = res {
                    return None;
                }
                Some(())
            },
            Stmt::If(cond, then, else_) => {
                let res = self.if_statement(cond, *then, *else_);
                if let None = res {
                    return None;
                }
                Some(())
            },
            Stmt::While(cond, body) => {
                let res = self.while_statement(cond, *body);
                if let None = res {
                    return None;
                }

                Some(())
            }
        }
    }

    fn while_statement(&mut self, condition: Expr, body: Stmt) -> Option<()> {
        while Self::is_truthy(self.evaluate(condition.clone()).unwrap()) {
            self.execute(body.clone());
        }

        return Some(())
    }

    fn if_statement(&mut self, condition: Expr, then: Stmt, else_: Option<Stmt>) -> Option<()> {
        let cond = self.evaluate(condition);
        if let None = cond {
            return None;
        }
        if Self::is_truthy(cond.unwrap()) {
            return self.execute(then);
        } else if else_.is_some() {
            return self.execute(else_.unwrap());
        }
        Some(())
    }

    fn execute_block(&mut self, statements: Vec<Box<Stmt>>, env: Environment) -> Option<()> {
        self.environment = env.clone();
        let mut cur = env.clone();

        for stmt in statements {
            let res = self.execute(*stmt);
            if let None = res {
                return None;
            }
            cur = *self.environment.enclosing.clone().unwrap();
        }

        self.environment = cur;
        Some(())
    }

    fn let_statement(&mut self, token: Token, expr: Option<Box<Expr>>) -> Option<()> {
        let mut value = "nil".to_string();
        if let Some(init_val) = expr {
            value = self.evaluate(*init_val).unwrap_or("nil".to_string());
        }

        self.environment.define(token.lexeme, value);
        return Some(())
    }

    fn evaluate(&mut self, expr: Expr) -> Option<String> {
        match expr {
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right),
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Grouping(expression) => self.eval_group(expression),
            Expr::Unary(op, right) => self.eval_unary(op, right),
            Expr::Ternary(cond, left, right) => self.eval_ternary(*cond, *left, *right),
            Expr::Variable(var) => {
                let value = self.environment.get(var);
                if let None = value {
                    return None;
                }
                return value;
            },
            Expr::Assign(name, expr) => {
                let value = self.evaluate(*expr);
                if let None = value.clone() {
                    return None;
                }
                self.environment.assign(
                    name,
                    value.clone().unwrap_or("nil".to_string())
                );
                return value;
            },
            Expr::Logical(left, op, right) => {
                let res = self.eval_logical(*left, op, *right);
                if let None = res {
                    return None;
                }

                return Some(res.unwrap());
            },
        }
    }

    fn eval_logical(
        &mut self,
        left: Expr,
        operator: Token,
        right: Expr
    ) -> Option<String> {
        let left = self.evaluate(left);
        if let None = left {
            return None;
        }
        let left = left.unwrap();

        if operator.token_type == TokenType::Or {
            if Self::is_truthy(left.clone()) {
                return Some(left);
            }
        } else {
            if !Self::is_truthy(left.clone()) {
                return Some(left);
            }
        }

        return self.evaluate(right);
    }

    fn eval_binary(
        &mut self, left: Box<Expr>,
        op: Token, right: Box<Expr>
    ) -> Option<String> {
        let left = self.evaluate(*left);
        if let None = left {
            return None;
        }
        let left = left.unwrap();
        let right = self.evaluate(*right);
        if let None = right {
            return None;
        }
        let right = right.unwrap();

        match op.token_type {
            TokenType::Greater => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left > right).to_string());
            },
            TokenType::GreaterEqual => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left >= right).to_string());
            },
            TokenType::Less => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left < right).to_string());
            },
            TokenType::LessEqual => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left <= right).to_string());
            },
            TokenType::BangEqual => {
                return Some((!Self::is_equals(left, right)).to_string());
            },
            TokenType::EqualEqual => {
                return Some(Self::is_equals(left, right).to_string());
            },
            TokenType::Minus => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left - right).to_string());
            },

            TokenType::Slash => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left / right).to_string());
            },

            TokenType::Star => {
                if let None = Self::check_number_operands(op, left.clone(), right.clone()) {
                    return None;
                }
                let left = left.parse::<f64>().unwrap();
                let right = right.parse::<f64>().unwrap();

                return Some((left * right).to_string());
            },
            TokenType::Plus => {
                if Self::is_number(&left) && Self::is_number(&right) {
                    let left = left.parse::<f64>().unwrap();
                    let right = right.parse::<f64>().unwrap();

                    return Some((left + right).to_string());
                }

                if Self::is_alpha(&left) || Self::is_alpha(&right) {
                    return Some(format!("{}{}", left, right));
                }

                Lox::report(op.line, op.lexeme, "Operands must be two numbers or two strings.".to_string());
                return None;
            }
            _ => todo!()
        }
    }

    fn eval_literal(&mut self, literal: String) -> Option<String> {
        Some(literal)
    }

    fn eval_group(&mut self, expr: Box<Expr>) -> Option<String> {
        return self.evaluate(*expr);
    }

    fn eval_unary(&mut self, operator: Token, right: Box<Expr>) -> Option<String> {
        let right = self.evaluate(*right).unwrap();

        match operator.token_type {
            TokenType::Minus => {
                if let None = Self::check_number_operand(operator, right.clone()) {
                    return None;
                }
                let right = right.parse::<f64>().unwrap();

                Some((-right).to_string())
            },
            TokenType::Bang => {
                Some((!Self::is_truthy(right)).to_string())
            },
            _ => unreachable!()
        }
    }

    fn eval_ternary(&mut self, condition: Expr, left: Expr, right: Expr) -> Option<String> {
        let condition = self.evaluate(condition).unwrap();
        if Self::is_truthy(condition) {
            return self.evaluate(left);
        } else {
            return self.evaluate(right);
        }
    }

    fn check_number_operand(
        operator: Token,
        operand: String
        ) -> Option<()> {
        if operand.parse::<f64>().is_ok() { return Some(()); }

        Lox::report(
            operator.line,
            format!("at '{}' ", operator.lexeme),
            "Operand must be a number.".to_string()
        );
        return None;
    }

    fn check_number_operands(
        operator: Token,
        operand1: String,
        operand2: String
        ) -> Option<()> {
        if operand1.parse::<f64>().is_ok() && operand2.parse::<f64>().is_ok()
        { return Some(()); }

        Lox::report(operator.line, operator.lexeme, "Operands must be number.".to_string());
        return None;
    }

    fn is_equals(a: String, b: String) -> bool {
        if a == "nil".to_string() && b == "nil".to_string() {
            return true;
        }

        if a == "nil".to_string() {
            return false;
        }

        return a == b;
    }

    fn is_truthy(object: String) -> bool {
        if object == "nil".to_string() { return false; }
        if object == "false".to_string() { return false; }

        return true;
    }

    fn is_number(string: &str) -> bool {
        string.parse::<f64>().is_ok()
    }

    fn is_alpha(string: &str) -> bool {
        string.parse::<f64>().is_err()
    }
}
