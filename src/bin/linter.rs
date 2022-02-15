use color_eyre::eyre::{eyre, Result};
use jwalk::WalkDir;
use php_linter::{lints, procces, should_lint, LintError};
use std::env::args;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let dir = args().nth(1).ok_or_else(|| eyre!("Path not given"))?;
    tracing::debug!(%dir, "searching");

    tracing::info!("registered lints: {:?}", lints::LINTS);

let mut count = 0;

    let receiver = {
        let (emit_error, receiver) = crossbeam::channel::unbounded();

        for entry in WalkDir::new(&dir).into_iter().flatten() {
            if should_lint(&entry) {
                count += 1;

                let emit_error = emit_error.clone();
                rayon::spawn(|| {
                    procces(entry, emit_error);
                });
            }
        }
        receiver
    };

    
    // for error in receiver {
    //     println!("{}", error);
    // }

    let errors: Vec<LintError> = receiver.into_iter().collect();

    for error in errors {
        println!("{}", error);
    }

    println!("{}", count);

    // if !errors.is_empty() {
    //     println!("there were errors: {:?}", errors);
    // }

    Ok(())
}
