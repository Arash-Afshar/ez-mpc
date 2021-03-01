use crate::mpc_core::GarblingMode;
use rand_core::{CryptoRng, RngCore};
use scuttlebutt::Block;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GarbledBit(pub Block);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpc_core::{EvaluatingWire, GarblingWire, Gate, Wire8Bit};
    use rand::{rngs::StdRng, SeedableRng};
    use scuttlebutt::{AbstractChannel, Block, TrackChannel};
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };
    const SEED: [u8; 32] = [42u8; 32];

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
}
