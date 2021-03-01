use rand_core::{CryptoRng, RngCore};
use scuttlebutt::Block;
use serde::{Deserialize, Serialize};
use std::marker::{PhantomData, Sized};

#[derive(Clone)]
pub struct Party {
    pub id: usize,
    // address:  network address
}

pub enum Role {
    Garbler,
    Evaluator,
}

pub struct Protocol {
    pub parties: Vec<Party>,
    pub me: Party,
    pub role: Role,
}

pub enum Operation {
    AddU8,
    MulU8,
}

pub trait GarblingMode {
    fn pair<R: RngCore + CryptoRng>(rng: &mut R) -> (Self, Self)
    where
        Self: Sized;
    fn to_block(&self) -> Block;
}

pub trait Wire {
    type ValueType;
    fn bits() -> u32;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Wire8Bit {}

impl Wire for Wire8Bit {
    type ValueType = u8;
    fn bits() -> u32 {
        8
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GarblingWire<M: GarblingMode, W: Wire> {
    pub bits: Vec<(M, M)>,
    wire_info: PhantomData<W>,
}

impl<M: GarblingMode, W: Wire> GarblingWire<M, W> {
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

    pub fn to_blocks(self) -> Vec<(Block, Block)> {
        self.bits
            .into_iter()
            .map(|(zero, one)| (zero.to_block(), one.to_block()))
            .collect()
    }
}

pub(crate) fn to_bit_arr(value: u8, len: u32) -> Vec<bool> {
    let mask = 2u8.pow(len - 1);
    (0..len)
        .into_iter()
        .map(|index| (value & (mask >> index)) != 0)
        .collect::<Vec<bool>>()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EvaluatingWire<M: GarblingMode> {
    pub bits: Vec<M>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Gate<M: GarblingMode, W: Wire> {
    pub output: GarblingWire<M, W>,
}
