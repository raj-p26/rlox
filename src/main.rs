use std::{fs, io::{self, BufRead, Write}};

use interpreter::Interpreter;
use parser::Parser;

use crate::scanner::Scanner;

mod environment;
mod interpreter;
mod parser;
mod scanner;
mod tests;
mod token;

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    extract_ast: bool,
    target_file: Option<String>,
}

impl Lox {
    fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
            extract_ast: false,
            target_file: None,
        }
    }
    fn run_prompt(&mut self) {
        loop {
            let mut source = String::new();
            print!(">>> ");
            io::stdout().flush().expect("Error flushing stdout.");
            let input = io::stdin()
                .lock()
                .read_line(&mut source)
                .expect("Error reading from stdin.");

            if input == 0 {
                return;
            }

            self.run(source);
            self.had_error = false;
        }
    }

    fn run_file(&mut self, path: String) {
        let source = std::fs::read_to_string(path);
        if let Err(e) = &source {
            eprintln!("{}", e.to_string());
            return;
        }
        let source = source.unwrap();

        self.run(source);

        if self.had_error { std::process::exit(65); }
        if self.had_runtime_error { std::process::exit(70); }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        if let None = tokens {
            self.had_error = true;
            return;
        }

        let tokens = tokens.unwrap().to_owned();

        let mut parser = Parser::new(tokens);
        let expr = parser.parse();
        if let None = expr {
            self.had_error = true;
            return;
        }

        let expr = expr.unwrap();

        if self.extract_ast && self.target_file.is_some() {
            let target_file = self.target_file.as_ref().unwrap();
            let mut file = fs::File::create(target_file).unwrap();
            file.write_all(format!("{:#?}", expr.clone()).as_bytes()).unwrap();

        }

        let mut interpreter = Interpreter::new();
        if let None = interpreter.interpret(expr) {
            self.had_runtime_error = true;
        }
    }

    pub fn error(line: usize, msg: String) {
        Self::report(line, "".to_string(), msg);
    }

    pub fn report(line: usize, where_: String, msg: String) {
        eprintln!("line[{line}] Error {where_}: {msg}");
    }
}

fn main() {
    let mut lox = Lox::new();

    let args = std::env::args().collect::<Vec<String>>();

    if args.len() > 4 {
        eprintln!("Usage: lox [script]");
        std::process::exit(64);
    } else if args.len() == 4 {
        // Weak arguments check.
        // TODO: check for the `--extract-ast` flag.
        lox.extract_ast = true;
        lox.target_file = Some(args[3].clone());
        lox.run_file(args[1].clone());
    }else if args.len() == 2 {
        lox.run_file(args[1].clone());
    } else {
        lox.run_prompt();
    }
}
