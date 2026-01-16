use hex_literal::*;
use ore_rs::{scheme::bit2::OREAES128, ORECipher, OREEncrypt};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
// or #[tokio::main]
// or #[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a connection pool
    //  for MySQL, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    /*let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://dan@localhost/ore_perf").await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test")
        //.bind(150_i64)
        .fetch_one(&pool).await?;

    println!("ROW: {:?}", row);
    //assert_eq!(row.0, 150);*/

    let k1 = hex!("00010203 04050607 08090a0b 0c0d0e0f");
    let k2 = hex!("d0d007a5 3f9a6848 83bc1f21 0f6595a3");
    let ore: OREAES128 = ORECipher::init(&k1, &k2).unwrap();

    for i in 0..100_000u64 {
        //let x = i.encrypt(&ore).unwrap();
        let y = (i + 100_1000).encrypt(&ore).unwrap();
        let z = (i + 200_1000).encrypt(&ore).unwrap();
        println!(
            "INSERT INTO plaintext_test (pseudo_value) VALUES ('{}{}');",
            /*hex::encode(x.to_bytes()),*/
            hex::encode(y.to_bytes()),
            hex::encode(z.to_bytes())
        );
    }

    Ok(())
}
