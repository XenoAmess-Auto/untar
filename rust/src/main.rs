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

    let limits = extract::LimitTracker::new(
        args.max_total_size,
        args.max_entry_size,
        args.max_entry_count,
        args.max_compression_ratio,
        args.max_recursion_depth,
        args.allow_unsafe,
    );
    let options = ExtractOptions {
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
