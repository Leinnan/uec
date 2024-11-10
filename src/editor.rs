use colour::{cyan_ln_bold, dark_green_ln_bold, e_red_ln, print_ln_bold, yellow_ln_bold};
use std::{
    error::Error,
    ffi::OsStr,
    io::{self, BufRead, BufReader, Write},
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
    /// No command would be run. Instead it will just output what it would run.
    dry_run: bool,
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
            dry_run: cli.dry_run,
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
        let Ok(project_path) = project_path else {
            panic!("PROJECT AT PATH DOES NOT EXIST!");
        };
        let sln_file = project_path.with_extension("sln");
        let parent = project_path.parent().expect("");
        self.remove_at_path(sln_file)?;
        let dirs_to_remove = [
            "Build",
            "Intermediate",
            "Saved",
            "DerivedDataCache",
            "PackagedProject",
        ];
        for dir in dirs_to_remove {
            let path = parent.join(dir);
            self.remove_at_path(path)?;
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
            .run_with_async_logs(self);
        if !cmd.success() {
            panic!("FAILED TO RUN SETUP");
        }

        let cmd = Command::new("cmd")
            .arg("/C")
            .arg(dir.join("GenerateProjectFiles.bat"))
            .run_with_async_logs(self);
        if !cmd.success() {
            panic!("FAILED TO GENERATE PROJECT FILES");
        }
        let cmd = Command::new("cmd")
            .arg("/C")
            .arg("msbuild")
            .arg(dir.join("UE5.sln"))
            .arg("/p:Configuration=\"Development Editor\"")
            .arg("/p:Platform=\"Win64\"")
            .run_with_async_logs(self);
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
        let Ok(project_path) = project_path else {
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

        if cmd.run_with_async_logs(self).success() {
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

    pub fn build_project(
        &self,
        path: &Option<PathBuf>,
        output_path: &Option<PathBuf>,
    ) -> Result<ExitStatus, Box<dyn Error>> {
        let archived_dir = output_path.clone().unwrap_or_else(|| {
            let p = find_file_by_extension(path, "uplugin").unwrap();

            Path::new(p.parent().unwrap()).join("CookedBuild")
        });

        let arch = format!(
            "-archivedirectory={}",
            archived_dir.to_str().expect("Failed to get project path.")
        )
        .replace("\\\\?\\", "");
        let args: Vec<&str> = vec![
            "BuildCookRun",
            &arch,
            "-utf8output",
            "-platform=Win64",
            "-noP4",
            "-nodebuginfo",
            "-cook",
            "-build",
            "-stage",
            "-archive",
            "-pak",
        ];

        self.run_uat(path, args)
    }

    pub fn build_plugin(&self, uplugin_path: &Option<PathBuf>, output_dir: &Option<PathBuf>) {
        let Ok(project_path) = find_file_by_extension(uplugin_path, "uplugin") else {
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
        cmd.run_with_async_logs(self);
    }

    pub fn run_uat<I, S>(
        &self,
        path: &Option<PathBuf>,
        args: I,
    ) -> Result<ExitStatus, Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let build_path = Path::new(&self.config.editor_path).join(consts::UAT_SCRIPT);
        let mut bind = Command::new("cmd");
        let mut args = args.into_iter();
        let pass_project_path = args.any(|s| {
            s.as_ref()
                .to_str()
                .is_some_and(|v| v.starts_with("-project="))
        });
        let mut cmd = bind.args(["/C", (build_path.to_str().unwrap())]).args(args);

        if pass_project_path {
            let Ok(project_path) = find_uproject_file(path) else {
                return Err("PROJECT AT GIVEN PATH DOES NOT EXIST".into());
            };
            let project_arg =
                format!("-project={}", project_path.to_str().unwrap()).replace("\\\\?\\", "");
            cmd = cmd.arg(project_arg);
        }
        let exit_code = cmd.run_with_async_logs(self);
        Ok(exit_code)
    }

    pub fn generate_proj_files(&self, path: &Option<PathBuf>) {
        let project_path = find_uproject_file(path);
        let Ok(project_path) = project_path else {
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

        cmd.run_with_async_logs(self);
    }

    fn remove_at_path<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(());
        }
        if self.dry_run {
            cyan_ln_bold!("[DRY_RUN] Removing: {}", path.display());
            return Ok(());
        }
        if !self.error_only {
            print_ln_bold!("Removing: {}", path.display());
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }
}

fn find_uproject_file(dir: &Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    find_file_by_extension(dir, "uproject")
}

fn find_file_by_extension(
    dir: &Option<PathBuf>,
    extension: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let path = dir.clone().unwrap_or(std::env::current_dir()?);
    // Read the directory contents
    let entries = std::fs::read_dir(path)?;

    // Iterate over the directory entries
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();

        // Check if the entry is a file with .uproject extension
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    let path = std::fs::canonicalize(path)?;
                    return Ok(path);
                }
            }
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not find file with extension in this dir",
    )))
}

trait CmdHelper {
    fn run_with_async_logs(&mut self, editor: &Editor) -> ExitStatus;
    fn run_in_bg(&mut self);
}
impl CmdHelper for Command {
    fn run_with_async_logs(&mut self, editor: &Editor) -> ExitStatus {
        if editor.dry_run {
            cyan_ln_bold!("[DRY_RUN] {:?}", self);
            return ExitStatus::default();
        }
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
        let error_only = editor.error_only;

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
        if let Some(path) = &editor.logs {
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
