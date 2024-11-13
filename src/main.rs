use std::{io, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{
    aot::{Bash, Elvish, Fish, PowerShell, Zsh},
    generate,
};
use clap_complete_nushell::Nushell;
use colour::{cyan_ln_bold, green_ln_bold};
use editor::Editor;
use serde::{Deserialize, Serialize};

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
    dry_run: bool,
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
        /// Before building the project it will generate it first.
        #[clap(long, action)]
        generate_project: bool,
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
        /// Before building the project it will generate it first.
        #[clap(long, action)]
        generate_project: bool,
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
    GenerateCompletions {
        shell: Shell,
        /// default- print
        #[clap(action)]
        action: Option<ActionToDo>,
    },
}

#[derive(Clone, Debug, Default, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ActionToDo {
    #[default]
    Print,
    CopyToClipboard,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize, ValueEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Powershell,
    Zsh,
}

fn main() {
    let cli = Cli::parse();
    let mut editor = Editor::create(&cli);

    match &cli.command {
        Commands::GenerateCompletions { shell, action } => {
            let mut cmd = Cli::command();
            let mut b = [0u8; 10000];
            let mut buffer = io::BufWriter::new(b.as_mut());
            match shell {
                Shell::Bash => generate(Bash, &mut cmd, "uec", &mut buffer),
                Shell::Nushell => generate(Nushell, &mut cmd, "uec", &mut buffer),
                Shell::Powershell => generate(PowerShell, &mut cmd, "uec", &mut buffer),
                Shell::Elvish => generate(Elvish, &mut cmd, "uec", &mut buffer),
                Shell::Fish => generate(Fish, &mut cmd, "uec", &mut buffer),
                Shell::Zsh => generate(Zsh, &mut cmd, "uec", &mut buffer),
            }

            let bytes: Vec<u8> = buffer.buffer().to_vec();
            let string = String::from_utf8(bytes).unwrap();
            match action.unwrap_or_default() {
                ActionToDo::Print => println!("{}", string),
                ActionToDo::CopyToClipboard => {
                    let mut clipboard = arboard::Clipboard::new().expect("Failed to get clipboard");
                    clipboard
                        .set_text(&string)
                        .expect("Failed to update clipboard");
                    green_ln_bold!("Completions generated and copied to clipboard!");
                }
            }
        }
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
        Commands::Build {
            path,
            output,
            generate_project,
        } => {
            if *generate_project {
                editor.generate_proj_files(path);
            }
            let _ = editor.build_project(path, output);
        }
        Commands::BuildEngine { path } => editor.build_engine_from_source(path),
        Commands::CleanProject { path } => {
            let _ = editor
                .clean_project(path)
                .inspect_err(|e| eprintln!("Failed to clean up the project, reason: {:#?}", e));
        }
        Commands::EditorProject {
            path,
            generate_project,
        } => {
            if *generate_project {
                editor.generate_proj_files(path);
            }
            editor.build_editor_project(path)
        }
        Commands::GenerateProjectFiles { path } => editor.generate_proj_files(path),
        Commands::BuildPlugin { path, output } => editor.build_plugin(path, output),
        Commands::UAT { input, path } => {
            let args: Vec<&str> = input.split(' ').collect();
            let _ = editor.run_uat(path, args);
        }
        Commands::PrintConfig => println!("{:#?}", &editor.config),
    }
}
