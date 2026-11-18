//! Spike: does a bundled static Git run after packaging, and do hooks see the terminal's PATH?

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use tauri::path::BaseDirectory;
use tauri::Manager;

type Env = HashMap<String, String>;

// ── Environment ──────────────────────────────────────────────────────────────

/// Printed before `env -0` to separate shell startup noise from the environment.
#[cfg(unix)]
const ENV_MARKER: &str = "__RAMOSO_ENV_START__";

/// The environment the user's terminal would have; `None` on failure or after 5 s.
#[cfg(unix)]
fn login_shell_env() -> Option<Env> {
    use std::io::Read;
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::time::Duration;

    // The user's shell, or /bin/sh.
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    // `-l -i` loads the terminal's startup files (where asdf/nvm set PATH).
    let script = format!("printf '%s' '{ENV_MARKER}'; env -0");
    let mut child = Command::new(shell)
        .args(["-l", "-i", "-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    // Read output on a helper thread so the wait can time out.
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        let _ = tx.send(bytes);
    });

    // Kill the shell if it hangs.
    let bytes = match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(bytes) => {
            let _ = child.wait();
            bytes
        }
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    };

    // Keep only what follows the marker.
    let marker = ENV_MARKER.as_bytes();
    let start = bytes.windows(marker.len()).position(|window| window == marker)? + marker.len();

    // Parse NUL-separated KEY=value entries.
    let env = bytes[start..]
        .split(|byte| *byte == 0)
        .filter_map(|entry| std::str::from_utf8(entry).ok())
        .filter_map(|entry| entry.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    Some(env)
}

/// Windows apps already get the user's environment.
#[cfg(not(unix))]
fn login_shell_env() -> Option<Env> {
    None
}

fn find_on_path(program: &str, env: &Env) -> Option<PathBuf> {
    // Windows variable names ignore case, and PATH is usually spelled `Path` there.
    let path = if cfg!(windows) {
        env.iter().find(|(key, _)| key.eq_ignore_ascii_case("PATH")).map(|(_, value)| value)
    } else {
        env.get("PATH")
    }?;

    // Search PATH folders in order, like the shell (`node.exe` on Windows).
    let file_name = format!("{program}{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(path)
        .map(|dir| dir.join(&file_name))
        .find(|candidate| candidate.is_file())
}

/// The system's CA bundle; our static curl has no default.
fn system_ca_bundle() -> Option<&'static str> {
    const CANDIDATES: [&str; 5] = [
        "/etc/ssl/certs/ca-certificates.crt", // Debian, Ubuntu, Arch, Alpine
        "/etc/pki/tls/certs/ca-bundle.crt",   // Fedora, RHEL
        "/etc/ssl/ca-bundle.pem",             // openSUSE
        "/etc/pki/tls/cacert.pem",
        "/etc/ssl/cert.pem",
    ];
    CANDIDATES.into_iter().find(|path| Path::new(path).is_file())
}

// ── Files ────────────────────────────────────────────────────────────────────

/// The Git program inside the bundle, and the files that must survive packaging.
#[cfg(unix)]
const GIT_PROGRAM: &str = "bin/git";
#[cfg(unix)]
const KEY_FILES: &[&str] = &["bin/git", "libexec/git-core/git-remote-http", "libexec/git-core/git-lfs"];

/// Git for Windows' launcher sets up its own paths, including `sh` for hooks.
#[cfg(not(unix))]
const GIT_PROGRAM: &str = "cmd/git.exe";
#[cfg(not(unix))]
const KEY_FILES: &[&str] = &[
    "cmd/git.exe",
    "ucrt64/bin/git-remote-https.exe",
    "ucrt64/libexec/git-core/git-lfs.exe",
    "usr/bin/sh.exe",
];

/// Unix permissions in octal (755 = executable).
#[cfg(unix)]
fn file_mode(path: &Path) -> io::Result<String> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path)?;
    Ok(format!("{:o}", meta.permissions().mode() & 0o777))
}

