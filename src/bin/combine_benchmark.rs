use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

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
struct CombinedOutput {
    results: Vec<CombinedResult>,
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

    let output = CombinedOutput {
        results: combined_results,
    };

    let json = serde_json::to_string_pretty(&output)?;
    
    let output_file = format!("results/ingest/{}_combined.json", benchmark_name);
    fs::write(&output_file, json)
        .context(format!("Failed to write output to {}", output_file))?;
    
    println!("Results written to {}", output_file);

    Ok(())
}
