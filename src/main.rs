use std::env;
use std::panic;
use std::process::{Stdio, Command};
use std::io::{BufRead, BufReader};
use std::os::unix::{ffi::OsStrExt, process::CommandExt};
use std::fs::{self, File};
use std::ffi::OsString;
use std::path::PathBuf;
use simple_eyre::eyre::{bail, eyre, Result, WrapErr};

fn home_manager_login_shell() -> Result<PathBuf> {
    let home_dir = env::var_os("HOME").map(PathBuf::from).ok_or_else(|| eyre!("HOME not set"))?;
    let login_shell = home_dir.join(".nix-profile/bin/login-shell");
    if !login_shell.exists() {
        bail!("Configured login shell {login_shell:?} does not exist");
    }
    let shell_check_status =
        Command::new(login_shell.clone())
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .wrap_err("Checking that {login_shell:?} works")?;
    if shell_check_status.success() {
        Ok(login_shell)
    } else {
        Err(eyre!(shell_check_status).wrap_err("Checking that {login_shell:?} works"))
    }
}

fn invoked_as_login_shell() -> Option<bool> {
    let argv0 = env::args_os().nth(0)?;
    Some(argv0.as_bytes().get(0) == Some(&b'-'))
}

fn run_home_manager_login_shell() -> Result<()> {
  let is_login_shell = invoked_as_login_shell().unwrap_or(false);
  let configured_login_shell = home_manager_login_shell()?;
  // We've already executed the shell so there must be a last path component
  let canonicalized_shell = fs::canonicalize(&configured_login_shell).wrap_err("Canonicalizing login-shell")?;
  let shell_name = canonicalized_shell.file_name().ok_or_else(|| eyre!("Canonical shell has no last component"))?;
  let mut shell_argv0 : OsString = if is_login_shell { "-".into() } else { OsString::new() };
  shell_argv0.push(shell_name);
  let shell_args: Vec<OsString> = env::args_os().skip(1).collect();
  bail!(Command::new(canonicalized_shell)
      .args(shell_args)
      .arg0(shell_argv0)
      .exec())
}

fn run_fallback_shell() -> Result<()> {
    let Ok(shells) = File::open("/etc/shells") else {
        bail!(Command::new("/bin/sh").exec())
    };
    let shells = BufReader::new(shells);
    for line in shells.lines().filter_map(Result::ok) {
        if line.starts_with("#") {
            continue
        }
        if line.is_empty() {
            continue
        }
        // Only consider "system" shells
        if !line.starts_with("/bin") {
            continue
        }
        Command::new(line).exec();
    }
    bail!(Command::new("/bin/sh").exec())
}

fn main() -> Result<()> {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Shell helper panicked, falling back: {s:?}");
        } else {
            eprintln!("Shell helper panicked, falling back");
        }
        let _ = run_fallback_shell();
    }));
    simple_eyre::install()?;
    match run_home_manager_login_shell() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Could not start configured shell: {e:#}");
            run_fallback_shell()
        }
    }
}
