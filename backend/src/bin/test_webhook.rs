use backend::services::github;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn sign(secret: &str, payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

fn main() {
    let secret = "my-test-secret";
    let payload = b"{\"action\":\"closed\",\"pull_request\":{\"merged\":true}}";

    let sig = sign(secret, payload);
    println!("Payload: {}", String::from_utf8_lossy(payload));
    println!("Signature: {}", sig);

    // Test valid signature
    let valid = github::verify_webhook_signature(secret, payload, &sig);
    assert!(valid, "Expected valid signature to pass");
    println!("✅ Valid signature: PASS");

    // Test mutated payload
    let mutated = b"{\"action\":\"closed\",\"pull_request\":{\"merged\":false}}";
    let invalid = github::verify_webhook_signature(secret, mutated, &sig);
    assert!(!invalid, "Expected invalid signature to fail");
    println!("✅ Mutated payload rejected: PASS");

    // Test wrong secret
    let wrong_secret = github::verify_webhook_signature("wrong-secret", payload, &sig);
    assert!(!wrong_secret, "Expected wrong secret to fail");
    println!("✅ Wrong secret rejected: PASS");

    println!("\nAll webhook signature tests passed!");
}
