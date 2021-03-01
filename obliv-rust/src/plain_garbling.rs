use crate::mpc_core::{EvaluatingWire, GarblingMode, GarblingWire, Gate, Operation, Wire8Bit};
use rand_core::{CryptoRng, RngCore};
use scuttlebutt::Block;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlainBit(pub Block);

impl GarblingMode for PlainBit {
    fn pair<R: RngCore + CryptoRng>(_rng: &mut R) -> (Self, Self) {
        (Self(Block::default()), Self(Block::default().set_lsb()))
    }

    fn to_block(&self) -> Block {
        self.0
    }
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

pub fn garble_add_u8_plain_scheme(
    input_1: GarblingWire<PlainBit, Wire8Bit>,
    _input_2: GarblingWire<PlainBit, Wire8Bit>,
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

pub fn garble_mul_u8_plain_scheme(
    input_1: GarblingWire<PlainBit, Wire8Bit>,
    _input_2: GarblingWire<PlainBit, Wire8Bit>,
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

pub fn evaluate_add_u8_plain_scheme(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    gates: Vec<Gate<PlainBit, Wire8Bit>>,
) -> EvaluatingWire<PlainBit> {
    let sum = to_u8(&input_1) + to_u8(&input_2);
    gates[0].clone().output.encode(sum)
}

pub fn evaluate_mul_u8_plain_scheme(
    input_1: EvaluatingWire<PlainBit>,
    _input_2: EvaluatingWire<PlainBit>,
    _gates: Vec<Gate<PlainBit, Wire8Bit>>,
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
    use crate::mpc_core::{
        to_bit_arr, EvaluatingWire, GarblingWire, Gate, Operation, Party, Protocol, Role, Wire8Bit,
    };
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
            let _protocol = Protocol {
                parties: vec![alice.clone(), bob],
                me: alice,
                role: Role::Garbler,
            };

            // assign!(a1 <- party 1, value 10);
            let a1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a1 = a1.clone().encode(10);
            let ser = bincode::serialize(&garbled_value_a1).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();

            // assign!(a2 <- party 1, value 20);
            let a2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a2 = a2.clone().encode(20);
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
        let _protocol = Protocol {
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
        assert_eq!(plain_g, 30);

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
            let _protocol = Protocol {
                parties: vec![alice.clone(), bob],
                me: alice,
                role: Role::Garbler,
            };

            // assign!(a1 <- party 1, value 10);
            let a1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a1 = a1.clone().encode(10);
            let ser = bincode::serialize(&garbled_value_a1).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            // assign!(a2 <- party 1, value 20);
            let a2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a2 = a2.clone().encode(20);
            let ser = bincode::serialize(&garbled_value_a2).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            // assign!(b1 <- party 2);
            let b1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            ot.send(&mut channel, &b1.clone().to_blocks(), &mut rng)
                .unwrap();

            // assign!(b2 <- party 2);
            let b2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            ot.send(&mut channel, &b2.clone().to_blocks(), &mut rng)
                .unwrap();

            // -------------------------- Proceed to garbling deeper layers next.

            // obliv!(c = a1 + b1);
            let (c, gates) = garble_u8_gate_plain(a1, b1, Operation::AddU8);
            let ser = bincode::serialize(&gates).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            // obliv!(d = a2 + b2);
            let (d, gates) = garble_u8_gate_plain(a2, b2, Operation::AddU8);
            let ser = bincode::serialize(&gates).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            // obliv!(e = c * d);
            let (_e, gates) = garble_u8_gate_plain(c, d, Operation::AddU8);
            let ser = bincode::serialize(&gates).unwrap();
            channel.write_usize(ser.len()).unwrap();
            channel.write_bytes(&ser).unwrap();
            channel.flush().unwrap();

            //// reveal!(e);
            //// TODO send(gates.mapping);
            //// TODO receive(plain_e);
        });
        let mut rng = AesRng::new();
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let mut channel = TrackChannel::new(reader, writer);
        let mut ot = ChouOrlandiReceiver::init(&mut channel, &mut rng).unwrap();

        // ------------------ Start of evaluator

        let alice = Party { id: 1 };
        let bob = Party { id: 2 };
        let _protocol = Protocol {
            parties: vec![alice, bob.clone()],
            me: bob,
            role: Role::Evaluator,
        };

        // assign!(a1 <- party 1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let a1: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();
        assert_eq!(to_u8(&a1), 10, "a1");

        // assign!(a2 <- party 1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let a2: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();
        assert_eq!(to_u8(&a2), 20, "a2");

        // assign!(b1 <- party 2, value 25);
        let bs = to_bit_arr(25, 8);
        let results = ot.receive(&mut channel, &bs, &mut rng).unwrap();
        let bits = results
            .into_iter()
            .map(|block| PlainBit(block))
            .collect::<Vec<PlainBit>>();
        let b1 = EvaluatingWire::<PlainBit> { bits };
        println!("{:?}", bs);
        assert_eq!(to_u8(&b1), 25, "b1");

        // assign!(b2 <- party 2, value 30);
        let bs = to_bit_arr(30, 8);
        let results = ot.receive(&mut channel, &bs, &mut rng).unwrap();
        let bits = results
            .into_iter()
            .map(|block| PlainBit(block))
            .collect::<Vec<PlainBit>>();
        let b2 = EvaluatingWire::<PlainBit> { bits };
        assert_eq!(to_u8(&b2), 30, "b2");

        // obliv!(c = a1 + b1);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = bincode::deserialize(&ser).unwrap();
        let c = evaluate_plain(a1, b1, Operation::AddU8, gates);
        assert_eq!(to_u8(&c), 35, "c");

        // obliv!(d = a2 + b2);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = bincode::deserialize(&ser).unwrap();
        let d = evaluate_plain(a2, b2, Operation::AddU8, gates);
        assert_eq!(to_u8(&d), 50, "d");

        // obliv!(e = c * d);
        let size = channel.read_usize().unwrap();
        let ser = channel.read_vec(size).unwrap();
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = bincode::deserialize(&ser).unwrap();
        let e = evaluate_plain(c, d, Operation::AddU8, gates);
        assert_eq!(to_u8(&e), 85, "e");

        //// reveal!(e);
        //// TODO send(gates.mapping);
        //// TODO receive(plain_e);

        handle.join().unwrap();
    }
}
