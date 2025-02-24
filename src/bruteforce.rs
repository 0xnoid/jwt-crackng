use crate::cli::Args;
use crate::jwt::{self, JwtParts};
use crate::validator::JwtValidator;
use crate::hw::{HardwareConfig, HardwareManager};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{self, BufRead, Write, BufWriter};
use std::path::PathBuf;
use std::time::{Duration};
use std::fs::{self, OpenOptions};

pub struct Logger {
    file: Option<Arc<Mutex<File>>>,
    tried_file: Option<Arc<Mutex<BufWriter<File>>>>,
}

impl Logger {
    pub fn new(log_path: Option<&PathBuf>, tried_path: Option<&PathBuf>) -> Result<Self> {
        let file = if let Some(path) = log_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            Some(Arc::new(Mutex::new(OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)?)))
        } else {
            None
        };

        let tried_file = if let Some(path) = tried_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            Some(Arc::new(Mutex::new(BufWriter::new(OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)?))))
        } else {
            None
        };

        Ok(Logger { file, tried_file })
    }

    pub fn log(&self, msg: &str) -> Result<()> {
        if let Some(ref file) = self.file {
            let mut file = file.lock().map_err(|_| anyhow::anyhow!("Failed to acquire log file lock"))?;
            writeln!(*file, "{}", msg)?;
        }
        Ok(())
    }

    pub fn log_tried(&self, attempt: &str) -> Result<()> {
        if let Some(ref file) = self.tried_file {
            let mut writer = file.lock().map_err(|_| anyhow::anyhow!("Failed to acquire tried file lock"))?;
            writeln!(*writer, "{}", attempt)?;
            writer.flush()?;
        }
        Ok(())
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Logger {
            file: self.file.clone(),
            tried_file: self.tried_file.clone(),
        }
    }
}

fn generate_combination(index: usize, alphabet: &str, length: usize) -> String {
    let mut result = String::with_capacity(length);
    let mut remaining = index;
    let base = alphabet.len();
    let chars: Vec<char> = alphabet.chars().collect();

    for _ in 0..length {
        let char_index = remaining % base;
        result.push(chars[char_index]);
        remaining /= base;
    }

    result
}

pub fn crack(args: &Args) -> Result<Option<String>> {
    let hw_config = HardwareConfig::new()
        .with_gpu(args.gpu)
        .with_gpu_limit(args.gpu_limit.unwrap_or(100) as u8)
        .with_cpu_limit(args.cpu.unwrap_or(100) as u8)
        .with_ram_limit(args.ram.unwrap_or(100) as u8)
        .with_core_limit(args.cores.unwrap_or_else(num_cpus::get))
        .with_global_limit(args.limit.unwrap_or(100) as u8);

    let hw_manager = HardwareManager::new(hw_config)?;
    
    // Configure Rayon thread pool with core limit
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(hw_manager.get_core_limit())
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to configure thread pool: {}", e))?;
    
    thread_pool.install(|| {
        hw_manager.init_resources()?;
        
        let jwt = jwt::parse_token(&args.token)?;

        if let Err(e) = JwtValidator::validate_token(&args.token) {
            return Err(anyhow::anyhow!("Invalid JWT token: {}", e));
        }

        let found = Arc::new(AtomicBool::new(false));
        let last_tried = Arc::new(std::sync::Mutex::new(String::new()));

        match Logger::new(args.log_file.as_ref(), args.all_tried_file.as_ref()) {
            Ok(logger) => {
                let logger = Arc::new(logger);
                logger.log(&format!("Starting crack with arguments: {:?}", args))?;
                logger.log(&format!("JWT Header and Payload: {}", jwt.content))?;

                if let Some(dict_files) = &args.dictionary {
                    return try_dictionary(
                        &jwt,
                        dict_files,
                        &found,
                        args.verbose,
                        &logger,
                        &args.algorithm,
                        args.base64,
                    );
                }

                let total_combinations: u64 = (args.min_length..=args.max_length)
                    .map(|n| (args.alphabet.len() as u64).pow(n as u32))
                    .sum();

                logger.log(&format!("Total combinations to try: {}", total_combinations))?;
                logger.log(&format!("Using alphabet: {}", args.alphabet))?;
                let pb = setup_progress_bar(total_combinations);

                for length in args.min_length..=args.max_length {
                    logger.log(&format!("Trying length: {}", length))?;

                    if let Some(result) = try_length(
                        &jwt,
                        &args.alphabet,
                        length,
                        &found,
                        &pb,
                        &last_tried,
                        args.verbose,
                        &logger,
                        &args.algorithm,
                        args.base64,
                    )? {
                        if let Some(output) = &args.output {
                            if let Some(parent) = output.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            crate::cli::save_result(output, &result)?;
                            logger.log(&format!("Saved result to file: {:?}", output))?;
                        }
                        logger.log(&format!("Found secret: {}", result))?;
                        pb.finish_with_message(format!("Found: {}", result));
                        return Ok(Some(result));
                    }

                    logger.log(&format!("No match found for length {}", length))?;
                }

                pb.finish_and_clear();
                logger.log("No secret found after exhausting all possibilities")?;
                Ok(None)
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize logging: {}. Continuing without logging...", e);
                Ok(None)
            }
        }
    })
}

