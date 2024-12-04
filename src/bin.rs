use cbindgen::{Config, Language};
use clap::{Args as ClapArgs, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Header(HeaderArgs),
}

#[derive(ClapArgs, Debug, Clone)]
struct HeaderArgs {
    output_path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Header(args) => {
            let config = Config {
                language: Language::C,
                pragma_once: true,
                ..Default::default()
            };

            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
            cbindgen::generate_with_config(&manifest_dir, config)
                .expect("Unable to generate header file")
                .write_to_file(&args.output_path);

            println!("Header file generated at: {}", args.output_path.display());
        }
    }
}
