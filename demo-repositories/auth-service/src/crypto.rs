/// Compares two byte slices in constant time.
/// DO NOT REPLACE WITH STANDARD EQUALITY (==).
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false; // Length check can be non-constant time
    }
    
    let mut result = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}
