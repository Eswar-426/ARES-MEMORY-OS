import re

file_path = 'crates/ares-git-memory/src/blame.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    code = f.read()

# Fix git log
old_log = """        let mut global_creation_cmd = Command::new("git");
        global_creation_cmd.current_dir(project_path).args([
            "log",
            "--name-status",
            "--pretty=format:COMMIT|%H|%an|%s|%at",
            "--reverse",
        ]);

        use std::process::Stdio;
        use wait_timeout::ChildExt;

        global_creation_cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Ok(mut child) = global_creation_cmd.spawn() {
            let timeout = std::time::Duration::from_secs(120);
            match child.wait_timeout(timeout) {
                Ok(Some(status)) => {
                    if status.success() {
                        let mut stdout = Vec::new();
                        if let Some(mut out) = child.stdout.take() {
                            use std::io::Read;
                            out.read_to_end(&mut stdout).ok();
                        }
                        let output_str = String::from_utf8_lossy(&stdout);"""

new_log = """        let mut global_creation_cmd = Command::new("git");
        global_creation_cmd.current_dir(project_path).args([
            "log",
            "--name-status",
            "--pretty=format:COMMIT|%H|%an|%s|%at",
            "--reverse",
            "--",
        ]);
        global_creation_cmd.args(&files);

        if let Ok(output) = global_creation_cmd.output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);"""

if old_log in code:
    code = code.replace(old_log, new_log)
else:
    print("WARNING: Could not find old_log block!")

# Remove the timeout branch for git log
old_log_timeout = """                    }
                }
                Ok(None) => {
                    child.kill().ok();
                    child.wait().ok();
                    println!("WARN: git log for creation commits timed out");
                }
                Err(e) => {
                    println!("WARN: Failed to wait for git log: {}", e);
                }
            }
        }"""
new_log_timeout = """            }
        }"""
if old_log_timeout in code:
    code = code.replace(old_log_timeout, new_log_timeout)
else:
    print("WARNING: Could not find old_log_timeout block!")

# Fix git blame
old_blame = """                // Run blame on each file
                let mut blame_cmd = Command::new("git");
                blame_cmd
                    .current_dir(project_path)
                    .args(["blame", "--line-porcelain", file]);

                use std::process::Stdio;
                use std::time::Duration;
                use wait_timeout::ChildExt;

                blame_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
                if let Ok(mut child) = blame_cmd.spawn() {
                    let timeout = Duration::from_secs(30);
                    match child.wait_timeout(timeout) {
                        Ok(Some(status)) => {
                            if status.success() {
                                let mut stdout = Vec::new();
                                if let Some(mut out) = child.stdout.take() {
                                    use std::io::Read;
                                    out.read_to_end(&mut stdout).ok();
                                }
                                let blame_str = String::from_utf8_lossy(&stdout);"""

new_blame = """                // Run blame on each file
                let mut blame_cmd = Command::new("git");
                blame_cmd
                    .current_dir(project_path)
                    .args(["blame", "--line-porcelain", file]);

                if let Ok(output) = blame_cmd.output() {
                    if output.status.success() {
                        let blame_str = String::from_utf8_lossy(&output.stdout);"""

if old_blame in code:
    code = code.replace(old_blame, new_blame)
else:
    print("WARNING: Could not find old_blame block!")

# Remove the timeout branch for git blame
old_blame_timeout = """                            }
                        }
                        Ok(None) => {
                            child.kill().ok();
                            child.wait().ok();
                            println!("WARN: git blame timed out for file: {}", file);
                        }
                        Err(e) => {
                            println!("WARN: Failed to wait for git blame for {}: {}", file, e);
                        }
                    }
                }"""
new_blame_timeout = """                    }
                }"""
if old_blame_timeout in code:
    code = code.replace(old_blame_timeout, new_blame_timeout)
else:
    print("WARNING: Could not find old_blame_timeout block!")

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(code)

print("Done replacing.")
