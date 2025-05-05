use std::collections::HashMap;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_process(&self, server_name: &str, command: Vec<&str>) -> (ChildStdin, ChildStdout) {
        let mut processes = self.processes.lock().unwrap();

        if let Some(process) = processes.get_mut(server_name) {
            let stdin = process.stdin.take().unwrap();
            let stdout = process.stdout.take().unwrap();
            return (stdin, stdout);
        }

        let mut cmd = Command::new(command[0]);
        for arg in command.iter().skip(1) {
            cmd.arg(arg);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("failed to spawn process");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        processes.insert(server_name.to_string(), child);
        (stdin, stdout)
    }

    pub async fn stop_process(&self, server_name: &str) {
        let mut processes = self.processes.lock().unwrap();
        if let Some(mut process) = processes.remove(server_name) {
            let _ = process.kill().await;
        }
    }
}