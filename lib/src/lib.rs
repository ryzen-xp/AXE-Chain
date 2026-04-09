use serde::{Deserialize, Serialize};
use uint::construct_uint;

construct_uint! {

  #[derive(Serialize, Deserialize)]
  pub struct U256(4);
}

pub const INITIAL_REWARD: i32 = 1;

pub const HALVING_INTERVAL: i32 = 23;

pub mod crypto;
pub mod error;
pub mod sha256;
pub mod types;
pub mod util;
