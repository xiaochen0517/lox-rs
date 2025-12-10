mod scanner;

fn run(content: String) {
    let scanner = scanner::Scanner::new(content);
    for token in scanner.scan_tokens() {
        println!("{:?}", token);
    }
}

fn run_file(path: &str) {
    let file_content_string = std::fs::read_to_string(path).expect("Reader File Error");
    run(file_content_string);
}

fn run_prompt() {
    let stdin = std::io::stdin();
    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                run(line);
            }
            Err(error) => {
                eprintln!("Error reading line: {}", error);
                break;
            }
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    println!("Arguments: {:?}", args);
    if args.len() > 2 {
        println!("Usage: lox-rs [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(args[1].as_str());
    } else {
        run_prompt();
    }
}
