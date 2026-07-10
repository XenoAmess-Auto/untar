mod archive;
mod cli;
mod extract;

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

    let options = ExtractOptions::new(
        args.output_dir(),
        args.quiet,
        args.list,
        args.on_exists,
        args.rename_suffix,
        args.strip_components,
        args.patterns,
    );

    if let Err(e) = extract::extract_archive(Path::new(&file), &options) {
        eprintln!("Error: {e}");
        if !options.quiet {
            eprintln!("{e:?}");
        }
        exit(1);
    }

    if !options.quiet && !options.list {
        println!("Done: {file}");
    }
}
