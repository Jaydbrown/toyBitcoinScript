use sha2::{Digest, Sha256};

pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    let mut out = [0u8; 32];
    out.copy_from_slice(&second);
    out
}

pub fn compute_merkle_root(mut hashes: Vec<[u8; 32]>) -> Result<[u8; 32], &'static str> {
    if hashes.is_empty() {
        return Err("Cannot compute merkle root of empty transaction list");
    }

    while hashes.len() > 1 {
        // Bitcoin rule: if odd number of hashes, duplicate the last one
        if hashes.len() % 2 != 0 {
            let last = *hashes.last().unwrap();
            hashes.push(last);
        }

        let mut next_level = Vec::with_capacity(hashes.len() / 2);
        for chunk in hashes.chunks_exact(2) {
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&chunk[0]);
            combined[32..].copy_from_slice(&chunk[1]);
            next_level.push(double_sha256(&combined));
        }

        hashes = next_level;
    }

    Ok(hashes[0])
}

