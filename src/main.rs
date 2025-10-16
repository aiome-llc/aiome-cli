use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, ExternalPrinter};

fn main() -> Result<(), ReadlineError> {
    run_repl()
}

fn run_repl() -> Result<(), ReadlineError> {
    let mut editor = DefaultEditor::new()?;
    let mut printer = editor.create_external_printer().ok();

    loop {
        match editor.readline("aiome> ") {
            Ok(line) => {
                if let Some(printer) = printer.as_mut() {
                    printer.print(format!("{line}\n"))?;
                } else {
                    println!("{line}");
                }
                editor.add_history_entry(line)?;
            }
            Err(ReadlineError::Interrupted) => {
                if let Some(printer) = printer.as_mut() {
                    printer.print("^C\n".to_string())?;
                } else {
                    println!("^C");
                }
            }
            Err(ReadlineError::Eof) => {
                if let Some(printer) = printer.as_mut() {
                    printer.print("\n".to_string())?;
                } else {
                    println!();
                }
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
