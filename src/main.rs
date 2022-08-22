use std::env;

fn main() -> rlox::Result<()> {
    let mut args = env::args();
    if args.len() == 1{
        rlox::run_repl()
    } else if args.len() == 2 {
        rlox::run_file(args.nth(1).unwrap())
    } else {
        eprintln!("Usage: rlox [script]");
        std::process::exit(64);
    }
}
