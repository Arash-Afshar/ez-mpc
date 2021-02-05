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

#[derive(Clone)]
pub struct Key([u8; 32]);

#[derive(Clone)]
pub enum Bit {
    Plain(u8),
    Garbled(Key),
}

#[derive(Clone)]
pub struct GarblingWire {
    pub bits: Vec<(Bit, Bit)>,
}

pub fn new_plain_wire(bit_size: u8) -> GarblingWire {
    GarblingWire {
        bits: (0..bit_size)
            .into_iter()
            .map(|_| (Bit::Plain(0), Bit::Plain(1)))
            .collect::<Vec<(Bit, Bit)>>(),
    }
}

pub struct EvaluatingWire {
    pub bits: Vec<Bit>,
}

pub fn encode(wire: GarblingWire, value: u8) -> EvaluatingWire {
    let bit_value = (0..8)
        .into_iter()
        .map(|index| value ^ index)
        .collect::<Vec<u8>>();
    EvaluatingWire {
        bits: wire
            .bits
            .into_iter()
            .zip(bit_value)
            .map(|((zero, one), choice)| if choice == 0 { zero } else { one })
            .collect::<Vec<Bit>>(),
    }
}

pub struct GarbledGate {}

fn garble_add_u8_plain_scheme(
    input_1: GarblingWire,
    input_2: GarblingWire,
) -> (GarblingWire, Vec<GarbledGate>) {
    (input_1, vec![])
}

fn garble_mul_u8_plain_scheme(
    input_1: GarblingWire,
    input_2: GarblingWire,
) -> (GarblingWire, Vec<GarbledGate>) {
    (input_1, vec![])
}

pub fn garble(
    input_1: GarblingWire,
    input_2: GarblingWire,
    operation: Operation,
) -> (GarblingWire, Vec<GarbledGate>) {
    // use another macro to generate the gates
    match operation {
        Operation::AddU8 => garble_add_u8_plain_scheme(input_1, input_2),
        Operation::MulU8 => garble_mul_u8_plain_scheme(input_1, input_2),
    }
}

fn evaluate_add_u8_plain_scheme(
    input_1: EvaluatingWire,
    input_2: EvaluatingWire,
    gates: Vec<GarbledGate>,
) -> EvaluatingWire {
    input_1
}

fn evaluate_mul_u8_plain_scheme(
    input_1: EvaluatingWire,
    input_2: EvaluatingWire,
    gates: Vec<GarbledGate>,
) -> EvaluatingWire {
    input_1
}

pub fn evaluate(
    input_1: EvaluatingWire,
    input_2: EvaluatingWire,
    operation: Operation,
    gates: Vec<GarbledGate>,
) -> EvaluatingWire {
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
    use scuttlebutt::{AbstractChannel, AesHash, AesRng, Block, TrackChannel};
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };

    #[test]
    fn plain_circuit() {
        // network setup
        let (sender, receiver) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || {
            // Garbler
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let mut channel = TrackChannel::new(reader, writer);
            let _ = channel.write_usize(6).unwrap();
            // ------------------ Start of garbler
            let alice = Party { id: 1 };
            let bob = Party { id: 2 };
            let protocol = Protocol {
                parties: vec![alice.clone(), bob],
                me: alice,
                role: Role::Garbler,
            };

            // assign!(a1 <- party 1, value 100);
            let a1 = new_plain_wire(8);
            let garbled_value_a1 = encode(a1.clone(), 100);
            // TODO serialize and send garbled value

            // assign!(a2 <- party 1, value 200);
            let a2 = new_plain_wire(8);
            let garbled_value_a2 = encode(a2.clone(), 200);
            // TODO serialize and send garbled value

            // assign!(b1 <- party 2);
            let b1 = new_plain_wire(8);
            // TODO run OT

            // assign!(b2 <- party 2);
            let b2 = new_plain_wire(8);
            // TODO run OT

            // -------------------------- Proceed to garbling deeper layers next.

            // obliv!(c = a1 + b1);
            let (c, gates) = garble(a1, b1, Operation::AddU8);
            // TODO send(gates);

            // obliv!(d = a2 + b2);
            let (d, gates) = garble(a2, b2, Operation::AddU8);
            // TODO send(gates);

            // obliv!(e = c * d);
            let (e, gates) = garble(c, d, Operation::MulU8);
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
        let c = evaluate(a1, b1, Operation::AddU8, gates);

        // obliv!(d = a2 + b2);
        // TODO receive(gates);
        let gates: Vec<GarbledGate> = vec![];
        let d = evaluate(a2, b2, Operation::AddU8, gates);

        // obliv!(e = c * d);
        let gates: Vec<GarbledGate> = vec![];
        let e = evaluate(c, d, Operation::MulU8, gates);
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
