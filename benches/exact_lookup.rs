
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ore_rs::{ORECipher, scheme::bit2::OREAES128, OREEncrypt};
use sqlx::{postgres::PgPoolOptions, Postgres, Pool};
use tokio::runtime::Runtime;
use hex_literal::hex;

#[inline]
async fn query(pool: &Pool<Postgres>, q: &str) {
    sqlx::query(q)
    .fetch_one(pool)
    .await
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt  = Runtime::new().unwrap();

    let k1 = hex!("00010203 04050607 08090a0b 0c0d0e0f");
    let k2 = hex!("d0d007a5 3f9a6848 83bc1f21 0f6595a3");

    let queries: Vec<u64> = vec![0, 1000, 5_000_000, 19_000_000];
    let ore: OREAES128 = ORECipher::init(&k1, &k2).unwrap();
    let ore_queries: Vec<String> = queries.iter().map(|&val| {
        hex::encode(val.encrypt(&ore).unwrap().to_bytes())
    }).collect();

    let pool = rt.block_on(async {
        PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://dan@localhost/ore_perf")
            .await
            .expect("Could not connect to the DB")
    });

    let mut group = c.benchmark_group("Exact");

    for q in queries.iter() {
        group.bench_function(format!("plaintext_linear/{}", q), |b| {
            b.to_async(&rt).iter(|| async {
                query(&pool, &format!("SELECT * FROM test_linear WHERE value = {} LIMIT 1", q)).await
            })
        });

        group.bench_function(format!("plaintext_btree/{}", q), |b| {
            b.to_async(&rt).iter(|| async {
                query(&pool, "SELECT * FROM test_btree WHERE value = 500000 LIMIT 1").await
            })
        });
    }

    for (q, ore_q) in queries.iter().zip(ore_queries) {
        group.bench_function(format!("ore_linear/{}", q), |b| {
            b.to_async(&rt).iter(|| async {
                query(&pool, &format!("SELECT * FROM test_ore_64_linear WHERE value = '(\\\\x{})' LIMIT 1", ore_q)).await
            })
        });

        group.bench_function(format!("ore_btree/{}", q), |b| {
            b.to_async(&rt).iter(|| async {
                query(&pool, "SELECT * FROM test_ore_64_btree WHERE value = '(\\\\x7272727272198fe17bb2dad40deb782edcdadcc2172941d70a2db2e2807efabd59d54ba32ed389ffccd18bd60f569bbb81b0734fb241f75d4f30387f511f2f2f790e9252c6de6c8395177a72ea1599bfb6240c753820ad42e11a24f0b01270aecab31db48f9c21c483941be86b1cab25cafba20c0c825b613e726b43e97c28853c4da0782af4855855e03b5edda54fdb4c91be5ad110193948615cda02340adac8b809a0e81df95399d9cc223919261821bd3e94429788d0064a05c045100cf660d969bd18c33784dedef8280a6013b5a9149e1855105237e5d77a072e609595d62a00bf6fe096716f773278608847fe40cd1cb9858fd59fa737dd463b9c8c67ff34319bc0175023096f38917f9cf77f8d6eb1431caf6076df3646b1d7baf95025f1c81763dbf54a8ee2b6f9328daab4d4c93edbd4111fcaf96c75c3a12bd8b6d2f14e2f808b5129dbaec03ae3f070fea73e6c440d188f6b04c08d3659a13ea96135b2626d4e887b1d6904c1846186bdd17a5831a30a393c09a63a28e803b3f3628e43e4e236dd57bc836a558e129dda2e8299189c0dad46)' LIMIT 1").await
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);