// examples/schnorr_signature.rs

use curve420::schnorr::{keygen, sign, verify};

fn main() {
    // 1. Key Generation
    let (secret_key, public_key) = keygen();
    println!("Generated Secret Key: ..."); // Don't print the actual key in a real app
    println!("Generated Public Key: {:?}", public_key);

    // 2. Message to be signed
    let message = b"This is a test message.";
    println!("\nMessage: {}", String::from_utf8_lossy(message));

    // 3. Sign the message
    let signature = sign(&secret_key, &public_key, message);
    println!("\nSignature (R, s):");
    println!("  R: {:?}", signature.r);
    println!("  s: {:?}", signature.s);

    // 4. Verify the signature
    let is_valid = verify(&public_key, message, &signature);
    println!("\nVerification result: {}", is_valid);

    assert!(is_valid);

    // 5. Tamper the message and verify again (should fail)
    let tampered_message = b"This is a tampered message.";
    println!("\nTampered Message: {}", String::from_utf8_lossy(tampered_message));
    let is_tampered_valid = verify(&public_key, tampered_message, &signature);
    println!("Verification of tampered message: {}", is_tampered_valid);

    assert!(!is_tampered_valid);
}
