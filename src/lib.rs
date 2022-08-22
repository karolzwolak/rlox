use std::{io::{self, Write}, fs};

pub mod bytecode;
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
    Ok(())
}
