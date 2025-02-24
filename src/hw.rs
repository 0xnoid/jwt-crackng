use crate::errors::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct HardwareConfig {
    pub gpu_enabled: bool,
    pub gpu_limit: u8,
    pub cpu_limit: u8,
    pub ram_limit: u8,
    pub core_limit: usize,
    pub global_limit: u8,
}

impl HardwareConfig {
    pub fn new() -> Self {
        Self {
            gpu_enabled: false,
            gpu_limit: 100,
            cpu_limit: 100,
            ram_limit: 100,
            core_limit: num_cpus::get(),
            global_limit: 100,
        }
    }

    pub fn with_gpu(mut self, enabled: bool) -> Self {
        self.gpu_enabled = enabled;
        self
    }

    pub fn with_gpu_limit(mut self, limit: u8) -> Self {
        self.gpu_limit = limit;
        self
    }

    pub fn with_cpu_limit(mut self, limit: u8) -> Self {
        self.cpu_limit = limit;
        self
    }

    pub fn with_ram_limit(mut self, limit: u8) -> Self {
        self.ram_limit = limit;
        self
    }

    pub fn with_core_limit(mut self, limit: usize) -> Self {
        self.core_limit = limit;
        self
    }

    pub fn with_global_limit(mut self, limit: u8) -> Self {
        self.global_limit = limit;
        self
    }
}

pub struct HardwareManager {
    config: HardwareConfig,
    stop_signal: Arc<AtomicBool>,
}

impl HardwareManager {
    pub fn new(config: HardwareConfig) -> Result<Self> {
        Ok(Self {
            config,
            stop_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn init_resources(&self) -> Result<()> {
        if self.config.cpu_limit < 100 {
            self.limit_cpu_usage(self.config.cpu_limit)?;
        }
        Ok(())
    }

    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::SeqCst);
    }

    fn limit_cpu_usage(&self, percentage: u8) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::process::{self, Command};
            use std::fs;
            use std::env;
            
            // systemd check
            if env::var("SYSTEMD_SCOPE_SET").is_ok() {
                return Ok(());
            }

            let pid = process::id();
            let exe_path = std::env::current_exe()?;
            let args: Vec<String> = std::env::args().collect();
            
            // Quota
            let total_cores = num_cpus::get();
            let used_cores = self.config.core_limit;
            
            // CPU task
            let cpu_mask = format!("0-{}", used_cores - 1);
            
            // Calculate the quota as a percentage of total available CPU
            let total_quota = percentage;
            
            let mut filtered_args = Vec::new();
            let mut skip_next = false;
            for (_i, arg) in args.iter().enumerate().skip(1) {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                
                if arg.starts_with("--cpu") || arg.starts_with("--ram") || arg.starts_with("--cores") {
                    skip_next = true;
                    continue;
                }
                
                filtered_args.push(arg.as_str());
            }

            let mut command = Command::new("taskset");
            command
                .args([
                    "-c",
                    &cpu_mask,
                    "systemd-run",
                    "--user",
                    "--scope",
                    "--unit", &format!("jwt-crackng_{}.scope", pid),
                    "--property", &format!("CPUQuota={}%", total_quota),
                    "--property", "CPUWeight=100",
                    "--property", "Delegate=yes",
                    "--same-dir",
                    "--",
                ])
                .arg(&exe_path)
                .args(&filtered_args)
                .env("SYSTEMD_SCOPE_SET", "1")
                .env("RAYON_NUM_THREADS", used_cores.to_string());

            println!("Using {} out of {} cores at {}% utilization", 
                    used_cores, total_cores, percentage);
            println!("CPU mask: {}", cpu_mask);
            println!("Total CPU quota: {}%", total_quota);

            let status = command.status()?;

            if !status.success() {
                eprintln!("Warning: Could not set CPU limits via systemd-run. Continuing without CPU limiting...");
                return Ok(());
            }

            let user_cgroup_path = format!("/sys/fs/cgroup/user.slice/user-{}.slice/user@{}.service/app.slice/jwt-crackng_{}.scope",
                users::get_current_uid(),
                users::get_current_uid(),
                pid);

            if let Ok(quota) = fs::read_to_string(format!("{}/cpu.max", user_cgroup_path)) {
                println!("CPU limit applied via systemd-run: {}", quota.trim());
                
                let parts: Vec<&str> = quota.trim().split_whitespace().collect();
                if parts.len() == 2 {
                    if let (Ok(max), Ok(per)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                        let effective_percentage = (max as f32 / per as f32) * 100.0;
                        println!("Effective CPU limit: {:.1}%", effective_percentage);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            std::process::exit(0);
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("Warning: CPU limiting not implemented on this platform");
        }
        
        #[cfg(not(target_os = "linux"))]
        Ok(())
    }

    pub fn get_core_limit(&self) -> usize {
        self.config.core_limit
    }
}