/// Windows has no executable bit; report presence only.
#[cfg(not(unix))]
fn file_mode(path: &Path) -> io::Result<String> {
    std::fs::metadata(path).map(|_| "present".to_owned())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

/// Git for Windows runs hooks through its own `sh`.
#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

// ── Running Git ──────────────────────────────────────────────────────────────

/// Where the bundled Git lives: `$APPDIR/git` in an AppImage, Tauri's resource folder elsewhere.
fn bundled_git_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // AppImage: quick-sharun rewrites Tauri's resource path, so use our own fixed location.
    if std::env::var_os("APPIMAGE").is_some() {
        if let Some(appdir) = std::env::var_os("APPDIR") {
            return Ok(PathBuf::from(appdir).join("git"));
        }
    }

    // Packages, Windows, macOS and development builds.
    let root = app.path().resolve("git", BaseDirectory::Resource).map_err(|e| e.to_string())?;
    Ok(plain_path(root))
}

/// Turns Windows' extended `\\?\E:\…` form back into `E:\…`; some tools can't read it.
fn plain_path(path: PathBuf) -> PathBuf {
    // Only drive-letter paths; `\\?\UNC\…` network paths need the long form.
    match path.to_str().and_then(|text| text.strip_prefix(r"\\?\")) {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => path,
    }
}

struct GitResult {
    exit_code: Option<i32>,
    output: String,
}

impl GitResult {
    fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

impl fmt::Display for GitResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.exit_code {
            Some(0) => write!(f, "ok  {}", self.output),
            Some(code) => write!(f, "FAILED (exit {code})  {}", self.output),
            None => write!(f, "FAILED to start  {}", self.output),
        }
    }
}

/// Runs the bundled Git with only `env`; nothing outside Ramoso can answer prompts.
fn run_git(git: &Path, env: &Env, repo: &Path, args: &[&str]) -> GitResult {
    // Drop config helpers, desktop ask-pass programs and terminal prompts.
    let mut command = Command::new(git);
    command
        .args(["-c", "credential.helper="])
        .args(args)
        .current_dir(repo)
        .env_clear()
        .envs(env)
        .env_remove("GIT_ASKPASS")
        .env_remove("SSH_ASKPASS")
        .env("GIT_TERMINAL_PROMPT", "0");

    // Windows: no console window for Git or the hooks it starts.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    // Keep the exit code and both outputs, one per line.
    match command.output() {
        Ok(output) => {
            let parts = [&output.stdout, &output.stderr].map(|bytes| String::from_utf8_lossy(bytes).trim().to_owned());
            GitResult {
                exit_code: output.status.code(),
                output: parts.into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("\n"),
            }
        }
        Err(error) => GitResult { exit_code: None, output: error.to_string() },
    }
}

/// Runs a step later steps depend on; fails with a clear message.
fn require_git(git: &Path, env: &Env, repo: &Path, args: &[&str]) -> Result<GitResult, String> {
    let result = run_git(git, env, repo, args);
    if result.succeeded() {
        Ok(result)
    } else {
        Err(format!("`git {}` {result}", args.join(" ")))
    }
}

// ── Checks ───────────────────────────────────────────────────────────────────

/// Are the bundled Git files still executable after packaging?
fn check_permissions(git_root: &Path, report: &mut Vec<String>) {
    // Where the package put our Git.
    report.push(format!("Bundled Git: {}", git_root.display()));

    // Permissions of the key files.
    for file in KEY_FILES {
        let mode = file_mode(&git_root.join(file)).unwrap_or_else(|error| format!("MISSING ({error})"));
        report.push(format!("  {file}: {mode}"));
    }
}

