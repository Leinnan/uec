use colour::{dark_green_ln_bold, e_red_ln, print_ln_bold, yellow_ln_bold};
use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{config::Config, consts, uproject, Cli};

pub struct Editor {
    pub config: Config,
    pub logs: Option<PathBuf>,
    pub error_only: bool,
}

impl Editor {
    pub fn create(cli: &Cli) -> Self {
        let mut config = Config::load_or_create();
        if let Some(engine) = &cli.engine_path {
            config.editor_path = engine.to_str().unwrap().into();
        };

        Editor {
            config,
            logs: cli.save_logs.clone(),
            error_only: cli.error_only,
        }
    }

    pub fn build_editor_exec(base_dir: &str) -> Option<PathBuf> {
        let editor = Path::new(base_dir).join(consts::EDITOR);
        if editor.exists() {
            Some(editor)
        } else {
            None
        }
    }

    pub fn get_editor_exec(&self) -> Option<PathBuf> {
        Self::build_editor_exec(&self.config.editor_path)
    }

    pub fn clean_project(&self, path: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        let project_path = find_uproject_file(path);
        let Some(project_path) = project_path else {
            panic!("PROJECT AT PATH DOES NOT EXIST!");
        };
        let sln_file = project_path.with_extension("sln");
        let parent = project_path.parent().expect("");
        if sln_file.exists() {
            if !self.error_only {
                print_ln_bold!("Removing: {}", sln_file.display());
            }
            std::fs::remove_file(sln_file)?;
        }
        let dirs_to_remove = [
            "Build",
            "Intermediate",
            "Saved",
            "DerivedDataCache",
            "PackagedProject",
        ];
        for dir in dirs_to_remove {
            let path = parent.join(dir);
            if path.exists() {
                if !self.error_only {
                    print_ln_bold!("Removing: {}", path.display());
                }
                std::fs::remove_dir_all(path)?;
            }
        }

        Ok(())
    }

    pub fn build_engine_from_source(&self, dir: &Option<PathBuf>) {
        let start = Instant::now();
        // TODO: Unix support.
        let dir = dir
            .clone()
            .unwrap_or(std::env::current_dir().expect("Failed to get current directory."));
        if !dir.join("Setup.bat").exists() || !dir.join("GenerateProjectFiles.bat").exists() {
            panic!(
                "WRONG DIRECTORY, it should be run from Unreal Engine Source code root directory"
            );
        }
        let cmd = Command::new("cmd")
            .arg("/C")
            .arg(dir.join("Setup.bat"))
            .run_with_async_logs(&self.logs, self.error_only);
        if !cmd.success() {
            panic!("FAILED TO RUN SETUP");
        }

        let cmd = Command::new("cmd")
            .arg("/C")
            .arg(dir.join("GenerateProjectFiles.bat"))
            .run_with_async_logs(&self.logs, self.error_only);
        if !cmd.success() {
            panic!("FAILED TO GENERATE PROJECT FILES");
        }
        let cmd = Command::new("cmd")
            .arg("/C")
            .arg("msbuild")
            .arg(dir.join("UE5.sln"))
            .arg("/p:Configuration=\"Development Editor\"")
            .arg("/p:Platform=\"Win64\"")
            .run_with_async_logs(&self.logs, self.error_only);
        if !cmd.success() {
            panic!("FAILED TO BUILD ENGINE");
        }
        let duration = start.elapsed();
        dark_green_ln_bold!("ENGINE BUILED SUCCESSFULLY! Duration: {:?}", duration);
    }

    pub fn run_editor(&self) {
        let path = self
            .get_editor_exec()
            .expect("Editor at path does not exists");
        let _ = std::process::Command::new(path.to_str().unwrap()).spawn();
    }
    pub fn build_editor_project(&self, path: &Option<PathBuf>) {
        let project_path = find_uproject_file(path);
        let Some(project_path) = project_path else {
            panic!("PROJECT AT PATH DOES NOT EXIST!");
        };
        let project_path = project_path
            .to_str()
            .unwrap()
            .to_owned()
            .replace("\\\\?\\", "");

        let p = format!("-Project={}", &project_path);
        let build_path = Path::new(&self.config.editor_path).join(consts::BUILD_SCRIPT);

        println!("Building project: {}", &project_path);
        let uproject = uproject::read_config(&project_path).unwrap();
        let module = uproject.find_editor_module().unwrap();

        let mut bind = Command::new("cmd");
        let cmd = bind
            .args(["/C", (build_path.to_str().unwrap())])
            .arg(&module.Name)
            .arg("Win64")
            .arg("Development")
            .arg(&p)
            .arg("-UsePrecompiled")
            .arg("-WaitMutex")
            .arg("-FromMsBuild");

        if cmd
            .run_with_async_logs(&self.logs, self.error_only)
            .success()
        {
            let path = self
                .get_editor_exec()
                .expect("Editor at path does not exists");
            let mut bind = Command::new("cmd");
            bind.args(["/C", path.to_str().unwrap()])
                .arg(&project_path)
                .arg("-skipcompile")
                .run_in_bg();
        }
    }

