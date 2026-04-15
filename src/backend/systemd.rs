use super::{ServiceBackend, ServiceStatus};
use std::path::PathBuf;
use std::process::Command;

pub struct SystemdBackend {
    user_level: bool,
}

impl SystemdBackend {
    pub fn new() -> Self {
        let user_level = std::env::var("HTBOX_USER_LEVEL").unwrap_or_default() == "true";
        SystemdBackend { user_level }
    }

    fn systemctl_cmd(&self) -> Command {
        let mut cmd = Command::new("systemctl");
        if self.user_level {
            cmd.arg("--user");
        }
        cmd
    }

    fn service_name(&self, name: &str) -> String {
        format!("htbox-{}.service", name)
    }

    pub fn generate_unit_file(
        &self,
        service_name: &str,
        service_config: &crate::config::ServiceConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let service_dir = service_config.service_dir()?;

        let is_onetime = service_config.service_type == "onetime";

        let (service_type, remain_after_exit) = if is_onetime {
            ("oneshot", "Yes")
        } else {
            ("simple", "no")
        };

        let unit = format!(
            r#"[Unit]
Description=htbox - {}

[Service]
Type={}
{}
WorkingDirectory={}
EnvironmentFile={}
ExecStart={}
{}
Restart={}
RestartSec={}
StandardOutput=append:{}
StandardError=append:{}

[Install]
WantedBy=multi-user.target
"#,
            service_config
                .description
                .as_deref()
                .unwrap_or(service_name),
            service_type,
            service_config
                .user
                .as_ref()
                .map(|u| format!("User={}", u))
                .unwrap_or_default(),
            service_dir.display(),
            service_config.env_file_path()?.display(),
            service_config.script_path()?.display(),
            if is_onetime {
                format!("RemainAfterExit={}", remain_after_exit)
            } else {
                String::new()
            },
            service_config
                .restart_policy
                .as_deref()
                .unwrap_or("on-failure"),
            service_config.restart_delay.unwrap_or(5),
            service_config.stdout_log()?.display(),
            service_config.stderr_log()?.display(),
        );
        Ok(unit)
    }

    pub fn install_unit(
        &self,
        service_name: &str,
        service_config: &crate::config::ServiceConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unit_content = self.generate_unit_file(service_name, service_config)?;

        let unit_path = if self.user_level {
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".config"));
            config_dir
                .join("systemd")
                .join("user")
                .join(format!("htbox-{}.service", service_name))
        } else {
            PathBuf::from(format!(
                "/etc/systemd/system/htbox-{}.service",
                service_name
            ))
        };

        if let Some(parent) = unit_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&unit_path, unit_content)?;

        self.systemctl_cmd().arg("daemon-reload").output()?;

        Ok(())
    }

    pub fn install_unit_for_service(
        &self,
        service_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unit_path = if self.user_level {
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".config"));
            config_dir
                .join("systemd")
                .join("user")
                .join(format!("htbox-{}.service", service_name))
        } else {
            PathBuf::from(format!(
                "/etc/systemd/system/htbox-{}.service",
                service_name
            ))
        };

        if !unit_path.exists() {
            let service_config = crate::config::ServiceConfig::load(service_name)?;
            self.install_unit(service_name, &service_config)?;
        }

        Ok(())
    }
}

impl ServiceBackend for SystemdBackend {
    fn start(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self
            .systemctl_cmd()
            .arg("start")
            .arg(self.service_name(service_name))
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to start service: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn stop(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self
            .systemctl_cmd()
            .arg("stop")
            .arg(self.service_name(service_name))
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to stop service: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn restart(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self
            .systemctl_cmd()
            .arg("restart")
            .arg(self.service_name(service_name))
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to restart service: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn enable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.install_unit_for_service(service_name)?;

        let output = self
            .systemctl_cmd()
            .arg("enable")
            .arg(self.service_name(service_name))
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to enable service: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn disable(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = self
            .systemctl_cmd()
            .arg("disable")
            .arg(self.service_name(service_name))
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to disable service: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn status(&self, service_name: &str) -> Result<ServiceStatus, Box<dyn std::error::Error>> {
        let output = self
            .systemctl_cmd()
            .arg("show")
            .arg(self.service_name(service_name))
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut running = false;
        let mut pid = None;

        for line in output_str.lines() {
            if line.starts_with("ActiveState=") {
                running = line.contains("active");
            }
            if line.starts_with("MainPID=") {
                let pid_str = line.trim_start_matches("MainPID=");
                if pid_str != "0" {
                    pid = pid_str.parse().ok();
                }
            }
        }

        let enabled = self.is_enabled(service_name).unwrap_or(false);

        Ok(ServiceStatus {
            running,
            pid,
            enabled,
        })
    }

    fn is_enabled(&self, service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let output = self
            .systemctl_cmd()
            .arg("is-enabled")
            .arg(self.service_name(service_name))
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).contains("enabled"))
    }

    fn install_unit(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let service_config = crate::config::ServiceConfig::load(service_name)?;
        self.install_unit(service_name, &service_config)
    }
}
