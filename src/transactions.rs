use base58::ToBase58;
use chrono::Utc;
use neo4rs::query;
use neo4rs::Graph;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct User {
   address: String,
}

impl User {
   pub fn new(address: String) -> Self {
      User { address }
   }

   pub async fn get_unspend_outups(&self, graph: Graph) -> Vec<Input> {
      let response = graph
         .execute(query(
            r#"
         MATCH (: User {address: $address})-[:OWN]->(o:Output)
         WHERE NOT ((o)-[:IN]->(:Transaction))
         RETURN o
      "#,
         ))
         .await
         .unwrap();

      vec![]
   }
}

#[derive(Debug, Serialize, Clone)]
pub struct Output {
   id: u32,
   value: f64,
   address: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Input {
   prev_tx: String,
   id: u32,
   value: f64,

   address: String,
   public_key: String,
   signature: String,
}

impl Input {
   pub fn new(prev_tx: String, output: Output) -> Self {
      Input {
         prev_tx,
         id: output.id,
         value: output.value,
         address: output.address,

         public_key: String::new(),
         signature: String::new(),
      }
   }
}

#[derive(Debug, Serialize, Clone)]
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

   pub async fn upload(&mut self, graph: Graph) {
      self.generate_hash();

      let mut queries = Vec::new();

      // Create transaction
      queries.push(format!(
         "CREATE (tx:Transaction {{ hash: '{}', uuid: '{}', date: {}, vin_hash: '{}', vout_hash: '{}' }})",
         &*self.hash, &*self.uuid, self.date, &*self.vin_hash, &*self.vout_hash,
      ));

      for (index, vout) in self.vout.clone().into_iter().enumerate() {
         let user = "uo".to_owned() + &index.to_string();

         queries.push(format!(
            "MERGE ({}:User {{address: '{}'}})",
            user,
            vout.address.clone()
         ));

         queries.push(format!(
            "CREATE (tx)-[:OUT]->(:Output {{id: {}, value: {}, address: '{}'}})<-[:OWN]-({})",
            vout.id, vout.value, vout.address, user
         ));
      }

      let q = queries.join("\n");
      println!("{}", q.as_str());

      graph.run(query(q.as_str())).await.unwrap();
   }
}

pub async fn create_coinbase(to_address: String, amount: String, graph: Graph) {
   let mut transaction = Transaction::new();

   transaction.add_output(amount.parse::<f64>().unwrap(), to_address);
   transaction.upload(graph).await;

   //println!("{:?}", transaction);
}
