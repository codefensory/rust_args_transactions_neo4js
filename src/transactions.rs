use base58::ToBase58;
use chrono::Utc;
use neo4rs::query;
use neo4rs::Graph;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Output {
   id: u32,
   value: f64,
   address: String,
}

#[derive(Debug, Serialize)]
pub struct Input {
   prev_tx: String,
   id: u32,
   value: f64,

   address: String,
   public_key: String,
   signature: String,
}

#[derive(Debug, Serialize)]
pub struct Transaction {
   hash: String,
   uuid: String,
   date: i64,
   vin_hash: String,
   vout_hash: String,

   #[serde(skip_serializing)]
   vout: Vec<Output>,
   #[serde(skip_serializing)]
   _vin: Vec<Input>,
}

impl Transaction {
   pub fn new() -> Transaction {
      Transaction {
         hash: String::new(),
         uuid: Uuid::new_v4().as_bytes().to_base58(),
         date: Utc::now().timestamp(),
         vin_hash: String::new(),
         vout_hash: String::new(),
         vout: Vec::new(),
         _vin: Vec::new(),
      }
   }

   pub fn generate_hash(&mut self) {
      let json = serde_json::to_string(&self.vout).unwrap();

      let mut vout_hasher = Sha256::new();
      vout_hasher.update(&json);
      self.vout_hash = hex::encode(vout_hasher.finalize());

      let json = serde_json::to_string(&self._vin).unwrap();
      let mut vin_hasher = Sha256::new();
      vin_hasher.update(&json);
      self.vin_hash = hex::encode(vin_hasher.finalize());

      let mut self_hasher = Sha256::new();
      let json = serde_json::to_string(&self).unwrap();
      self_hasher.update(&json);
      self.hash = hex::encode(self_hasher.finalize());
   }

   pub fn add_output(&mut self, amount: f64, to_address: String) {
      let output = Output {
         id: self.vout.len() as u32,
         value: amount,
         address: to_address,
      };

      self.vout.push(output);
   }

   pub fn upload(&self, _graph: Graph) {
      let mut queries = Vec::new();
      // Create transaction
      queries.push(
         query("CREATE (tx:Transaction {hash: $hash, uuid: $uuid, date: $date, vin_hash: $vin_hash, vout_hash: $vout_hash})")
         .param("hash", &*self.hash)
         .param("uuid", &*self.uuid)
         .param("date", self.date)
         .param("vin_hash", &*self.vin_hash)
         .param("vout_hash", &*self.vout_hash)
      );

      // Error here
      for (index, vout) in self.vout.into_iter().enumerate() {
         queries.push(
            query("MERGE (u:User {address: $address})").param("address", vout.address.clone()),
         );

         queries.push(
            query(
               "CREATE (tx)-[:OUT]->(o:Output {id: $id, value: $value, address: $address})<-[:OWN]",
            )
            .param("id", vout.id.to_string())
            .param("value", vout.value.to_string())
            .param("address", &*vout.address),
         );
      }
   }
}

pub fn create_coinbase(to_address: String, amount: String, graph: Graph) {
   let mut transaction = Transaction::new();

   transaction.add_output(amount.parse::<f64>().unwrap(), to_address);

   transaction.generate_hash();
   transaction.upload(graph);
   println!("{:?}", transaction)
}
