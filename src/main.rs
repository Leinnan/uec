use std::path::PathBuf;

use clap::{Parser, Subcommand};
use editor::Editor;

pub mod config;
pub mod consts;
pub mod editor;
pub mod uproject;

#[derive(Parser)]
#[command(version, about, long_about = "Unreal Engine CLI helper tool")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    /// Override the Unreal Engine Path from config.
    engine_path: Option<PathBuf>,
    #[arg(long)]
    /// Save logs from command into specified file.
    save_logs: Option<PathBuf>,
    /// Log only errors
    #[clap(long, action)]
    error_only: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Runs the Unreal editor without an Unreal project.
    Editor,
    /// Builds a Unreal project.
    Build { path: Option<PathBuf> },
    /// Generate a Unreal project.
    GenerateProjectFiles { path: Option<PathBuf> },
    /// Builds and run a Unreal editor project.
    EditorProject { path: Option<PathBuf> },
    /// Sets the default Unreal Engine Path.
    SetEditor { name: PathBuf },
    /// Prints the current command configuration.
    PrintConfig,
    /// Builds a Unreal plugin.
    BuildPlugin {
        path: Option<PathBuf>,
        output_dir: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut editor = Editor::create(&cli);

    match &cli.command {
        Commands::SetEditor { name } => {
            if Editor::build_editor_exec(name.to_str().unwrap()).is_none() {
                panic!("EDITOR AT PATH DOES NOT EXISTS! {}", name.display());
            }
            name.to_str()
                .unwrap()
                .clone_into(&mut editor.config.editor_path);
            editor.config.save();
            println!("Updated the editor path to new one: {name:?}");
        }
        Commands::Editor => editor.run_editor(),
        Commands::Build { path } => editor.build_project(path),
        Commands::EditorProject { path } => editor.build_editor_project(path),
        Commands::GenerateProjectFiles { path } => editor.generate_proj_files(path),
        Commands::BuildPlugin { path, output_dir } => editor.build_plugin(path, output_dir),
        Commands::PrintConfig => println!("{:#?}", &editor.config),
    }
}
