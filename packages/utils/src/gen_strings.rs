use rand::{Rng, distr::Alphanumeric, rng};
use ulid::Ulid;
use uuid::Uuid;

pub fn ulid_string() -> String {
    let ulid = Ulid::new();
    ulid.to_string()
}
pub fn uuid_sring() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

pub fn generate_random_string() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric) // Iterator over Alphanumeric characters
        .take(6) // Take the first 6 characters
        .map(char::from) // Convert bytes to characters
        .collect() // Collect into a String
}

pub fn generate_otp(length: usize) -> String {
    let mut rng = rng();
    (0..length).map(|_| rng.random_range(0..10).to_string()).collect()
}

#[inline(always)]
pub fn cal_hash(ulid_str: &str) -> u128 {
    // Convert to u128
    let ulid = Ulid::from_string(ulid_str).unwrap_or_default().0;
    // Extract upper 64 bits
    let high = (ulid >> 64) as u64;
    // Extract lower 64 bits
    let low = ulid as u64;

    high as u128 + low as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cal_hash_with_valid_ulid() {
        let ulid_str = "01JQ3PDXRW82F5Y2BTVCE63GNX"; // A valid ULID
        let ulid = Ulid::from_string(ulid_str).unwrap().0; // Convert to u128 manually

        let expected_high = (ulid >> 64) as u64;
        let expected_low = ulid as u64;

        let sum_ulid = expected_high as u128 + expected_low as u128;
        let cl = cal_hash(ulid_str);

        assert_eq!(sum_ulid, cl, "Test hash matches");

        let shard_a = (cl % 128) as u64;
        let shard_b = (sum_ulid % 128) as u64;

        println!("shard_a, {:?}", shard_a);

        assert_eq!(shard_a, shard_b, "shard_a and shard_b are equal..");
    }
}
