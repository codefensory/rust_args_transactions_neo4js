mod key_pair;
// mod signatures;
mod transactions;

use neo4rs::Graph;
use std::env;

use crate::transactions::Transaction;

#[tokio::main]
async fn main() {
   let args: Vec<String> = env::args().collect();

   if args.len() < 2 {
      println!("Sin resultados");
      return;
   }

   let action = &args[1];

   match action.as_str() {
      "create_account" => key_pair::generate_keys(),
      "send" => {
         if args.len() < 5 {
            println!("USE: send <from_private_key> <to_public_key_hashed> <amount>");
            return;
         }

         let send_args: Vec<&String> = args[2..].into_iter().collect();
         let from_private_key = &send_args[0];
         let to_public_key_hashed = &send_args[1];
         let amount = &send_args[2];

         println!("From: {}", from_private_key);
         println!("To: {}", to_public_key_hashed);
         println!("Amount: {}", amount);
      }
      "coinbase" => {
         if args.len() < 4 {
            println!("USE: coinbase <to_public_key_hashed> <amount>");
            return;
         }

         let transaction = Transaction::new();
         println!("{:?}", transaction);

         let send_args: Vec<&String> = args[2..].into_iter().collect();
         let to_public_key_hashed = &send_args[0];
         let amount = &send_args[1];

         println!(
            "Coinbase created for {} with {} coins",
            to_public_key_hashed, amount
         );
      }
      "balance" => {
         if args.len() < 3 {
            println!("USE: balance <public_key_hashed>");
            return;
         }

         let to_public_key_hashed = &args[2];
         println!("balance of {} is: {}", to_public_key_hashed, "None");
      }
      _ => println!("Sin resultado"),
   };
}

async fn _connect_db() -> Graph {
   let uri = "127.0.0.1:7687";
   let user = "neo4j";
   let pass = "1234";
   Graph::new(&uri, user, pass).await.unwrap()
}
