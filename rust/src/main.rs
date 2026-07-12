mod archive;
mod cli;
mod cracker;
mod extract;

use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

use anyhow::anyhow;
use clap::Parser;

use cli::Args;
use extract::ExtractOptions;

fn main() {
    let args = Args::parse();

    if args.files.is_empty() {
        eprintln!("Error: No archive file specified");
        eprintln!("\nUsage: untar [OPTIONS] <FILES>...");
        eprintln!("\nFor more information, use --help.");
        exit(1);
    }

    let base_output_dir = args.output_dir();
    let mut had_error = false;

    for file in &args.files {
        let file_path = Path::new(file);
        let output_dir = if args.auto_dir {
            base_output_dir.join(extract::archive_stem(file_path))
        } else {
            base_output_dir.clone()
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
            output_dir,
            quiet: args.quiet,
            list: args.list,
            on_exists: args.on_exists,
            rename_suffix: args.rename_suffix.clone(),
            strip_components: args.strip_components,
            patterns: args.patterns.clone(),
            password: args.password.clone(),
            format: args.format.clone(),
            limits,
        };

        if let Err(e) = process_file(file_path, &args, &mut options) {
            had_error = true;
            eprintln!("Error: {e}");
            if !args.quiet {
                eprintln!("{e:?}");
            }
        }
    }

    if had_error {
        exit(1);
    }
}

fn process_file(file_path: &Path, args: &Args, options: &mut ExtractOptions) -> anyhow::Result<()> {
    if args.extract_hash {
        let fmt = match cracker::resolve_format_for(file_path, options.format.as_deref()) {
            Ok(f) => f,
            Err(e) => {
                return Err(anyhow!("Cannot determine format for hash extraction: {e}"));
            }
        };
        cracker::extract_hash(file_path, fmt)?;
        return Ok(());
    }

    if args.crack {
        try_crack(file_path, args, options)?;
    }

    if let Err(e) = extract::extract_archive(file_path, options) {
        if is_password_error(&e) && !args.crack && extract::is_tty() {
            eprintln!("Error: {e}");
            eprint!("Password incorrect. Try to crack? [y/N] ");
            let _ = Write::flush(&mut io::stderr());
            if prompt_yes() {
                try_crack(file_path, args, options)?;
                extract::extract_archive(file_path, options)?;
            } else {
                return Err(e);
            }
        } else {
            return Err(e);
        }
    }

    if !args.quiet && !args.list {
        println!("Done: {}", file_path.display());
    }

    Ok(())
}

fn try_crack(file_path: &Path, args: &Args, options: &mut ExtractOptions) -> anyhow::Result<()> {
    match cracker::crack_archive(file_path, options, args.wordlist.as_deref().map(Path::new)) {
        Ok(Some(password)) => {
            options.password = Some(password);
            Ok(())
        }
        Ok(None) => Err(anyhow!("Could not crack archive password")),
        Err(e) => Err(e),
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
