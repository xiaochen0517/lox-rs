use lox_rs::Lox;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    println!("Arguments: {:?}", args);
    let lox = Lox::new();
    if args.len() > 2 {
        println!("Usage: lox-rs [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        lox.run_file(args[1].as_str());
    } else {
        lox.run_prompt();
    }
}