fn try_length(
    jwt: &JwtParts,
    alphabet: &str,
    length: usize,
    found: &Arc<AtomicBool>,
    pb: &ProgressBar,
    _last_tried: &Arc<std::sync::Mutex<String>>,
    verbose: bool,
    logger: &Arc<Logger>,
    algorithm: &str,
    base64: bool,
) -> Result<Option<String>> {
    let result = (0..alphabet.len().pow(length as u32))
        .into_par_iter()
        .find_map_any(|index| {
            if found.load(Ordering::Relaxed) {
                return None;
            }

            let attempt = generate_combination(index, alphabet, length);

            if jwt::verify_signature(jwt, &attempt, algorithm, base64) {
                found.store(true, Ordering::Relaxed);
                logger.log(&format!("Correct Secret: {}", attempt)).unwrap();
                return Some(attempt);
            }

            if verbose {
                println!("Attempt: {}", attempt);
            }
            logger.log_tried(&attempt).unwrap();
            pb.inc(1);

            None
        });

    Ok(result)
}

fn try_dictionary(
    jwt: &JwtParts,
    dict_files: &[PathBuf],
    found: &Arc<AtomicBool>,
    verbose: bool,
    logger: &Arc<Logger>,
    algorithm: &str,
    base64: bool,
) -> Result<Option<String>> {
    let mut total_attempts = 0;

    for file_path in dict_files {
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
        total_attempts += lines.len();

        if verbose {
            println!("Trying dictionary: {}", file_path.display());
        }
        logger.log(&format!("Trying dictionary: {}", file_path.display()))?;

        let logger = Arc::clone(logger);
        if let Some(secret) = lines
            .par_iter()
            .find_map_any(|line| {
                if found.load(Ordering::Relaxed) {
                    return None;
                }
                if verbose {
                    println!("Trying: {}", line);
                }

                if let Err(e) = logger.log_tried(line) {
                    eprintln!("Error logging attempt: {}", e);
                }

                if jwt::verify_signature(jwt, line, algorithm, base64) {
                    found.store(true, Ordering::Relaxed);
                    Some(line.clone())
                } else {
                    None
                }
            })
        {
            logger.log(&format!("Found secret in dictionary: {}", secret))?;
            return Ok(Some(secret));
        }
    }

    if verbose {
        println!("Tried {} passwords", total_attempts);
    }
    logger.log(&format!("Tried {} passwords from dictionaries", total_attempts))?;

    Ok(None)
}

fn setup_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% ({pos}/{len}) Last: {msg}")
            .unwrap()
            .progress_chars("=>-")
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}