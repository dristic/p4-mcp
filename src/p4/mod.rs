use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use tracing::debug;

pub mod commands;

pub use commands::P4Command;

pub struct P4Handler {
    mock_mode: bool,
}

impl P4Handler {
    pub fn new() -> Self {
        Self {
            mock_mode: std::env::var("P4_MOCK_MODE").is_ok(),
        }
    }

    pub async fn execute(&mut self, command: P4Command) -> Result<String> {
        if self.mock_mode {
            self.execute_mock(command).await
        } else {
            self.execute_real(command).await
        }
    }

    async fn execute_real(&mut self, command: P4Command) -> Result<String> {
        let (cmd, args) = command.to_command_args();

        debug!("Executing p4 command: {} {:?}", cmd, args);

        let output = Command::new("p4")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("p4 command failed: {}", stderr))
        }
    }

    async fn execute_mock(&mut self, command: P4Command) -> Result<String> {
        debug!("Mock executing p4 command: {:?}", command);

        match command {
            P4Command::Status { path } => {
                let path_info = path.unwrap_or("current directory".to_string());
                Ok(format!(
                    "Mock P4 Status for {}:\n\
                     //depot/main/file1.txt#1 - edit default change (text)\n\
                     //depot/main/file2.cpp#2 - add default change (text)\n\
                     ... (mock data)",
                    path_info
                ))
            }

            P4Command::Sync { path, force } => {
                let force_flag = if force { " (forced)" } else { "" };
                Ok(format!(
                    "Mock P4 Sync{}:\n\
                     //depot/main/{}#1 - updating /local/workspace/file1.txt\n\
                     //depot/main/{}#2 - updating /local/workspace/file2.cpp\n\
                     ... synced 15 files",
                    force_flag, path, path
                ))
            }

            P4Command::Edit { files } => {
                let file_list = files.join(", ");
                Ok(format!(
                    "Mock P4 Edit:\n\
                     Files opened for edit:\n\
                     {}\n\
                     ... {} file(s) opened for edit",
                    file_list,
                    files.len()
                ))
            }

            P4Command::Add { files } => {
                let file_list = files.join(", ");
                Ok(format!(
                    "Mock P4 Add:\n\
                     Files opened for add:\n\
                     {}\n\
                     ... {} file(s) opened for add",
                    file_list,
                    files.len()
                ))
            }

            P4Command::Submit { description, files } => {
                let file_info = if let Some(files) = files {
                    format!("Specific files: {}", files.join(", "))
                } else {
                    "All opened files".to_string()
                };
                Ok(format!(
                    "Mock P4 Submit:\n\
                     Change description: {}\n\
                     Files: {}\n\
                     Change 12345 submitted successfully",
                    description, file_info
                ))
            }

            P4Command::Revert { files } => {
                let file_list = files.join(", ");
                Ok(format!(
                    "Mock P4 Revert:\n\
                     Files reverted:\n\
                     {}\n\
                     ... {} file(s) reverted",
                    file_list,
                    files.len()
                ))
            }

            P4Command::Opened { changelist } => {
                let cl_info = if let Some(cl) = changelist {
                    format!(" in changelist {}", cl)
                } else {
                    String::new()
                };
                Ok(format!(
                    "Mock P4 Opened{}:\n\
                     //depot/main/file1.txt#1 - edit default change (text)\n\
                     //depot/main/file2.cpp#2 - add default change (text)\n\
                     //depot/main/file3.h#1 - edit change 12346 (text)",
                    cl_info
                ))
            }

            P4Command::Changes { max, path } => {
                let path_info = if let Some(path) = path {
                    format!(" for path {}", path)
                } else {
                    String::new()
                };

                let mut result = format!("Mock P4 Changes (max: {}){}:\n", max, path_info);

                for i in 0..std::cmp::min(max, 5) {
                    let change_num = 12350 - i;
                    result.push_str(&format!(
                        "Change {} on 2024/01/1{} by user@workspace 'Sample change description {}'\n",
                        change_num,
                        15 + i,
                        i + 1
                    ));
                }

                Ok(result)
            }

            P4Command::Info => Ok(format!(
                "Mock P4 Info:\n\
                     User name: testuser\n\
                     Client name: test-client\n\
                     Client host: test-host\n\
                     Client root: C:\\workspace\\p4\\test-client\n\
                     Current directory: C:\\workspace\\p4\\test-client\\main\n\
                     Peer address: ssl:perforce.example.com:1666\n\
                     Client address: 192.168.1.100\n\
                     Server address: perforce.example.com:1666\n\
                     Server root: /opt/perforce/depot\n\
                     Server date: 2024/01/15 12:30:45 -0800 PST\n\
                     Server uptime: 15:32:18\n\
                     Server version: P4D/LINUX26X86_64/2023.1/2553040 (2023/06/15)\n\
                     ServerID: perforce-server\n\
                     Case Handling: insensitive"
            )),
        }
    }
}

impl Default for P4Handler {
    fn default() -> Self {
        Self::new()
    }
}
