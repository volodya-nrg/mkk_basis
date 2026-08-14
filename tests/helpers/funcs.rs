use rand::Rng;

pub fn gen_priv_key_bytes(len: usize) -> Vec<u8> {
    let mut key = vec![0u8; len];
    rand::rng().fill_bytes(&mut key);
    key
}
