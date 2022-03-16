use crate::transactions::{Input, Transaction};
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
      graph: Graph,
      amount: f64,
   ) -> (Vec<Input>, f64) {
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

      let mut outputs: Vec<(Input)> = Vec::new();
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
}
