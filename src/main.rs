use lox_rs::Lox;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    println!("Arguments: {:?}", args);
    if args.len() > 2 {
        println!("Usage: lox-rs [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        Lox::run_file(args[1].as_str());
    } else {
        Lox::run_prompt();
    }
}
