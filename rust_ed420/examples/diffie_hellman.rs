// examples/diffie_hellman.rs

use rust_ed420::montgomery::{montgomery_ladder, G_MONT};
use rust_ed420::curve::L; // The order of the group
use num_bigint::RandBigInt;

fn main() {
    println!("--- Diffie-Hellman Key Exchange with Montgomery Curve ---");

    let mut rng = rand::thread_rng();

    // 1. Alice generates her secret and public keys
    let alice_secret_key = rng.gen_biguint_below(&L);
    let alice_public_key = montgomery_ladder(&G_MONT, &alice_secret_key);
    println!("\nAlice's public key: {:?}", alice_public_key.0.to_biguint());

    // 2. Bob generates his secret and public keys
    let bob_secret_key = rng.gen_biguint_below(&L);
    let bob_public_key = montgomery_ladder(&G_MONT, &bob_secret_key);
    println!("Bob's public key:   {:?}", bob_public_key.0.to_biguint());

    // 3. They exchange public keys and compute the shared secret

    // Alice computes the shared secret with Bob's public key
    let shared_secret_alice = montgomery_ladder(&bob_public_key, &alice_secret_key);
    println!("\nAlice's computed shared secret: {:?}", shared_secret_alice.0.to_biguint());

    // Bob computes the shared secret with Alice's public key
    let shared_secret_bob = montgomery_ladder(&alice_public_key, &bob_secret_key);
    println!("Bob's computed shared secret:   {:?}", shared_secret_bob.0.to_biguint());

    // 4. Verification
    assert_eq!(shared_secret_alice, shared_secret_bob);
    println!("\nShared secrets match! The key exchange was successful.");
}
