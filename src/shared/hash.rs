use sha2::{Digest, Sha256};

pub fn make_hash(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source);
    let hash = hasher.finalize();

    base16ct::lower::encode_string(&hash)
}

pub fn verify_hash(source: &str, hash: &str) -> bool {
    make_hash(source).eq_ignore_ascii_case(hash)
}
