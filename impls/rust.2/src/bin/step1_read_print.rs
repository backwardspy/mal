use std::error::Error;

use rustyline::{error::ReadlineError, Editor};

use mal::{
    printer::pr_str,
    reader::{read_str, ReadError},
    types::Value,
};

const HISTFILE: &str = ".mal_history";
const PRETTYPRINT: bool = false;
const DBGINFO: bool = false;

fn read(input: &str) -> Result<Value, ReadError> {
    if DBGINFO {
        println!("read: {input}");
    }
    read_str(input)
}

fn eval(input: Value) -> Value {
    if DBGINFO {
        println!("eval: {input:?}");
    }
    input
}

fn print(input: Value) -> String {
    if DBGINFO {
        println!("print: {input:?}");
    }
    pr_str(input, PRETTYPRINT)
}

fn rep(input: &str) -> Result<String, ReadError> {
    Ok(print(eval(read(input)?)))
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
                    match rep(input) {
                        Ok(output) => println!("{output}"),
                        Err(ReadError::NoInput) => (),
                        Err(error) => eprintln!("error: {error}"),
                    }
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
