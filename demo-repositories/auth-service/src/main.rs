mod crypto;

fn main() {
    let secret = b"super_secret_key";
    let input = b"super_secret_kex";
    
    // As per SEC-4, use constant_time_compare to validate
    if crypto::constant_time_compare(secret, input) {
        println!("Authenticated!");
    } else {
        println!("Access Denied.");
    }
}
