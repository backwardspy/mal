use std::error::Error;

use rustyline::{error::ReadlineError, Editor};

const HISTFILE: &str = ".mal_history";

fn read(input: &str) -> &str {
    input
}

fn eval(input: &str) -> &str {
    input
}

fn print(input: &str) -> &str {
    input
}

fn rep(input: &str) -> &str {
    print(eval(read(input)))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut editor = Editor::<()>::new()?;
    editor.load_history(HISTFILE).ok();

    loop {
        match editor.readline("user> ") {
            Ok(input) => {
                let input = input.trim();
                editor.add_history_entry(input);
                if !input.is_empty() {
                    println!("{}", rep(input));
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("error: {e}");
                continue;
            }
        }
    }

    editor.save_history(HISTFILE)?;
    Ok(())
}
