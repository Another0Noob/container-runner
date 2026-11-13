use std::process::Command;

fn main() {
    let out = run_git_commit().unwrap();
    println!("{}", out);
}

fn run_git_commit() -> Result<String, String> {
    // Build the command with arguments and working directory.
    // current_dir changes where the process starts.
    let output = Command::new("git")
        .arg("status")
        .current_dir(".")
        .output()
        .map_err(|e| format!("Failed to spawn git: {}", e))?;

    // Check the exit status.
    // git returns non-zero on failure and writes details to stderr.
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git commit failed: {}", err_msg.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
