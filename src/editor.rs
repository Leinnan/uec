use std::{
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{config::Config, consts};

pub fn get_editor_exec(base_dir: &str) -> Option<PathBuf> {
    let editor = Path::new(base_dir).join(consts::EDITOR);
    if editor.exists() {
        Some(editor)
    } else {
        None
    }
}

pub fn run_editor(config: &Config) {
    let path = get_editor_exec(&config.editor_path).expect("Editor at path does not exists");
    let _ = std::process::Command::new(path.to_str().unwrap()).spawn();
}

pub fn build_project(config: &Config, path: &Option<PathBuf>) {
    let project_path = find_uproject_file(path);
    let Some(project_path) = project_path else {
        panic!("PROJECT AT PATH DOES NOT EXIST!");
    };
    let archived_dir = Path::new(project_path.parent().unwrap()).join("SuperPackage");

    let p = format!(
        "-project={}",
        project_path.to_str().expect("Failed to get project path.")
    )
    .replace("\\\\?\\", "");
    let arch = format!(
        "-archivedirectory={}",
        archived_dir.to_str().expect("Failed to get project path.")
    )
    .replace("\\\\?\\", "");
    let build_path = Path::new(&config.editor_path).join(consts::UAT_SCRIPT);

    println!("Building project: {}", &project_path.display());

    let output = Command::new("cmd")
        .args(["/C", (build_path.to_str().unwrap())])
        .arg("BuildCookRun")
        .arg(&p)
        .arg("-utf8output")
        .arg("-platform=Win64")
        .arg("-noP4")
        .arg("-nodebuginfo")
        .arg("-cook")
        .arg("-build")
        .arg("-stage")
        .arg("-archive")
        .arg("-pak")
        .arg(&arch)
        .output()
        .expect("Failed to execute command");

    // Print the output of the script
    println!("Status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

pub fn generate_proj_files(config: &Config, path: &Option<PathBuf>) {
    let project_path = find_uproject_file(path);
    let Some(project_path) = project_path else {
        panic!("PROJECT AT PATH DOES NOT EXIST!");
    };

    let p = format!(
        "-project={}",
        project_path.to_str().expect("Failed to get project path.")
    )
    .replace("\\\\?\\", "");
    let build_path = Path::new(&config.editor_path).join(consts::BUILD_SCRIPT);

    let mut child = Command::new("cmd")
        .args(["/C", (build_path.to_str().unwrap())])
        .arg("-projectfiles")
        .arg(&p)
        .arg("-game")
        .arg("-rocket")
        .arg("-progress")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    // Get the stdout and stderr of the child process
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Create readers for the stdout and stderr
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Spawn a thread to handle stdout
    let stdout_handle = std::thread::spawn(move || {
        for line in stdout_reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(err) => eprintln!("Error reading stdout: {}", err),
            }
        }
    });

    // Spawn a thread to handle stderr
    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_reader.lines() {
            match line {
                Ok(line) => eprintln!("{}", line),
                Err(err) => eprintln!("Error reading stderr: {}", err),
            }
        }
    });

    // Wait for the child process to exit
    let status = child.wait().expect("Child process wasn't running");

    // Wait for the threads to finish
    stdout_handle.join().expect("Failed to join stdout thread");
    stderr_handle.join().expect("Failed to join stderr thread");

    // Print the exit status
    println!("Command exited with status: {}", status);
}

fn find_uproject_file(dir: &Option<PathBuf>) -> Option<PathBuf> {
    let path = dir
        .clone()
        .unwrap_or(std::env::current_dir().expect("Failed to get current directory."));
    // Read the directory contents
    let entries = std::fs::read_dir(path).expect("Failed to read directory");

    // Iterate over the directory entries
    for entry in entries {
        let entry = entry.expect("Failed to get directory entry");
        let path = entry.path();

        // Check if the entry is a file with .uproject extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "uproject" {
                    let path = std::fs::canonicalize(path).unwrap();
                    return Some(path);
                }
            }
        }
    }

    None
}
