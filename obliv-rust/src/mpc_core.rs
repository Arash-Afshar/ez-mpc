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

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlainBit(pub Block);

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GarbledBit(pub Block);

impl GarblingMode for PlainBit {
    fn pair<R: RngCore + CryptoRng>(_rng: &mut R) -> (Self, Self) {
        (Self(Block::default()), Self(Block::default().set_lsb()))
    }

    fn to_block(&self) -> Block {
        self.0
    }
}

impl GarblingMode for GarbledBit {
    fn pair<R: RngCore + CryptoRng>(rng: &mut R) -> (Self, Self) {
        let mut buffer: [u8; 16] = [0; 16];
        rng.fill_bytes(&mut buffer);
        let zero = Block::from(buffer);

        buffer = [0; 16];
        rng.fill_bytes(&mut buffer);
        let one = Block::from(buffer);

        (Self(zero), Self(one))
    }

    fn to_block(&self) -> Block {
        self.0
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
                .map(|((zero, one), choice)| if choice { zero } else { one })
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

fn to_bit_arr(value: u8, len: u32) -> Vec<bool> {
    let mask = 2u8.pow(len - 1);
    (0..len)
        .into_iter()
        .map(|index| (value & (mask >> index)) == 0)
        .collect::<Vec<bool>>()
}

fn to_u8(garbled_value: &EvaluatingWire<PlainBit>) -> u8 {
    garbled_value
        .bits
        .iter()
        .rev()
        .fold((0, 0), |(acc, idx), p_bit| {
            let bit = if p_bit.0 == Block::default() { 0 } else { 1 };
            (acc + 2u8.pow(idx) * bit, idx + 1)
        })
        .0
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EvaluatingWire<M: GarblingMode> {
    pub bits: Vec<M>,
}

//#[derive(Clone)]
//pub struct PlainGate {
//    output: GarblingWire<PlainBit, Wire8Bit>,
//}

#[derive(Clone, Serialize, Deserialize)]
pub struct Gate<M: GarblingMode, W: Wire> {
    pub output: GarblingWire<M, W>,
}

fn garble_add_u8_plain_scheme(
    input_1: GarblingWire<PlainBit, Wire8Bit>,
    input_2: GarblingWire<PlainBit, Wire8Bit>,
) -> (
    GarblingWire<PlainBit, Wire8Bit>,
    Vec<Gate<PlainBit, Wire8Bit>>,
) {
    // Plain wires are always 0 and 1 so it does not matter whether we generate a new output or
    // reuse input.
    (
        input_1.clone(),
        vec![Gate::<PlainBit, Wire8Bit> { output: input_1 }],
    )
}

fn garble_mul_u8_plain_scheme(
    input_1: GarblingWire<PlainBit, Wire8Bit>,
    input_2: GarblingWire<PlainBit, Wire8Bit>,
) -> (
    GarblingWire<PlainBit, Wire8Bit>,
    Vec<Gate<PlainBit, Wire8Bit>>,
) {
    (input_1, vec![])
}

pub fn garble_u8_gate_plain(
    input_1: GarblingWire<PlainBit, Wire8Bit>,
    input_2: GarblingWire<PlainBit, Wire8Bit>,
    operation: Operation,
) -> (
    GarblingWire<PlainBit, Wire8Bit>,
    Vec<Gate<PlainBit, Wire8Bit>>,
) {
    // use another macro to generate the gates
    match operation {
        Operation::AddU8 => garble_add_u8_plain_scheme(input_1, input_2),
        Operation::MulU8 => garble_mul_u8_plain_scheme(input_1, input_2),
    }
}

fn evaluate_add_u8_plain_scheme(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    gates: Vec<Gate<PlainBit, Wire8Bit>>,
) -> EvaluatingWire<PlainBit> {
    let sum = to_u8(&input_1) + to_u8(&input_2);
    gates[0].clone().output.encode(sum)
}

fn evaluate_mul_u8_plain_scheme(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    gates: Vec<Gate<PlainBit, Wire8Bit>>,
) -> EvaluatingWire<PlainBit> {
    input_1
}

pub fn evaluate_plain(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    operation: Operation,
    gates: Vec<Gate<PlainBit, Wire8Bit>>,
) -> EvaluatingWire<PlainBit> {
    // use another macro to generate the gates
    match operation {
        Operation::AddU8 => evaluate_add_u8_plain_scheme(input_1, input_2, gates),
        Operation::MulU8 => evaluate_mul_u8_plain_scheme(input_1, input_2, gates),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocelot::ot::{ChouOrlandiReceiver, ChouOrlandiSender, Receiver, Sender};
    use rand::{rngs::StdRng, SeedableRng};
    use scuttlebutt::{AbstractChannel, AesRng, Block, TrackChannel};
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };
    const SEED: [u8; 32] = [42u8; 32];

    #[test]
    fn test_plain_garbling() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
        let garbled_value = garbled_wires.encode(6);
        let expect = vec![0, 0, 0, 0, 0, 1, 1, 0];
        let got = garbled_value
            .bits
            .into_iter()
            .map(|g_bit| if g_bit.0 == Block::default() { 0 } else { 1 })
            .collect::<Vec<u8>>();
        assert_eq!(expect, got);
    }

    #[test]
    fn test_plain_decode_to_u8() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
        let garbled_value = garbled_wires.encode(6);
        let got = to_u8(&garbled_value);
        assert_eq!(6, got);
    }

    #[test]
    fn test_plain_add_u8() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires_1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
        let garbled_value_1 = garbled_wires_1.clone().encode(6);
        let garbled_wires_2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
        let garbled_value_2 = garbled_wires_2.clone().encode(6);
        let (_, gates) = garble_add_u8_plain_scheme(garbled_wires_1, garbled_wires_2);
        let result = evaluate_add_u8_plain_scheme(garbled_value_1, garbled_value_2, gates);

        let got = to_u8(&result);

        assert_eq!(12, got);
    }

