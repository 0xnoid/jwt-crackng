use clap::Parser;
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, anyhow};

#[derive(Parser, Debug)]
#[clap(name = "jwt-crackng", about = "Advanced JWT cracking tool")]
pub struct Args {
    #[clap(short, long)]
    pub token: String,

    #[clap(short, long)]
    pub output: Option<PathBuf>,

    #[clap(long = "min-length", short = 'n', default_value = "1")]
    pub min_length: usize,

    #[clap(long = "max-length", short = 'm', default_value = "12")]
    pub max_length: usize,

    #[clap(short = 'u', long = "use-alphabet", 
           default_value = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")]
    pub alphabet: String,

    #[clap(short = 'l', long = "logfile")]
    pub log_file: Option<PathBuf>,

    #[clap(short = 'g', long = "all-guesses")]
    pub all_tried_file: Option<PathBuf>,

    #[clap(short = 'a', long = "algorithm", default_value = "HS256",
           value_parser = ["HS256", "HS384", "HS512", 
                         "HMACSHA256", "HMACSHA384", "HMACSHA512"])]
    pub algorithm: String,

    #[clap(short = 'b', long = "base64")]
    pub base64: bool,

    #[clap(short = 'v', long)]
    pub verbose: bool,

    #[clap(long = "gpu")]
    pub gpu: bool,

    #[clap(long = "gpu-limit")]
    pub gpu_limit: Option<usize>,

    #[clap(long = "cpu")]
    pub cpu: Option<usize>,

    #[clap(long = "ram")]
    pub ram: Option<usize>,

    #[clap(long = "cores")]
    pub cores: Option<usize>,

    #[clap(long = "limit")]
    pub limit: Option<usize>,

    #[clap(long = "dictionary", num_args = 1..)]
    pub dictionary: Option<Vec<PathBuf>>,
}

pub fn parse_args() -> Result<Args> {
    let args = Args::parse();
    validate_args(&args)?;
    Ok(args)
}

fn validate_args(args: &Args) -> Result<()> {
    for (limit, name) in [
        (args.gpu_limit, "GPU"),
        (args.cpu, "CPU"),
        (args.ram, "RAM"),
        (args.limit, "Global"),
    ] {
        if let Some(value) = limit {
            if value == 0 || value > 100 {
                return Err(anyhow!("{} limit must be between 1 and 100", name));
            }
        }
    }

    if args.limit.is_some() && (args.cpu.is_some() || args.ram.is_some()) {
        return Err(anyhow!("--limit cannot be used with --cpu or --ram"));
    }

    if args.gpu && args.cores.is_some() {
        return Err(anyhow!("--cores cannot be used with --gpu"));
    }

    if let Some(cores) = args.cores {
        let available_cores = num_cpus::get();
        if cores == 0 || cores > available_cores {
            return Err(anyhow!("Core count must be between 1 and {}", available_cores));
        }
    }

    if args.gpu_limit.is_some() && !args.gpu {
        return Err(anyhow!("--gpu-limit requires --gpu to be enabled"));
    }

    Ok(())
}

pub fn save_result(output: &PathBuf, secret: &str) -> std::io::Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, secret)
}