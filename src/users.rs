use crate::transactions::{Compare, Input, Transaction};
use neo4rs::{query, Graph, Node};

pub struct User {
   address: String,
}

impl User {
   pub fn new(address: String) -> Self {
      User { address }
   }

   pub async fn get_unspend_outputs_as_inputs(
      &self,
      graph: &Graph,
      amount: f32,
   ) -> (Vec<Input>, f32) {
      let mut response = graph
         .execute(
            query(
               r#"
         MATCH (:User {address: $address})-[:OWN]->(o:Output)<-[:OUT]-(tx:Transaction)
         WHERE isEmpty((o)-[:IN]->(:Transaction))
         WITH o.id as oid, tx
         WITH oid, tx, [(tx)-[:OUT]->(o:Output) | o] as t_outputs
         WITH oid, tx, t_outputs, [(tx)-[:IN]->(i:Output) | i] as t_inputs
         RETURN oid, tx, t_outputs, t_inputs
         "#,
            )
            .param("address", &*self.address),
         )
         .await
         .unwrap();

      let mut outputs: Vec<Input> = Vec::new();
      let mut balance = 0.0;

      while let Ok(Some(row)) = response.next().await {
         let oid = row.get::<i64>("oid").unwrap() as u32;

         let tx_node: Node = row.get("tx").unwrap();
         let outputs_nodes: Vec<Node> = row.get("t_outputs").unwrap();
         let inputs_nodes: Vec<Node> = row.get("t_inputs").unwrap();

         let mut transaction = Transaction::from_node(&tx_node, outputs_nodes, inputs_nodes);
         let is_valid = transaction.validate();

         if is_valid {
            let output = transaction.vout[oid as usize].clone();
            let input = Input::new(transaction.hash.clone(), output);

            outputs.push(input);

            balance += transaction.vout[oid as usize].value;

            if balance >= amount {
               break;
            }
         }
      }

      (outputs, balance)
   }

   // In a future this will use multi address
   pub async fn send(&self, to_address: String, amount: f32, mut inputs: Vec<Input>, graph: &Graph) {
      if !inputs.verify() {
         println!("Inputs no validos");
         return;
      }

      let balance = inputs.iter().fold(0.0f32, |acc, x| acc + x.value);

      // Verify Balance
      if balance < amount {
         println!("Insufficient balance");
         return;
      }

      // Order by ID
      inputs.sort_by(|a, b| b.id.cmp(&a.id));

      // Create Transaction
      let mut transaction = Transaction::new();
      transaction.set_inputs(inputs);
      transaction.add_output(amount, to_address);

      let remaining = balance - amount;

      if remaining != 0.0f32 {
         transaction.add_output(remaining, self.address.clone());
      }

      transaction.upload(graph).await;
   }
}
