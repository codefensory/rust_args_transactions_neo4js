use k256::{elliptic_curve::sec1::ToEncodedPoint, SecretKey};
use rand_core::OsRng;
use ripemd::{Ripemd160, Digest};

pub fn generate_keys() {
    let private_key = SecretKey::random(&mut OsRng);
    let public_key = private_key.public_key().to_encoded_point(false);

    println!("Private Key: {}", hex::encode(private_key.to_be_bytes()));
    println!("Public Key: {}", hex::encode(&public_key));

    let mut hasher = Ripemd160::new();
    hasher.update(&public_key);
    let id = hasher.finalize();

    println!("ID: {}", hex::encode(&id));
}
