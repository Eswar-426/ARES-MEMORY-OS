import re

file_path = 'crates/ares-git-memory/src/blame.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    code = f.read()

# Fix git log
old_log = """        global_creation_cmd.current_dir(project_path).args([
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

new_log = """        global_creation_cmd.current_dir(project_path).args([
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

code = code.replace(old_log, new_log)

old_log_timeout = """                    }
                }
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    println!("WARN: git log for creation commits timed out");
                }
                Err(e) => {
                    println!("WARN: wait_timeout error for creation commits: {}", e);
                }
            }
        }"""
new_log_timeout = """                    }
                }
            }
        }"""
code = code.replace(old_log_timeout, new_log_timeout)

# Fix git blame
old_blame = """                blame_cmd
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
new_blame = """                blame_cmd
                    .current_dir(project_path)
                    .args(["blame", "--line-porcelain", file]);

                if let Ok(output) = blame_cmd.output() {
                    if output.status.success() {
                        let blame_str = String::from_utf8_lossy(&output.stdout);"""

code = code.replace(old_blame, new_blame)

old_blame_timeout = """                            }
                        }
                        Ok(None) => {
                            let _ = child.kill();
                            let _ = child.wait();
                            println!("WARN: git blame timed out for file: {}", file);
                        }
                        Err(e) => {
                            println!("WARN: wait_timeout error for git blame: {}", e);
                        }
                    }
                }"""
new_blame_timeout = """                            }
                        }
                    }
                }"""
code = code.replace(old_blame_timeout, new_blame_timeout)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(code)

print("Replaced code!")
