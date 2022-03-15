use base58::ToBase58;
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use neo4rs::{query, Graph, Node};

#[derive(Debug, Serialize, Clone)]
pub struct Transaction {
   #[serde(skip_serializing)]
   pub hash: String,

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
   pub fn new() -> Self {
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

   pub fn from_node(node: &Node, outputs: Vec<Node>, inputs: Vec<Node>) -> Self {
      Transaction {
         hash: node.get("hash").unwrap_or(String::new()),
         uuid: node.get("uuid").unwrap_or(String::new()),
         date: node.get("date").unwrap_or(0),
         vin_hash: node.get("vin_hash").unwrap_or(String::new()),
         vout_hash: node.get("vout_hash").unwrap_or(String::new()),
         vout: outputs.iter().map(Output::from_node).collect(),
         _vin: inputs.iter().map(Input::from_node).collect(),
      }
   }

   pub fn validate(&mut self) -> bool {
      let prev_hash = self.hash.clone();
      self.generate_hash();
      prev_hash == self.hash
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

      let json = serde_json::to_string(&self).unwrap();
      let mut self_hasher = Sha256::new();
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

         let value = if vout.value % 1.0 == 0.0 {
            format!("{:.1}", vout.value)
         } else {
            format!("{}", vout.value)
         };

         queries.push(format!(
            "CREATE (tx)-[:OUT]->(:Output {{id: {}, value: {}, address: '{}'}})<-[:OWN]-({})",
            vout.id, value, vout.address, user
         ));
      }

      graph.run(query(queries.join("\n").as_str())).await.unwrap();
   }
}

// INPUTS AND OUTPUTS

//------
// Outputs
//------
#[derive(Debug, Serialize, Clone)]
pub struct Output {
   id: u32,
   value: f64,
   address: String,
}

impl Output {
   pub fn from_node(node: &Node) -> Self {
      Output {
         id: node.get::<i64>("id").unwrap() as u32,
         value: node.get::<f64>("value").unwrap(),
         address: node.get("address").unwrap(),
      }
   }
}

//------
// Inputs
//------
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

   pub fn from_node(node: &Node) -> Self {
      Input {
         prev_tx: node.get("prev_tx").unwrap_or(String::new()),
         id: node.get::<i64>("id").unwrap_or(0) as u32,
         value: node.get("value").unwrap_or(0.0),
         address: node.get("address").unwrap_or(String::new()),
         public_key: node.get("public_key").unwrap_or(String::new()),
         signature: node.get("signature").unwrap_or(String::new()),
      }
   }

   pub fn verify(&self) -> bool {
      false
   }
}
