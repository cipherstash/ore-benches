//! Generate random plaintext data for benchmarking
//!
//! This binary generates random integer values and inserts them into the
//! integer_plaintext table. This is used for testing and development purposes.
//!
//! Note: This binary is not currently used in the main benchmark workflow.
//! The ingest benchmarks (encrypt_int, encrypt_string, etc.) generate and
//! encrypt data directly rather than reading from plaintext tables.
//!
//! Environment variables:
//! - DATABASE_URL: PostgreSQL connection string
//! - NUM_RECORDS: Number of records to generate (default: 10000)

use anyhow::{Context, Result};
use rand::Rng;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!(
        "Generating and inserting {} random integers...",
        num_records
    );

    let mut rng = rand::thread_rng();
    let batch_size = 1000;

    for batch_start in (0..num_records).step_by(batch_size) {
        let batch_end = (batch_start + batch_size as i32).min(num_records);
        let batch_count = batch_end - batch_start;

        // Build bulk insert query
        let mut query = "INSERT INTO integer_plaintext (value) VALUES ".to_string();
        let mut values = Vec::new();

        for i in 0..batch_count {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str(&format!("(${})", i + 1));
            values.push(rng.gen::<i32>());
        }

        let mut query_builder = sqlx::query(&query);
        for value in values {
            query_builder = query_builder.bind(value);
        }

        query_builder.execute(&pool).await?;

        println!("Inserted {} / {} records", batch_end, num_records);
    }

    println!(
        "Successfully inserted {} records into integer_plaintext",
        num_records
    );

    Ok(())
}
