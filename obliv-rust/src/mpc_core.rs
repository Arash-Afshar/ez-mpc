use rand_core::{CryptoRng, RngCore};

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

//pub struct CircutOperation {
//    pub input_1: WireName,
//    pub input_2: WireName,
//    pub output: WireName,
//    pub op: Operation,
//}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct Key([u8; 32]);

impl Key {
    fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);
        Self(key)
    }
}

#[derive(Clone)]
pub struct PlainBit(pub u8);

#[derive(Clone)]
pub struct GarbledBit(pub Key);

pub trait BitType<R: RngCore + CryptoRng> {
    fn pair(rng: &mut R) -> (Self, Self)
    where
        Self: std::marker::Sized;
}

impl<R: RngCore + CryptoRng> BitType<R> for PlainBit {
    fn pair(_rng: &mut R) -> (Self, Self) {
        (Self(0), Self(1))
    }
}

impl<R: RngCore + CryptoRng> BitType<R> for GarbledBit {
    fn pair(rng: &mut R) -> (Self, Self) {
        (Self(Key::new(rng)), Self(Key::new(rng)))
    }
}

//#[derive(Clone)]
//pub enum Bit {
//    Plain(u8),
//    Garbled(Key),
//}

#[derive(Clone)]
pub struct GarblingWire<Bit> {
    pub bits: Vec<(Bit, Bit)>,
}

pub trait BitSize {
    fn bits() -> u32;
}

impl BitSize for u8 {
    fn bits() -> u32 {
        8
    }
}

pub fn garble_wires<T, S, R>(rng: &mut R) -> GarblingWire<T>
where
    S: BitSize,
    R: RngCore + CryptoRng,
    T: BitType<R>,
{
    GarblingWire {
        bits: (0..S::bits())
            .into_iter()
            .map(|_| T::pair(rng))
            .collect::<Vec<(T, T)>>(),
    }
}

pub struct EvaluatingWire<Bit> {
    pub bits: Vec<Bit>,
}

/// TODO: change the type of value to S
pub fn encode<T, S>(wire: GarblingWire<T>, value: u8) -> EvaluatingWire<T>
where
    S: BitSize,
{
    let mask = 2u8.pow(S::bits() - 1);
    let bit_value = (0..S::bits())
        .into_iter()
        .map(|index| (value & (mask >> index)) == 0)
        .collect::<Vec<bool>>();
    println!("{}", value);
    println!("{:?}", bit_value);
    EvaluatingWire {
        bits: wire
            .bits
            .into_iter()
            .zip(bit_value)
            .map(|((zero, one), choice)| if choice { zero } else { one })
            .collect::<Vec<T>>(),
    }
}

pub struct GarbledGate {}

fn garble_add_u8_plain_scheme(
    input_1: GarblingWire<PlainBit>,
    input_2: GarblingWire<PlainBit>,
) -> (GarblingWire<PlainBit>, Vec<GarbledGate>) {
    (input_1, vec![])
}

fn garble_mul_u8_plain_scheme(
    input_1: GarblingWire<PlainBit>,
    input_2: GarblingWire<PlainBit>,
) -> (GarblingWire<PlainBit>, Vec<GarbledGate>) {
    (input_1, vec![])
}

pub fn garble_u8_gate_plain(
    input_1: GarblingWire<PlainBit>,
    input_2: GarblingWire<PlainBit>,
    operation: Operation,
) -> (GarblingWire<PlainBit>, Vec<GarbledGate>) {
    // use another macro to generate the gates
    match operation {
        Operation::AddU8 => garble_add_u8_plain_scheme(input_1, input_2),
        Operation::MulU8 => garble_mul_u8_plain_scheme(input_1, input_2),
    }
}

fn evaluate_add_u8_plain_scheme(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    gates: Vec<GarbledGate>,
) -> EvaluatingWire<PlainBit> {
    input_1
}

fn evaluate_mul_u8_plain_scheme(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    gates: Vec<GarbledGate>,
) -> EvaluatingWire<PlainBit> {
    input_1
}

pub fn evaluate_plain(
    input_1: EvaluatingWire<PlainBit>,
    input_2: EvaluatingWire<PlainBit>,
    operation: Operation,
    gates: Vec<GarbledGate>,
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

        let garbled_wires = garble_wires::<PlainBit, u8, _>(&mut rng);
        let garbled_value = encode::<PlainBit, u8>(garbled_wires, 6);
        let expect = vec![0, 0, 0, 0, 0, 1, 1, 0];
        let got = garbled_value
            .bits
            .into_iter()
            .map(|g_bit| g_bit.0)
            .collect::<Vec<u8>>();
        assert_eq!(expect, got);
    }

    #[test]
    fn test_non_plain_garbling() {
        let mut rng = StdRng::from_seed(SEED);

        let garbled_wires = garble_wires::<GarbledBit, u8, _>(&mut rng);
        let garbled_value = encode::<GarbledBit, u8>(garbled_wires.clone(), 6);
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
            let a1 = garble_wires::<PlainBit, u8, _>(&mut rng);
            let garbled_value_a1 = encode::<PlainBit, u8>(a1.clone(), 100);
            // TODO serialize and send garbled value

            // assign!(a2 <- party 1, value 200);
            let a2 = garble_wires::<PlainBit, u8, _>(&mut rng);
            let garbled_value_a2 = encode::<PlainBit, u8>(a2.clone(), 200);
            // TODO serialize and send garbled value

            // assign!(b1 <- party 2);
            let b1 = garble_wires::<PlainBit, u8, _>(&mut rng);
            // TODO run OT

            // assign!(b2 <- party 2);
            let b2 = garble_wires::<PlainBit, u8, _>(&mut rng);
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
        let gates: Vec<GarbledGate> = vec![];
        let c = evaluate_plain(a1, b1, Operation::AddU8, gates);

        // obliv!(d = a2 + b2);
        // TODO receive(gates);
        let gates: Vec<GarbledGate> = vec![];
        let d = evaluate_plain(a2, b2, Operation::AddU8, gates);

        // obliv!(e = c * d);
        let gates: Vec<GarbledGate> = vec![];
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
