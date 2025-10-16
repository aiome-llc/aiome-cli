use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn main() -> Result<(), ReadlineError> {
    run_repl()
}

fn run_repl() -> Result<(), ReadlineError> {
    let mut editor = DefaultEditor::new()?;
    loop {
        match editor.readline("aiome> ") {
            Ok(line) => {
                println!("{line}");
                editor.add_history_entry(line)?;
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(err) => {
                eprintln!("error: {err}");
                break;
            }
        }
    }

    Ok(())
}
