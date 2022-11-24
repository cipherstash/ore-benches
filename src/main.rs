use sqlx::postgres::PgPoolOptions;

#[tokio::main]
// or #[tokio::main]
// or #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    //  for MySQL, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://dan@localhost/ore_perf").await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test")
        //.bind(150_i64)
        .fetch_one(&pool).await?;

    println!("ROW: {:?}", row);
    //assert_eq!(row.0, 150);

    Ok(())
}
