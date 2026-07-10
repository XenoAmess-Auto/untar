mod archive;
mod cli;
mod cracker;
mod extract;

use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

use clap::Parser;

use cli::Args;
use extract::ExtractOptions;

fn main() {
    let args = Args::parse();

    let file = match args.archive_file() {
        Some(f) => f,
        None => {
            eprintln!("Error: No archive file specified");
            eprintln!("\nUsage: untar [OPTIONS] <FILE>");
            eprintln!("\nFor more information, use --help.");
            exit(1);
        }
    };

    let limits = extract::LimitTracker::new(
        args.max_total_size,
        args.max_entry_size,
        args.max_entry_count,
        args.max_compression_ratio,
        args.max_recursion_depth,
        args.allow_unsafe,
    );
    let mut options = ExtractOptions {
        output_dir: args.output_dir(),
        quiet: args.quiet,
        list: args.list,
        on_exists: args.on_exists,
        rename_suffix: args.rename_suffix,
        strip_components: args.strip_components,
        patterns: args.patterns,
        password: args.password,
        format: args.format,
        limits,
    };

    let file_path = Path::new(&file);

    if args.extract_hash {
        let fmt = match cracker::resolve_format_for(file_path, options.format.as_deref()) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: {e}");
                exit(1);
            }
        };
        if let Err(e) = cracker::extract_hash(file_path, fmt) {
            eprintln!("Error: {e}");
            exit(1);
        }
        return;
    }

    if args.crack {
        match cracker::crack_archive(file_path, &options, args.wordlist.as_deref().map(Path::new)) {
            Ok(Some(password)) => {
                options.password = Some(password);
            }
            Ok(None) => {
                eprintln!("Error: Could not crack archive password");
                exit(1);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                exit(1);
            }
        }
    }

    if let Err(e) = extract::extract_archive(file_path, &options) {
        if is_password_error(&e) && !args.crack && extract::is_tty() {
            eprintln!("Error: {e}");
            eprint!("Password incorrect. Try to crack? [y/N] ");
            let _ = Write::flush(&mut io::stderr());
            if prompt_yes() {
                match cracker::crack_archive(
                    file_path,
                    &options,
                    args.wordlist.as_deref().map(Path::new),
                ) {
                    Ok(Some(password)) => {
                        options.password = Some(password);
                    }
                    Ok(None) => {
                        eprintln!("Error: Could not crack archive password");
                        exit(1);
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        exit(1);
                    }
                }
                if let Err(e) = extract::extract_archive(file_path, &options) {
                    eprintln!("Error: {e}");
                    if !options.quiet {
                        eprintln!("{e:?}");
                    }
                    exit(1);
                }
            } else {
                exit(1);
            }
        } else {
            eprintln!("Error: {e}");
            if !options.quiet {
                eprintln!("{e:?}");
            }
            exit(1);
        }
    }

    if !options.quiet && !options.list {
        println!("Done: {file}");
    }
}

fn is_password_error(e: &anyhow::Error) -> bool {
    let msg = e.to_string().to_lowercase();
    msg.contains("password")
        || msg.contains("decrypt")
        || msg.contains("encrypted")
        || msg.contains("crc")
}

fn prompt_yes() -> bool {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}
