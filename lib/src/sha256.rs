use crate::U256;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fmt;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Hash(U256);

impl Hash {
    pub fn hash<T: Serialize>(data: &T) -> Self {
        let mut serialized = vec![];

        if let Err(e) = ciborium::into_writer(data, &mut serialized) {
            panic!(
                "Failed to serialized data : {:?}. \
                    this Should not happen",
                e
            );
        }

        let hash = digest(&serialized);
        let hash_bytes = hex::decode(hash).unwrap();

        let hash_array: [u8; 32] = hash_bytes.as_slice().try_into().unwrap();

        Hash(U256::from_big_endian(&hash_array))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let bytes: [u8; 32] = self.0.to_little_endian();

        bytes.as_slice().try_into().unwrap()
    }

    pub fn match_target(&self, target: U256) -> bool {
        self.0 < target
    }

    pub fn zero() -> Self {
        Hash(U256::zero())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use serde::Serialize;

    use crate::sha256::Hash;

    #[derive(Serialize)]
    struct TestData {
        a: u32,
        b: String,
    }

    #[test]
    fn test_hash() {
        let data = TestData {
            a: 420,
            b: String::from_str("Ryzen").unwrap(),
        };

        let h = Hash::hash(&data);

        println!("Hashed data: {:?}", h.0);
    }

    #[test]
    fn test_as_bytes() {
        let data = TestData {
            a: 420,
            b: String::from_str("Ryzen").unwrap(),
        };

        let bytes = Hash::hash(&data).to_bytes();

        println!("Bytes of data : {:?}", bytes);
    }
}
