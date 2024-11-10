use std::path::PathBuf;

use clap::{Parser, Subcommand};
use colour::cyan_ln_bold;
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
    /// Dry run, no command would be run. Instead it will just output what it would run.
    #[clap(long, action)]
    dry_run: bool
}

#[derive(Subcommand)]
pub enum Commands {
    /// Runs the Unreal editor without an Unreal project.
    Editor,
    /// Builds a Unreal project.
    Build {
        /// Optional path to directory containing the `.uproject` file.
        /// When no value is provided it will use current directory
        path: Option<PathBuf>,
        /// Optional path to directory that game would be build to.
        /// When no value is provided, it will save to the newly created `CookedBuild` directory created in current directory
        output: Option<PathBuf>,
    },
    /// Generate a Unreal project.
    GenerateProjectFiles {
        /// Optional path to directory containing the `.uproject` file.
        /// When no value is provided it will use current directory
        path: Option<PathBuf>,
    },
    /// Builds and run a Unreal editor project.
    EditorProject {
        /// Optional path to directory containing the `.uproject` file.
        /// When no value is provided it will use current directory
        path: Option<PathBuf>,
    },
    /// Cleans all the intermediate files and directories from project.
    CleanProject {
        /// Optional path to directory containing the `.uproject` file.
        /// When no value is provided it will use current directory
        path: Option<PathBuf>,
    },
    /// Sets the default Unreal Engine Path.
    SetEditor { name: PathBuf },
    /// Prints the current command configuration.
    PrintConfig,
    /// Builds a Unreal plugin.
    BuildPlugin {
        path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    /// Build Unreal Engine from source.
    BuildEngine { path: Option<PathBuf> },
    /// Run Unreal Automation Tool Command.
    UAT {
        /// Input value for the UAT, example: "BuildCookRun -help"
        input: String,
        /// Optional path to directory containing the `.uproject` file.
        /// When no value is provided it will use current directory
        path: Option<PathBuf>,
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
            if !cli.dry_run {
                editor.config.save();
                println!("Updated the editor path to new one: {name:?}");
            } else {
                cyan_ln_bold!("[DRY_RUN] Updated the editor path to new one: {name:?}");
            }
        }
        Commands::Editor => editor.run_editor(),
        Commands::Build { path, output } => {
            let _ = editor.build_project(path, output);
        }
        Commands::BuildEngine { path } => editor.build_engine_from_source(path),
        Commands::CleanProject { path } => {
            let _ = editor
                .clean_project(path)
                .inspect_err(|e| eprintln!("Failed to clean up the project, reason: {:#?}", e));
        }
        Commands::EditorProject { path } => editor.build_editor_project(path),
        Commands::GenerateProjectFiles { path } => editor.generate_proj_files(path),
        Commands::BuildPlugin { path, output } => editor.build_plugin(path, output),
        Commands::UAT { input, path } => {
            let args: Vec<&str> = input.split(' ').collect();
            let _ = editor.run_uat(path, args);
        }
        Commands::PrintConfig => println!("{:#?}", &editor.config),
    }
}
