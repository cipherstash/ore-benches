//! Encrypt data binary - encrypts large JSON objects from json_large_plaintext table using CipherStash
//!
//! This binary reads plaintext large JSON objects from the json_large_plaintext table and encrypts
//! them using the cipherstash-client library's `encrypt_eql` function, storing the
//! encrypted values in the json_large_encrypted table.
//!
//! Environment variables required:
//! - DATABASE_URL: PostgreSQL connection string
//! - CS_CLIENT_ID: CipherStash client ID
//! - CS_CLIENT_KEY: CipherStash client key  
//! - CS_WORKSPACE_CRN: CipherStash workspace CRN
//!
//! TODO: The CipherStash client API needs to be properly configured based on the
//! actual cipherstash-client crate API. This is a placeholder implementation that
//! outlines the structure. Refer to cipherstash-client documentation at:
//! https://docs.rs/cipherstash-client

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{
        column::{Index, IndexType},
        ColumnConfig, ColumnType,
    },
};
use dbbenches::{IngestOptionsBuilder, WrappedJson};
use fake::{
    faker::{address, chrono, company, internet, name, phone_number},
    Dummy, Fake, Rng,
};
use serde_json::json;
use std::env;

struct FakeJsonLarge;

// FIXME: cipherstash-client doesn't have a From<serde_json::Value> for Plaintext impl yet, so we use String here
impl Dummy<FakeJsonLarge> for WrappedJson {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &FakeJsonLarge, _: &mut R) -> Self {
        let department = [
            "Engineering",
            "Sales",
            "Marketing",
            "HR",
            "Finance",
            "Operations",
        ]
        .iter()
        .take((1..6).fake())
        .last()
        .unwrap()
        .to_string();
        let type_ = ["Home", "Work", "Billing", "Shipping"]
            .iter()
            .take((1..4).fake())
            .last()
            .unwrap()
            .to_string();
        let status = [
            "Pending",
            "Processing",
            "Shipped",
            "Delivered",
            "Cancelled",
            "Returned",
        ]
        .iter()
        .take((1..6).fake())
        .last()
        .unwrap()
        .to_string();
        let relationship = ["Spouse", "Parent", "Sibling", "Friend", "Other"]
            .iter()
            .take((1..5).fake())
            .last()
            .unwrap()
            .to_string();

        let value = json!({
            "user": {
                "first_name": name::en::FirstName().fake::<String>(),
                "last_name": name::en::LastName().fake::<String>(),
                "age": (18..=99).fake::<i32>(),
                "email": internet::en::FreeEmail().fake::<String>(),
                "username": internet::en::Username().fake::<String>(),
                "contact": {
                    "phone": phone_number::en::PhoneNumber().fake::<String>(),
                    "mobile": phone_number::en::CellNumber().fake::<String>(),
                    "emergency_contact": {
                        "name": name::en::Name().fake::<String>(),
                        "phone": phone_number::en::PhoneNumber().fake::<String>(),
                        "relationship": relationship
                    }
                }
            },
            "company": {
                "name": company::en::CompanyName().fake::<String>(),
                "industry": company::en::Industry().fake::<String>(),
                "position": company::en::Profession().fake::<String>(),
                "department": department,
                "salary": (40000..=300000).fake::<i32>(),
                "start_date": chrono::en::Date().fake::<String>()
            },
            "addresses": (0..(1..4).fake::<i32>()).map(|_| {
                json!({
                    "type": type_,
                    "street": address::en::StreetName().fake::<String>(),
                    "city": address::en::CityName().fake::<String>(),
                    "state": address::en::StateName().fake::<String>(),
                    "zip": address::en::ZipCode().fake::<String>(),
                    "country": "United States"
                })
            }).collect::<Vec<_>>(),
            "orders": (0..(5..=20).fake::<i32>()).map(|_| {
                json!({
                    "order_id": format!("ORD-{}", (100000..=999999).fake::<i32>()),
                    "date": chrono::en::Date().fake::<String>(),
                    "total": (10.0..=5000.0).fake::<f64>(),
                    "status": status,
                    "items": (0..(1..=8).fake::<i32>()).map(|_| {
                        json!({
                            "product": company::en::Buzzword().fake::<String>(),
                            "quantity": (1..=10).fake::<i32>(),
                            "price": (5.0..=500.0).fake::<f64>()
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });

        WrappedJson(value)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    IngestOptionsBuilder::new()
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new("json_large_encrypted", "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::JsonB)
                // FIXME: There is no convenience method for SteVec yet on Index
                .add_index(Index::new(IndexType::SteVec {
                    prefix: "value".to_string(),
                    term_filters: Default::default(),
                })),
        )
        .build()?
        .ingest::<WrappedJson, _>(FakeJsonLarge)
        .await?;

    Ok(())
}
