use std::path::PathBuf;

use clap::{Parser, Subcommand};
use config::Config;

pub mod config;
pub mod consts;
pub mod editor;

#[derive(Parser)]
#[command(version, about, long_about = "Unreal Engine helper tool")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    /// Used Unreal Engine Path
    engine_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the unreal editor without an Unreal project.
    Editor,
    /// Builds a Unreal project.
    Build { path: Option<PathBuf> },
    /// Generate a Unreal project.
    GenerateProjectFiles { path: Option<PathBuf> },
    /// Sets the default Unreal Engine Path.
    SetEditor { name: PathBuf },
}

fn main() {
    let cli = Cli::parse(); //.unwrap_or(Cli::parse_from(&all_args[0..2]));
    let mut config = Config::load_or_create();
    if let Some(engine) = cli.engine_path {
        config.editor_path = engine.to_str().unwrap().into();
    }
    println!("{:?}", &config);

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::SetEditor { name } => {
            if editor::get_editor_exec(name.to_str().unwrap()).is_none() {
                panic!("EDITOR AT PATH DOES NOT EXISTS! {}", name.display());
            }
            name.to_str().unwrap().clone_into(&mut config.editor_path);
            config.save();
            println!("Updated the editor path to new one: {name:?}");
        }
        Commands::Editor => editor::run_editor(&config),
        Commands::Build { path } => editor::build_project(&config, path),
        Commands::GenerateProjectFiles { path } => editor::generate_proj_files(&config, path),
    }
}
