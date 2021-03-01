//! Provides the API for a 2PC protocol. The user of this crate is not expected to call these
//! directly. Instead, the user will call the macros and the macros call these APIs.

use rand_core::{CryptoRng, RngCore};
use scuttlebutt::Block;
use serde::{Deserialize, Serialize};
use std::marker::{PhantomData, Sized};

// ----------------------------------------------------------------------------------------------
// -                                  Type Definitions                                          -
// ----------------------------------------------------------------------------------------------

/// Represents a party in the computation. At the moment, we only consider 2-Party protocols where
/// one party is a garbler and the other is the evaluator.
#[derive(Clone)]
pub struct Party {
    pub id: usize,
    // address:  network address
}

/// The roles of the 2PC protocol.
pub enum Role {
    Garbler,
    Evaluator,
}

/// Stores metadata about the protocol. Specifically, the current party.
pub struct Protocol {
    pub parties: Vec<Party>,
    pub me: Party,
    pub role: Role,
}

/// The operations that are supported by the protocol. Insead of focusing on 1-bit logic gates, the
/// intention is to create higher level constructs that are used in writing typical programs.
pub enum Operation {
    AddU8,
    MulU8,
}

/// The main trait that distinguishes various garbling modes.
pub trait GarblingMode {
    /// Generates a pair of keys for the zero and one value of a 1-bit wire.
    fn pair<R: RngCore + CryptoRng>(rng: &mut R) -> (Self, Self)
    where
        Self: Sized;
    /// Converts the key of a wire to a `Block` so that it can be used with `swanky` library.
    fn to_block(&self) -> Block;
}

/// The main trait for a wire. The typical usecasse is to represent a value that can be seen in a
/// Rust program. Therefore, the wire is rarely single bit.
pub trait Wire {
    /// The Rust data primitive datatype that can be passed to functions implementing this trait.
    type ValueType;

    /// Returins the number of bits represented in this wire.
    fn bits() -> u32;
}

/// Represents a group of garbled wires defined by the garbling mode and the wire specificacion.
#[derive(Clone, Serialize, Deserialize)]
pub struct GarblingWire<M: GarblingMode, W: Wire> {
    pub bits: Vec<(M, M)>,
    wire_info: PhantomData<W>,
}

/// Represents a group of garbled values.
#[derive(Clone, Serialize, Deserialize)]
pub struct EvaluatingWire<M: GarblingMode> {
    pub bits: Vec<M>,
}

/// Represents a garbled gate.
#[derive(Clone, Serialize, Deserialize)]
pub struct Gate<M: GarblingMode, W: Wire> {
    pub output: GarblingWire<M, W>,
}

// ----------------------------------------------------------------------------------------------
// -                                    Impl blocks                                             -
// ----------------------------------------------------------------------------------------------

impl<M: GarblingMode, W: Wire> GarblingWire<M, W> {
    /// Generates garbled keys for all the wires and returns the garbled wires.
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> GarblingWire<M, W> {
        GarblingWire {
            wire_info: PhantomData,
            bits: (0..W::bits())
                .into_iter()
                .map(|_| M::pair(rng))
                .collect::<Vec<(M, M)>>(),
        }
    }

    /// TODO: change the type of value W::ValueType
    /// Encodes a value to to garbled value.
    pub fn encode(self, value: u8) -> EvaluatingWire<M> {
        EvaluatingWire {
            bits: self
                .bits
                .into_iter()
                .zip(to_bit_arr(value, W::bits()))
                .map(|((zero, one), choice)| if choice { one } else { zero })
                .collect::<Vec<M>>(),
        }
    }

    /// Returns the corresponding garbled keys as `Block`s to be used with the `swanky` library.
    pub fn to_blocks(self) -> Vec<(Block, Block)> {
        self.bits
            .into_iter()
            .map(|(zero, one)| (zero.to_block(), one.to_block()))
            .collect()
    }
}

// ----------------------------------------------------------------------------------------------
// -                                 Utility Functions                                          -
// ----------------------------------------------------------------------------------------------

pub(crate) fn to_bit_arr(value: u8, len: u32) -> Vec<bool> {
    let mask = 2u8.pow(len - 1);
    (0..len)
        .into_iter()
        .map(|index| (value & (mask >> index)) != 0)
        .collect::<Vec<bool>>()
}
