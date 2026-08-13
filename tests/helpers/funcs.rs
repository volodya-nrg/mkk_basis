use rand::Rng;

pub fn generate_private_key_bytes(len: usize) -> Vec<u8> {
    let mut key = vec![0u8; len];
    rand::rng().fill_bytes(&mut key);
    key
}