//! This module implements the macros that will expand the `obliv` keyword into an MPC protocol.
//!

use crate::{
    mpc_core::{
        EvaluatingWire, GarblingMode, GarblingWire, Gate, Operation, Party, Protocol, Role, Wire,
    },
    plain_garbling::PlainBit,
    wires::Wire8Bit,
};
use rand_core::{CryptoRng, RngCore};

#[macro_export]
macro_rules! obliv {
    ($a:ident + $b:ident) => {{
        {
            let val: usize = $a + $b;
            val
        }
    }};
    ($a:ident - $b:ident) => {{
        {
            let val: usize = $a - $b;
            val
        }
    }};
}

/// Example:
/// One side calls:       assign!(a, value 10);
/// The other side calls: assign!(a);
///
/// If the garbler calls the first statement, then it encodes and sends.
/// If the evaluator calls the first statement, then they run OT.
#[macro_export]
macro_rules! assign {
    ($a:ident, $c:expr, $p:expr, $g:ty, $w:ty) => {{
        {
            // TODO: get proper Wire from type of $b.
            // TODO: set the garbling mode in an external macro somehow.
            // TODO: get the rng from outside.

            // If called by the garbler, encode and send the garbled value
            match $p.role {
                Role::Garbler => {
                    let auto_generated_garbling_wire = GarblingWire::<$g, $w>::new(&mut $p.rng);
                    let auto_generated_garbled_value =
                        auto_generated_garbling_wire.clone().encode($c);
                    let auto_generated_ser =
                        bincode::serialize(&auto_generated_garbled_value).unwrap();
                    $p.channel.write_usize(auto_generated_ser.len()).unwrap();
                    $p.channel.write_bytes(&auto_generated_ser).unwrap();
                    $p.channel.flush().unwrap();
                    Some(auto_generated_garbling_wire)
                }
                Role::Evaluator => None,
            }
            // If called by the evaluator
        }
    }};
    ($a:ident <- party $b:expr) => {{
        {
            // If called by the garbler
            $c
            // If called by the evaluator
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mpc_core::{EvaluatingWire, GarblingWire, Gate, Operation, Party, Protocol, Role},
        plain_garbling::{evaluate_plain, garble_u8_gate_plain, to_u8, PlainBit},
        wires::Wire8Bit,
    };
    use scuttlebutt::{AbstractChannel, AesRng, Channel, TrackChannel};
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };

    #[test]
    fn obliv_init() {
        let rng = AesRng::new();
        let (sender, receiver) = UnixStream::pair().unwrap();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let channel = Channel::new(reader, writer);
        let alice = Party { id: 1 };
        let bob = Party { id: 2 };
        let mut protocol = Protocol {
            parties: vec![alice.clone(), bob],
            me: alice,
            role: Role::Garbler,
            channel,
            rng,
        };
        let ret = assign!(a, 3 * 4, protocol, PlainBit, Wire8Bit);
        println!("{:?}", ret);
    }

    #[test]
    fn obliv_add() {
        let a = 1;
        let b = 2;
        let val = obliv!(a + b);
        assert_eq!(val, 3);
    }

    #[test]
    fn obliv_sub() {
        let a = 4;
        let b = 3;
        let val = obliv!(a - b);
        assert_eq!(val, 1);
    }

    //#[test]
    //fn plain_circuit_without_ot() {
    //    // network setup
    //    let (sender, receiver) = UnixStream::pair().unwrap();
    //    let handle = std::thread::spawn(move || {
    //        let mut rng = AesRng::new();
    //        // Garbler
    //        let reader = BufReader::new(sender.try_clone().unwrap());
    //        let writer = BufWriter::new(sender);
    //        let mut channel = TrackChannel::new(reader, writer);
    //        // ------------------ Start of the Garbler
    //        let alice = Party { id: 1 };
    //        let bob = Party { id: 2 };
    //        let _protocol = Protocol {
    //            parties: vec![alice.clone(), bob],
    //            me: alice,
    //            role: Role::Garbler,
    //        };

    //        // assign!(a1 <- party 1, value 10);
    //        let a1 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
    //        let garbled_value_a1 = a1.clone().encode(10);
    //        let ser = bincode::serialize(&garbled_value_a1).unwrap();
    //        channel.write_usize(ser.len()).unwrap();
    //        channel.write_bytes(&ser).unwrap();

    //        // assign!(a2 <- party 1, value 20);
    //        let a2 = GarblingWire::<PlainBit, Wire8Bit>::new(&mut rng);
    //        let garbled_value_a2 = a2.clone().encode(20);
    //        let ser = bincode::serialize(&garbled_value_a2).unwrap();
    //        channel.write_usize(ser.len()).unwrap();
    //        channel.write_bytes(&ser).unwrap();

    //        // obliv!(g = a1 + a2);
    //        let (_, gates) = garble_u8_gate_plain(a1, a2, Operation::AddU8);
    //        let ser = bincode::serialize(&gates).unwrap();
    //        channel.write_usize(ser.len()).unwrap();
    //        channel.write_bytes(&ser).unwrap();

    //        // reveal!(g);
    //        // TODO: send decoding
    //        // TODO: receive the plain value
    //    });

    //    // Evaluator
    //    let reader = BufReader::new(receiver.try_clone().unwrap());
    //    let writer = BufWriter::new(receiver);
    //    let mut channel = TrackChannel::new(reader, writer);
    //    // ------------------ Start of evaluator
    //    let alice = Party { id: 1 };
    //    let bob = Party { id: 2 };
    //    let _protocol = Protocol {
    //        parties: vec![alice, bob.clone()],
    //        me: bob,
    //        role: Role::Evaluator,
    //    };
    //    // assign!(a1 <- party 1);
    //    let size = channel.read_usize().unwrap();
    //    let ser = channel.read_vec(size).unwrap();
    //    let a1: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();

    //    // assign!(a2 <- party 1);
    //    let size = channel.read_usize().unwrap();
    //    let ser = channel.read_vec(size).unwrap();
    //    let a2: EvaluatingWire<PlainBit> = bincode::deserialize(&ser).unwrap();

    //    // obliv!(g = a1 + a2);
    //    let size = channel.read_usize().unwrap();
    //    let ser = channel.read_vec(size).unwrap();
    //    let gates: Vec<Gate<PlainBit, Wire8Bit>> = bincode::deserialize(&ser).unwrap();
    //    let g = evaluate_plain(a1, a2, Operation::AddU8, gates);

    //    // reveal!(g);
    //    // TODO receive decoding and decode.
    //    let plain_g = to_u8(&g);
    //    assert_eq!(plain_g, 30);

    //    handle.join().unwrap();
    //}
}
