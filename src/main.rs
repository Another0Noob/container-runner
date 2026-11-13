use std::process::Command;

fn main() {
    let out = podman_version().unwrap();
    println!("{}", out);
}

fn podman_version() -> Result<String, String> {
    // Build the command with arguments and working directory.
    // current_dir changes where the process starts.
    let output = Command::new("podman")
        .arg("-v")
        .current_dir(".")
        .output()
        .map_err(|e| format!("Failed to spawn podman: {}", e))?;

    // Check the exit status.
    // git returns non-zero on failure and writes details to stderr.
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Podman commit failed: {}", err_msg.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