    pub fn build_project(&self, path: &Option<PathBuf>) {
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
        let build_path = Path::new(&self.config.editor_path).join(consts::UAT_SCRIPT);

        println!("Building project: {}", &project_path.display());

        let mut bind = Command::new("cmd");
        let cmd = bind
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
            .arg(&arch);

        cmd.run_with_async_logs(&self.logs, self.error_only);
    }

    pub fn build_plugin(&self, uplugin_path: &Option<PathBuf>, output_dir: &Option<PathBuf>) {
        let Some(project_path) = find_file_by_extension(uplugin_path, "uplugin") else {
            panic!("Plugin at given path does not exist!");
        };
        let output = output_dir
            .clone()
            .unwrap_or_else(|| std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap());

        let p = format!(
            "-plugin={}",
            project_path.to_str().expect("Failed to get project path.")
        )
        .replace("\\\\?\\", "");
        let tmp = format!("-package={}", output.to_str().unwrap()).replace("\\\\?\\", "");
        let build_path = Path::new(&self.config.editor_path).join(consts::UAT_SCRIPT);
        let mut bind = Command::new("cmd");
        let cmd = bind
            .args(["/C", (build_path.to_str().unwrap())])
            .arg("BuildPlugin")
            .arg(&p)
            .arg(&tmp)
            .arg("-CreateSubfolder");
        cmd.run_with_async_logs(&self.logs, self.error_only);
    }

    pub fn generate_proj_files(&self, path: &Option<PathBuf>) {
        let project_path = find_uproject_file(path);
        let Some(project_path) = project_path else {
            panic!("PROJECT AT PATH DOES NOT EXIST!");
        };

        let p = format!(
            "-project={}",
            project_path.to_str().expect("Failed to get project path.")
        )
        .replace("\\\\?\\", "");
        let build_path = Path::new(&self.config.editor_path)
            .join(consts::BUILD_SCRIPT)
            .to_str()
            .unwrap()
            .to_owned();

        let mut bind = Command::new("cmd");
        let cmd = bind
            .args(["/C", &build_path])
            .arg("-projectfiles")
            .arg(&p)
            .arg("-game")
            .arg("-rocket")
            .arg("-progress");

        cmd.run_with_async_logs(&self.logs, self.error_only);
    }
}

fn find_uproject_file(dir: &Option<PathBuf>) -> Option<PathBuf> {
    find_file_by_extension(dir, "uproject")
}

fn find_file_by_extension(dir: &Option<PathBuf>, extension: &str) -> Option<PathBuf> {
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
            if let Some(ext) = path.extension() {
                if ext == extension {
                    let path = std::fs::canonicalize(path).unwrap();
                    return Some(path);
                }
            }
        }
    }
    None
}

trait CmdHelper {
    fn run_with_async_logs(&mut self, logs_path: &Option<PathBuf>, error_only: bool) -> ExitStatus;
    fn run_in_bg(&mut self);
}
impl CmdHelper for Command {
    fn run_with_async_logs(&mut self, logs_path: &Option<PathBuf>, error_only: bool) -> ExitStatus {
        println!("Command to be run: {:?}", self);

        let mut child = self
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to execute command");

        // Create shared log buffer
        let shared_log = Arc::new(Mutex::new(String::new()));
        // Get the stdout and stderr of the child process
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        // Create readers for the stdout and stderr
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Spawn a thread to handle stdout
        let stdout_log = Arc::clone(&shared_log);

        let stdout_handle = std::thread::spawn(move || {
            for line in stdout_reader.lines() {
                match line {
                    Ok(line) => {
                        if error_only {
                            if !line.contains("): error C") {
                                continue;
                            }
                            println!("{}", line);
                            let mut log = stdout_log.lock().unwrap();
                            log.push_str(&line);
                            log.push('\n');
                            continue;
                        }
                        if line.contains("): warning C") {
                            yellow_ln_bold!("{}", line);
                        } else if line.contains("): error C") {
                            e_red_ln!("{}", line);
                        } else {
                            println!("{}", line);
                        }
                        let mut log = stdout_log.lock().unwrap();
                        log.push_str(&line);
                        log.push('\n');
                    }
                    Err(err) => eprintln!("Error reading stdout: {}", err),
                }
            }
        });

        // Spawn a thread to handle stderr
        let stderr_log = Arc::clone(&shared_log);

        let stderr_handle = std::thread::spawn(move || {
            for line in stderr_reader.lines() {
                match line {
                    Ok(line) => {
                        e_red_ln!("{}", line);
                        let mut log = stderr_log.lock().unwrap();
                        log.push_str(&line);
                        log.push('\n');
                    }
                    Err(err) => eprintln!("Error reading stderr: {}", err),
                }
            }
        });

        // Wait for the child process to exit
        let status = child.wait().expect("Child process wasn't running");

        // Wait for the threads to finish
        stdout_handle.join().expect("Failed to join stdout thread");
        stderr_handle.join().expect("Failed to join stderr thread");
        if let Some(path) = logs_path {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Failed to open file");
            let log_output = Arc::try_unwrap(shared_log)
                .expect("Failed to unwrap Arc")
                .into_inner()
                .unwrap();
            writeln!(file, "{}", log_output).expect("Failed to write to file");
        }

        // Print the exit status
        println!("Command exited with status: {}", status);
        status
    }

    fn run_in_bg(&mut self) {
        self.spawn().unwrap();
    }
}
