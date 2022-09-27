use std::env;

fn main() {
    let mut args = env::args();
    let result = if args.len() == 1 {
        rlox::run_repl()
    } else if args.len() == 2 {
        rlox::run_file(args.nth(1).unwrap())
    } else {
        eprintln!("Usage: rlox [script]");
        std::process::exit(64);
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
