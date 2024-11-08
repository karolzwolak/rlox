use std::{
    cell::RefCell,
    fs,
    io::{self, Write},
};

pub mod bytecode;
pub mod compiler;
pub mod scanner;
pub mod token;
pub mod vm;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub fn run_repl() -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if matches!(line.trim(), "quit" | "q!") {
            break;
        }

        if let Err(error) = interpret(line) {
            eprintln!("error: {}", error);
        }
    }
    Ok(())
}

pub fn run_file(path: String) -> Result<()> {
    fs::read_to_string(path)
        .map_err(|e| e.into())
        .and_then(interpret)
}

fn interpret(source: String) -> Result<()> {
    let parser = RefCell::new(compiler::Parser::with_source(&source));
    let compiler = compiler::Compiler::main_compiler(&parser);

    let code = compiler.compile()?;
    let mut vm = vm::VM::with_code(code);
    vm.run()
}