    #[test]
    fn test_non_plain_garbling() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<GarbledBit, Wire8Bit>::new(&mut rng);
        let garbled_value = garbled_wires.clone().encode(6);
        let expect = vec![
            garbled_wires.clone().bits[0].0 .0,
            garbled_wires.clone().bits[1].0 .0,
            garbled_wires.clone().bits[2].0 .0,
            garbled_wires.clone().bits[3].0 .0,
            garbled_wires.clone().bits[4].0 .0,
            garbled_wires.clone().bits[5].1 .0,
            garbled_wires.clone().bits[6].1 .0,
            garbled_wires.clone().bits[7].0 .0,
        ];
        let got = garbled_value
            .bits
            .into_iter()
            .map(|g_bit| g_bit.0)
            .collect::<Vec<Block>>();
        assert_eq!(expect, got);
    }

    #[test]
    fn test_serde_garbled_wires() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<GarbledBit, Wire8Bit>::new(&mut rng);
        let serialized_garbled_wires = bincode::serialize(&garbled_wires).unwrap();
        let deserialized_garbled_wires: GarblingWire<GarbledBit, Wire8Bit> =
            bincode::deserialize(&serialized_garbled_wires).unwrap();

        assert!(garbled_wires
            .clone()
            .bits
            .into_iter()
            .zip(deserialized_garbled_wires.bits)
            .fold(true, |acc, (want, got)| acc || (want.0 == got.0)));
    }

    #[test]
    fn test_serde_eval_wires() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<GarbledBit, Wire8Bit>::new(&mut rng);
        let garbled_value = garbled_wires.encode(6);
        let serialized_garbled_value = bincode::serialize(&garbled_value).unwrap();
        let deserialized_garbled_value: EvaluatingWire<GarbledBit> =
            bincode::deserialize(&serialized_garbled_value).unwrap();
        assert!(garbled_value
            .clone()
            .bits
            .into_iter()
            .zip(deserialized_garbled_value.bits)
            .fold(true, |acc, (want, got)| acc || (want.0 == got.0)));
    }

    #[test]
    fn test_serde_garbled_gate() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<GarbledBit, Wire8Bit>::new(&mut rng);
        let gate = Gate::<GarbledBit, Wire8Bit> {
            output: garbled_wires,
        };

        let serialized_gate = bincode::serialize(&gate).unwrap();
        let deserialized_gate: Gate<GarbledBit, Wire8Bit> =
            bincode::deserialize(&serialized_gate).unwrap();
        assert!(gate
            .clone()
            .output
            .bits
            .into_iter()
            .zip(deserialized_gate.output.bits)
            .fold(true, |acc, (want, got)| acc || (want.0 == got.0)));
    }

    #[test]
    fn test_serde_plain_gate() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
        let gate = Gate::<PlainBit, Wire8Bit> {
            output: garbled_wires,
        };

        let serialized_gate = bincode::serialize(&gate).unwrap();
        let deserialized_gate: Gate<PlainBit, Wire8Bit> =
            bincode::deserialize(&serialized_gate).unwrap();
        assert!(gate
            .clone()
            .output
            .bits
            .into_iter()
            .zip(deserialized_gate.output.bits)
            .fold(true, |acc, (want, got)| acc || (want.0 == got.0)));
    }

    #[test]
    fn test_pipe_send_vec_u8() {
        // network setup
        let message: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let message_clone = message.clone();
        let (sender, receiver) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || {
            // Garbler
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let mut channel = TrackChannel::new(reader, writer);
            channel.write_bytes(&message).unwrap();
        });

        // Evaluator
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let mut channel = TrackChannel::new(reader, writer);
        let res = channel.read_vec(message_clone.len()).unwrap();
        handle.join().unwrap();
        assert_eq!(message_clone, res);
    }

    #[test]
    fn plain_circuit_without_ot() {
        // network setup
        let (sender, receiver) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || {
            let mut rng = StdRng::from_seed(SEED);

            // Garbler
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let mut channel = TrackChannel::new(reader, writer);
            // ------------------ Start of the Garbler
            let alice = Party { id: 1 };
            let bob = Party { id: 2 };
            let protocol = Protocol {
                parties: vec![alice.clone(), bob],
                me: alice,
                role: Role::Garbler,
            };

            // assign!(a1 <- party 1, value 100);
            let a1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a1 = a1.clone().encode(100);
            let ser = bincode::serialize(&garbled_value_a1).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();

            // assign!(a2 <- party 1, value 200);
            let a2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a2 = a2.clone().encode(50);
            let ser = bincode::serialize(&garbled_value_a2).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();

            // obliv!(g = a1 + a2);
            let (_, gates) = garble_u8_gate_plain(a1, a2, Operation::AddU8);
            let ser = bincode::serialize(&gates).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();

            // reveal!(g);
            // TODO: send decoding
            // TODO: receive the plain value
        });

        // Evaluator
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let mut channel = TrackChannel::new(reader, writer);
        // ------------------ Start of evaluator
        let alice = Party { id: 1 };
        let bob = Party { id: 2 };
        let protocol = Protocol {
            parties: vec![alice, bob.clone()],
            me: bob,
            role: Role::Evaluator,
        };
        // assign!(a1 <- party 1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let a1: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();

        // assign!(a2 <- party 1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let a2: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();

        // obliv!(g = a1 + a2);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = bincode::deserialize(&ser).unwrap();
        let g = evaluate_plain(a1, a2, Operation::AddU8, gates);

        // reveal!(g);
        // TODO receive decoding and decode.
        let plain_g = to_u8(&g);
        assert_eq!(plain_g, 150);

        handle.join().unwrap();
        println!(
            "Receiver communication (read): {:.2} Mb",
            channel.kilobits_read() / 1000.0
        );
        println!(
            "Receiver communication (write): {:.2} Mb",
            channel.kilobits_written() / 1000.0
        );
    }

    #[test]
    fn test_plain_circuit_with_ot() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || {
            // Garbler
            let mut rng = AesRng::new();
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let mut channel = TrackChannel::new(reader, writer);
            let mut ot = ChouOrlandiSender::init(&mut channel, &mut rng).unwrap();

            // ------------------ Start of the Garbler
            let alice = Party { id: 1 };
            let bob = Party { id: 2 };
            let protocol = Protocol {
                parties: vec![alice.clone(), bob],
                me: alice,
                role: Role::Garbler,
            };

            // assign!(a1 <- party 1, value 100);
            let a1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a1 = a1.clone().encode(100);
            let ser = bincode::serialize(&garbled_value_a1).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            // assign!(a2 <- party 1, value 200);
            let a2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a2 = a2.clone().encode(50);
            let ser = bincode::serialize(&garbled_value_a2).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            // assign!(b1 <- party 2);
            let b1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            ot.send(&mut channel, &b1.to_blocks(), &mut rng).unwrap();

            // TODO: After OT: // assign!(b2 <- party 2);
            // TODO: After OT: let b2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            // TODO: After OT: // TODO run OT

            // TODO: After OT: // -------------------------- Proceed to garbling deeper layers next.

            // TODO: After OT: // obliv!(c = a1 + b1);
            // TODO: After OT: let (c, gates) = garble_u8_gate_plain(a1, b1, Operation::AddU8);
            // TODO: After OT: // TODO send(gates);

            // TODO: After OT: // obliv!(d = a2 + b2);
            // TODO: After OT: let (d, gates) = garble_u8_gate_plain(a2, b2, Operation::AddU8);
            // TODO: After OT: // TODO send(gates);

            // TODO: After OT: // obliv!(e = c * d);
            // TODO: After OT: let (e, gates) = garble_u8_gate_plain(c, d, Operation::MulU8);
            // TODO: After OT: // TODO send(gates);

            // TODO: After OT: // reveal!(e);
            // TODO: After OT: // TODO send(gates.mapping);
            // TODO: After OT: // TODO receive(plain_e);
        });
        let mut rng = AesRng::new();
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let mut channel = TrackChannel::new(reader, writer);
        let mut ot = ChouOrlandiReceiver::init(&mut channel, &mut rng).unwrap();

        // ------------------ Start of evaluator

        let alice = Party { id: 1 };
        let bob = Party { id: 2 };
        let protocol = Protocol {
            parties: vec![alice, bob.clone()],
            me: bob,
            role: Role::Evaluator,
        };

        // assign!(a1 <- party 1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let a1: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();

        // assign!(a2 <- party 1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let a2: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();

        // assign!(b1 <- party 2, value 255);
        let bs = to_bit_arr(255, 8);
        let results = ot.receive(&mut channel, &bs, &mut rng).unwrap();
        let b1_bits = results
            .into_iter()
            .map(|block| PlainBit(block))
            .collect::<Vec<PlainBit>>();
        let b1 = EvaluatingWire::<PlainBit> { bits: b1_bits };

        // TODO: After OT: // assign!(b2 <- party 2, value 400);
        // TODO: After OT: // TODO run OT and receive the garbled value
        // TODO: After OT: let b2 = EvaluatingWire { bits: vec![] };

        // TODO: After OT: // obliv!(c = a1 + b1);
        // TODO: After OT: // TODO receive(gates);
        // TODO: After OT: let gates: Vec<Gate<PlainBit, Wire8Bit>> = vec![];
        // TODO: After OT: let c = evaluate_plain(a1, b1, Operation::AddU8, gates);

        // TODO: After OT: // obliv!(d = a2 + b2);
        // TODO: After OT: // TODO receive(gates);
        // TODO: After OT: let gates: Vec<Gate<PlainBit, Wire8Bit>> = vec![];
        // TODO: After OT: let d = evaluate_plain(a2, b2, Operation::AddU8, gates);

        // TODO: After OT: // obliv!(e = c * d);
        // TODO: After OT: let gates: Vec<Gate<PlainBit, Wire8Bit>> = vec![];
        // TODO: After OT: let e = evaluate_plain(c, d, Operation::MulU8, gates);
        // TODO: After OT: // TODO receive(gates);

        // TODO: After OT: // reveal!(e);
        // TODO: After OT: // TODO send(gates.mapping);
        // TODO: After OT: // TODO receive(plain_e);

        handle.join().unwrap();
    }
}
