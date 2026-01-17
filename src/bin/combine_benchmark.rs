use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct HyperfineResult {
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkResult {
    mean: f64,
    times: Vec<f64>,
    memory_usage_byte: Vec<u64>,
    parameters: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct OutputFile {
    inserted: u64,
}

#[derive(Debug, Serialize)]
struct CombinedResult {
    num_records: u64,
    total_time_seconds: f64,
    total_records: u64,
    throughput_records_per_second: f64,
    average_memory_usage_bytes: u64,
    average_memory_usage_mb: f64,
    num_runs: usize,
}

#[derive(Debug, Serialize)]
struct Metadata {
    timestamp: String,
    rust_version: String,
    postgres_version: Option<String>,
    host_info: HostInfo,
}

#[derive(Debug, Serialize)]
struct HostInfo {
    os: String,
    cpu_model: Option<String>,
    cpu_cores: Option<u32>,
    total_memory_gb: Option<f64>,
}

#[derive(Debug, Serialize)]
struct CombinedOutput {
    metadata: Metadata,
    results: Vec<CombinedResult>,
}

fn get_command_output(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
}

fn get_rust_version() -> String {
    get_command_output("rustc", &["--version"])
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_postgres_version() -> Option<String> {
    // Try to get version from running Docker container
    get_command_output(
        "docker",
        &["exec", "ore-benches-postgres", "psql", "--version"],
    )
}

fn get_host_info() -> HostInfo {
    let os = if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        "Unknown".to_string()
    };

    let cpu_model = if cfg!(target_os = "macos") {
        get_command_output("sysctl", &["-n", "machdep.cpu.brand_string"])
    } else if cfg!(target_os = "linux") {
        get_command_output("grep", &["-m", "1", "model name", "/proc/cpuinfo"])
            .map(|s| s.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or(s))
    } else {
        None
    };

    let cpu_cores = if cfg!(target_os = "macos") {
        get_command_output("sysctl", &["-n", "hw.ncpu"])
            .and_then(|s| s.parse().ok())
    } else if cfg!(target_os = "linux") {
        get_command_output("nproc", &[])
            .and_then(|s| s.parse().ok())
    } else {
        None
    };

    let total_memory_gb = if cfg!(target_os = "macos") {
        get_command_output("sysctl", &["-n", "hw.memsize"])
            .and_then(|s| s.parse::<u64>().ok())
            .map(|bytes| bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if cfg!(target_os = "linux") {
        get_command_output("grep", &["MemTotal", "/proc/meminfo"])
            .and_then(|s| {
                s.split_whitespace()
                    .nth(1)
                    .and_then(|kb| kb.parse::<u64>().ok())
                    .map(|kb| kb as f64 / (1024.0 * 1024.0))
            })
    } else {
        None
    };

    HostInfo {
        os,
        cpu_model,
        cpu_cores,
        total_memory_gb,
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        anyhow::bail!("Usage: {} <benchmark_name>", args.get(0).map(|s| s.as_str()).unwrap_or("combine_benchmark"));
    }
    
    let benchmark_name = &args[1];
    let hyperfine_file = format!("target/{}_hyperfine.json", benchmark_name);
    
    // Read hyperfine results
    let hyperfine_data = fs::read_to_string(&hyperfine_file)
        .context(format!("Failed to read hyperfine results from {}", hyperfine_file))?;
    let hyperfine: HyperfineResult = serde_json::from_str(&hyperfine_data)
        .context("Failed to parse hyperfine JSON")?;

    let mut combined_results = Vec::new();

    for result in hyperfine.results {
        let num_records: u64 = result
            .parameters
            .get("num_records")
            .context("Missing num_records parameter")?
            .parse()
            .context("Failed to parse num_records as u64")?;

        let num_runs = result.times.len();
        
        // Validate output files exist and contain correct data
        for run_idx in 0..num_runs {
            let output_file = format!("target/{}-{}_{}.json", benchmark_name, num_records, run_idx);
            
            if !Path::new(&output_file).exists() {
                bail!("Expected output file {} does not exist", output_file);
            }
            
            let output_data = fs::read_to_string(&output_file)
                .context(format!("Failed to read output file {}", output_file))?;
            let output: OutputFile = serde_json::from_str(&output_data)
                .context(format!("Failed to parse output file {}", output_file))?;
            
            if output.inserted != num_records {
                bail!(
                    "Mismatch in {}: expected {} records but file contains {}",
                    output_file,
                    num_records,
                    output.inserted
                );
            }
        }

        // Calculate average memory usage
        let avg_memory = if !result.memory_usage_byte.is_empty() {
            result.memory_usage_byte.iter().sum::<u64>() / result.memory_usage_byte.len() as u64
        } else {
            0
        };

        let total_time = result.mean;
        let throughput = num_records as f64 / total_time;

        combined_results.push(CombinedResult {
            num_records,
            total_time_seconds: total_time,
            total_records: num_records,
            throughput_records_per_second: throughput,
            average_memory_usage_bytes: avg_memory,
            average_memory_usage_mb: avg_memory as f64 / (1024.0 * 1024.0),
            num_runs,
        });
    }

    // Sort by num_records for consistent output
    combined_results.sort_by_key(|r| r.num_records);

    let metadata = Metadata {
        timestamp: chrono::Utc::now().to_rfc3339(),
        rust_version: get_rust_version(),
        postgres_version: get_postgres_version(),
        host_info: get_host_info(),
    };

    let output = CombinedOutput {
        metadata,
        results: combined_results,
    };

    let json = serde_json::to_string_pretty(&output)?;
    
    let output_file = format!("results/ingest/{}_combined.json", benchmark_name);
    fs::write(&output_file, json)
        .context(format!("Failed to write output to {}", output_file))?;
    
    println!("Results written to {}", output_file);

    Ok(())
}
