use super::{ServiceBackend, ServiceStatus};
use std::path::PathBuf;
use std::process::Command;

pub struct CronBackend;

impl CronBackend {
    pub fn new() -> Self {
        CronBackend
    }

    fn service_dir(&self, service_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        use crate::config::Config;
        let config = Config::load()?;
        Ok(config.services_dir()?.join(service_name))
    }

    fn pid_file(&self, service_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(self.service_dir(service_name)?.join("run").join("pid"))
    }

    fn read_pid(&self, service_name: &str) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let pid_file = self.pid_file(service_name)?;
        if !pid_file.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(pid_file)?;
        let pid = content.trim().parse().ok();
        Ok(pid)
    }

    fn is_running(&self, pid: u32) -> bool {
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn daemon_start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let service_dir = self.service_dir(service_name)?;
        let run_dir = service_dir.join("run");
        std::fs::create_dir_all(&run_dir)?;

        if let Some(pid) = self.read_pid(service_name)? {
            if self.is_running(pid) {
                return Err(
                    format!("Service {} already running with PID {}", service_name, pid).into(),
                );
            }
        }

        let logs_dir = service_dir.join("logs");
        std::fs::create_dir_all(&logs_dir)?;

        let script_path = service_dir.join("script.sh");
        let stdout_log = logs_dir.join("stdout.log");
        let stderr_log = logs_dir.join("stderr.log");

        let mut cmd = Command::new("nohup");
        cmd.arg(&script_path)
            .arg(format!(">> {}", stdout_log.display()))
            .arg(format!("2>> {}", stderr_log.display()))
            .arg("&")
            .current_dir(&service_dir);

        let child = cmd.spawn()?;
        let pid = child.id();

        let pid_file = self.pid_file(service_name)?;
        std::fs::write(pid_file, pid.to_string())?;

        Ok(())
    }

    pub fn daemon_stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pid) = self.read_pid(service_name)? {
            if self.is_running(pid) {
                std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .output()?;

                for _ in 0..10 {
                    if !self.is_running(pid) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }

                if self.is_running(pid) {
                    std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output()?;
                }
            }
        }

        let pid_file = self.pid_file(service_name)?;
        if pid_file.exists() {
            std::fs::remove_file(pid_file)?;
        }

        Ok(())
    }

    pub fn onetime_enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let service_dir = self.service_dir(service_name)?;
        let script_path = service_dir.join("script.sh");
        let logs_dir = service_dir.join("logs");

        let stdout_log = logs_dir.join("stdout.log");
        let stderr_log = logs_dir.join("stderr.log");

        let output = Command::new("crontab").arg("-l").output()?;

        let mut lines = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let marker = format!("# htbox-onetime-{}", service_name);
        if !lines.iter().any(|l| l.contains(&marker)) {
            let job = format!(
                "@reboot {} >> {} 2>> {}",
                script_path.display(),
                stdout_log.display(),
                stderr_log.display()
            );
            lines.push(marker);
            lines.push(job);
        }

        let input = std::process::Stdio::piped();
        let mut child = Command::new("crontab").arg("-").stdin(input).spawn()?;

        use std::io::Write;
        let mut input = child.stdin.take().unwrap();
        for line in &lines {
            writeln!(input, "{}", line)?;
        }
        drop(input);
        child.wait()?;

        Ok(())
    }

    pub fn onetime_disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("crontab").arg("-l").output()?;

        let lines: Vec<String> = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| !l.contains(&format!("htbox-onetime-{}", service_name)))
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        };

        let input = std::process::Stdio::piped();
        let mut child = Command::new("crontab").arg("-").stdin(input).spawn()?;

        use std::io::Write;
        let mut input = child.stdin.take().unwrap();
        for line in &lines {
            writeln!(input, "{}", line)?;
        }
        drop(input);
        child.wait()?;

        Ok(())
    }
}

impl ServiceBackend for CronBackend {
    fn start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;

        if config.service_type == "daemon" {
            self.daemon_start(service_name)
        } else {
            Ok(())
        }
    }

    fn stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;

        if config.service_type == "daemon" {
            self.daemon_stop(service_name)
        } else {
            Ok(())
        }
    }

    fn restart(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.stop(service_name)?;
        self.start(service_name)
    }

    fn enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;

        if config.service_type == "onetime" {
            self.onetime_enable(service_name)
        } else {
            Ok(())
        }
    }

    fn disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::ServiceConfig;
        let config = ServiceConfig::load(service_name)?;

        if config.service_type == "onetime" {
            self.onetime_disable(service_name)
        } else {
            Ok(())
        }
    }

    fn status(&self, service_name: &str) -> Result<ServiceStatus, Box<dyn std::error::Error>> {
        let pid = self.read_pid(service_name)?;
        let running = pid.map(|p| self.is_running(p)).unwrap_or(false);

        Ok(ServiceStatus {
            running,
            pid,
            enabled: false,
        })
    }

    fn is_enabled(&self, service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let output = Command::new("crontab").arg("-l").output()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            Ok(content.contains(&format!("htbox-onetime-{}", service_name)))
        } else {
            Ok(false)
        }
    }
}