/// Which environment hooks will see; returns the one to use.
fn check_environment(report: &mut Vec<String>) -> Env {
    // What the app inherited (from the desktop, usually no asdf/nvm).
    let app_env: Env = std::env::vars().collect();
    report.push(format!("App PATH finds node:         {:?}", find_on_path("node", &app_env)));

    // What the terminal would have, else the app's own.
    let mut env = match login_shell_env() {
        Some(login_env) => {
            report.push(format!("Login-shell PATH finds node: {:?}", find_on_path("node", &login_env)));
            login_env
        }
        None if cfg!(windows) => {
            report.push("Login-shell environment: not needed on Windows".into());
            app_env
        }
        None => {
            report.push("Login-shell environment: FAILED or timed out, using the app's".into());
            app_env
        }
    };

    // Point HTTPS at the system's certificates.
    if let Some(ca_bundle) = system_ca_bundle() {
        env.insert("GIT_SSL_CAINFO".into(), ca_bundle.into());
    }
    report.push(format!("CA bundle: {:?}", env.get("GIT_SSL_CAINFO")));

    env
}

/// Commits in a fresh repo whose pre-commit hook calls `node`.
fn check_hook_commit(git: &Path, env: &Env, report: &mut Vec<String>) -> Result<PathBuf, String> {
    // A fresh repo in the temp folder.
    let repo = std::env::temp_dir().join("ramoso-spike-repo");
    let _ = std::fs::remove_dir_all(&repo);
    std::fs::create_dir_all(&repo).map_err(|e| format!("creating {}: {e}", repo.display()))?;

    // Report the Git version, then initialise.
    report.push(format!("Version: {}", require_git(git, env, &repo, &["--version"])?));
    require_git(git, env, &repo, &["init", "-q"])?;

    // A hook that blocks the commit unless `node` is on PATH.
    let hook_script = "#!/bin/sh\n\
        command -v node >/dev/null || { echo 'pre-commit hook: node NOT FOUND' >&2; exit 1; }\n\
        echo \"pre-commit hook: node $(node --version)\" >&2\n";
    let hook = repo.join(".git/hooks/pre-commit");
    std::fs::write(&hook, hook_script).map_err(|e| format!("writing the hook: {e}"))?;
    make_executable(&hook).map_err(|e| format!("making the hook executable: {e}"))?;

    // Stage one file.
    std::fs::write(repo.join("a.txt"), "spike\n").map_err(|e| format!("writing a.txt: {e}"))?;
    require_git(git, env, &repo, &["add", "a.txt"])?;

    // Commit (runs the hook); a failure here is a result, not a stop.
    let identity = ["-c", "user.name=spike", "-c", "user.email=spike@example.com"];
    let commit = run_git(git, env, &repo, &[&identity[..], &["commit", "-m", "spike"]].concat());
    report.push(format!("Commit with hook: {commit}"));

    Ok(repo)
}

/// HTTPS through our static curl with the system's certificates.
fn check_https(git: &Path, env: &Env, repo: &Path, report: &mut Vec<String>) {
    // Reads refs only; nothing is downloaded or written.
    let url = "https://github.com/octocat/Hello-World.git";

    report.push(format!("HTTPS ls-remote: {}", run_git(git, env, repo, &["ls-remote", url, "HEAD"])));
}

fn run_checks(app: &tauri::AppHandle) -> Result<String, String> {
    // Our Git inside the installed package.
    let git_root = bundled_git_root(app)?;
    let git = git_root.join(GIT_PROGRAM);
    let mut report = Vec::new();

    // Packaging kept the files executable.
    check_permissions(&git_root, &mut report);
    report.push(String::new());

    // Pick the environment hooks will run with.
    let env = check_environment(&mut report);
    report.push(String::new());

    // A real commit through a hook, then HTTPS; keep partial results on failure.
    match check_hook_commit(&git, &env, &mut report) {
        Ok(repo) => check_https(&git, &env, &repo, &mut report),
        Err(reason) => report.push(format!("Stopped: {reason}")),
    }

    Ok(report.join("\n"))
}

// ── Tauri ────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn run_spike_checks(app: tauri::AppHandle) -> Result<String, String> {
    // Git calls block; run them off the async threads.
    tauri::async_runtime::spawn_blocking(move || run_checks(&app))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_spike_checks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
