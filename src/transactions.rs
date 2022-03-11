use chrono::Utc;
use neo4rs::Graph;
use uuid::Uuid;

#[derive(Debug)]
pub struct Transaction {
   hash: String,
   uuid: String,
   date: i64,
   vin_hash: String,
   vout_hash: String,
}

impl Transaction {
   pub fn new() -> Transaction {
      Transaction {
         hash: String::new(),
         uuid: Uuid::new_v4().to_string(),
         date: Utc::now().timestamp(),
         vin_hash: String::new(),
         vout_hash: String::new(),
      }
   }
}

pub fn create_coinbase(graph: Graph) {}
