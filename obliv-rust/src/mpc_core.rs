use rand_core::{CryptoRng, RngCore};
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
}

pub trait Wire {
    type ValueType;
    fn bits() -> u32;
}

#[derive(Clone)]
pub struct Wire8Bit {}

impl Wire for Wire8Bit {
    type ValueType = u8;
    fn bits() -> u32 {
        8
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct Key([u8; 32]);

impl Key {
    fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);
        Self(key)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlainBit(pub u8);

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GarbledBit(pub Key);

impl GarblingMode for PlainBit {
    fn pair<R: RngCore + CryptoRng>(_rng: &mut R) -> (Self, Self) {
        (Self(0), Self(1))
    }
}

impl GarblingMode for GarbledBit {
    fn pair<R: RngCore + CryptoRng>(rng: &mut R) -> (Self, Self) {
        (Self(Key::new(rng)), Self(Key::new(rng)))
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
        let mask = 2u8.pow(W::bits() - 1);
        let bit_value = (0..W::bits())
            .into_iter()
            .map(|index| (value & (mask >> index)) == 0)
            .collect::<Vec<bool>>();
        EvaluatingWire {
            bits: self
                .bits
                .into_iter()
                .zip(bit_value)
                .map(|((zero, one), choice)| if choice { zero } else { one })
                .collect::<Vec<M>>(),
        }
    }
}

fn to_u8(garbled_value: &EvaluatingWire<PlainBit>) -> u8 {
    garbled_value
        .bits
        .iter()
        .rev()
        .fold((0, 0), |(acc, idx), p_bit| {
            (acc + 2u8.pow(idx) * p_bit.0, idx + 1)
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

#[derive(Clone)]
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
    use scuttlebutt::{AbstractChannel, AesHash, AesRng, Block, TrackChannel};
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
            .map(|g_bit| g_bit.0)
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
            .collect::<Vec<Key>>();
        assert_eq!(expect, got);
    }

    #[test]
    fn test_serde_garbled_wires() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = GarblingWire::<GarbledBit, Wire8Bit>::new(&mut rng);
        let serialized_garbled_wires = serde_json::to_vec(&garbled_wires).unwrap();
        let deserialized_garbled_wires: GarblingWire<GarbledBit, Wire8Bit> =
            serde_json::from_slice(&serialized_garbled_wires).unwrap();

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
        let serialized_garbled_value = serde_json::to_vec(&garbled_value).unwrap();
        let deserialized_garbled_value: EvaluatingWire<GarbledBit> =
            serde_json::from_slice(&serialized_garbled_value).unwrap();
        assert!(garbled_value
            .clone()
            .bits
            .into_iter()
            .zip(deserialized_garbled_value.bits)
            .fold(true, |acc, (want, got)| acc || (want.0 == got.0)));
    }

    #[test]
    #[ignore]
    fn plain_circuit() {
        // network setup
        let (sender, receiver) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || {
            let mut rng = StdRng::from_seed(SEED);

            // Garbler
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let mut channel = TrackChannel::new(reader, writer);
            let _ = channel.write_usize(6).unwrap();
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
            // TODO serialize and send garbled value

            // assign!(a2 <- party 1, value 200);
            let a2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            let garbled_value_a2 = a2.clone().encode(200);
            // TODO serialize and send garbled value

            // assign!(b1 <- party 2);
            let b1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            // TODO run OT

            // assign!(b2 <- party 2);
            let b2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
            // TODO run OT

            // -------------------------- Proceed to garbling deeper layers next.

            // obliv!(c = a1 + b1);
            let (c, gates) = garble_u8_gate_plain(a1, b1, Operation::AddU8);
            // TODO send(gates);

            // obliv!(d = a2 + b2);
            let (d, gates) = garble_u8_gate_plain(a2, b2, Operation::AddU8);
            // TODO send(gates);

            // obliv!(e = c * d);
            let (e, gates) = garble_u8_gate_plain(c, d, Operation::MulU8);
            // TODO send(gates);

            // reveal!(e);
            // TODO send(gates.mapping);
            // TODO receive(plain_e);
        });

        // Evaluator
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let mut channel = TrackChannel::new(reader, writer);
        println!("-------------- Received: {}", channel.read_usize().unwrap());
        // ------------------ Start of evaluator
        let alice = Party { id: 1 };
        let bob = Party { id: 2 };
        let protocol = Protocol {
            parties: vec![alice, bob.clone()],
            me: bob,
            role: Role::Evaluator,
        };
        // assign!(a1 <- party 1);
        // TODO receive the garbled value
        let a1 = EvaluatingWire { bits: vec![] };

        // assign!(a2 <- party 1);
        // TODO receive the garbled value
        let a2 = EvaluatingWire { bits: vec![] };

        // assign!(b1 <- party 2, value 300);
        // TODO run OT and receive the garbled value
        let b1 = EvaluatingWire { bits: vec![] };

        // assign!(b2 <- party 2, value 400);
        // TODO run OT and receive the garbled value
        let b2 = EvaluatingWire { bits: vec![] };

        // obliv!(c = a1 + b1);
        // TODO receive(gates);
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = vec![];
        let c = evaluate_plain(a1, b1, Operation::AddU8, gates);

        // obliv!(d = a2 + b2);
        // TODO receive(gates);
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = vec![];
        let d = evaluate_plain(a2, b2, Operation::AddU8, gates);

        // obliv!(e = c * d);
        let gates: Vec<Gate<PlainBit, Wire8Bit>> = vec![];
        let e = evaluate_plain(c, d, Operation::MulU8, gates);
        // TODO receive(gates);

        // reveal!(e);
        // TODO send(gates.mapping);
        // TODO receive(plain_e);

        // -------------------------- Proceed to garbling deeper layers next.

        handle.join().unwrap();
        println!(
            "Receiver communication (read): {:.2} Mb",
            channel.kilobits_read() / 1000.0
        );
        println!(
            "Receiver communication (write): {:.2} Mb",
            channel.kilobits_written() / 1000.0
        );

        //let alice = Party { id: 1 };
        //let bob = Party { id: 1 };

        //let protocol = Protocol {
        //    parties: vec![alice.clone(), bob],
        //    me: alice,
        //    role: Role::Garbler,
        //};
        //// alice's input: a1, a2
        //// bob's   input: b1, b2
        //// function: ( a1 + b1 ) * (a2 + b2)

        //// obliv!(c = a + b) -> based on the role, do the garbling or evaluating
        //// reveal!(c)
        //// assign!(a <- party 1, value)

        //let a1 = GarblingWire { bits: vec![] };
        //let a2 = GarblingWire { bits: vec![] };
        //let b1 = GarblingWire { bits: vec![] };
        //let b2 = GarblingWire { bits: vec![] };

        //// assign!(a1 <- party 1, 10);
        //// assign!(a2 <- party 1, 20);
        //// assign!(b1 <- party 2, 30);
        //// assign!(b2 <- party 2, 40);

        //// obliv!(c = a1 + b1) as garbler
        //let (c, gates) = garble(a1, b1, Operation::AddU32);
        //send(gates);

        //// obliv!(c = a1 + b1) as evaluator
        //let gates = receive();
        //let c = evaluate(a1, b1, gates);

        //// obliv!(d = a2 + b2);
        //// obliv!(e = c + d);

        //// reveal!(e) as garbler
        //send(e.mapping);
        //let plain_e = receive();

        //// reveal!(e) as evaluator
        //let mapping = receive();
        //let plain_e = decode(e, mapping);
        //send(plain_e);
    }
}
