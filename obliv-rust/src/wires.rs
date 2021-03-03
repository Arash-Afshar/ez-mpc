//! Defines a struct for each type that is supported by the 2PC protocol.

use crate::mpc_core::Wire;
use serde::{Deserialize, Serialize};

/// A wire that represents a `u8` value.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Wire8Bit {}

impl Wire for Wire8Bit {
    type ValueType = u8;
    fn bits() -> u32 {
        8
    }
}
