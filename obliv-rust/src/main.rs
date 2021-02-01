use ocelot::ot::{ChouOrlandiReceiver, ChouOrlandiSender, Receiver, Sender};
use scuttlebutt::{AesHash, AesRng, Block, TrackChannel};
use std::{
    io::{BufReader, BufWriter},
    os::unix::net::UnixStream,
};
mod semihonest_2pc;

fn rand_block_vec(size: usize) -> Vec<(Block, Block)> {
    (0..size)
        .map(|_| (rand::random::<Block>(), rand::random::<Block>()))
        .collect()
}

fn rand_bool_vec(size: usize) -> Vec<bool> {
    (0..size).map(|_| rand::random::<bool>()).collect()
}

fn test_hash() {
    let key = Block::from([1; 16]);
    let hash = AesHash::new(key);
    let unused_block = Block::from([0; 16]);
    let input_block = Block::from([2; 16]);
    let output = hash.cr_hash(unused_block, input_block);
    println!("key: {}, input: {} output: {}", key, input_block, output);
}

fn test_ot(n: usize) {
    let input_pairs = rand_block_vec(n);
    let choices = rand_bool_vec(n);
    //println!("choices: {:?}\ninputs:{:?}\n", choices, input_pairs);
    let (sender, receiver) = UnixStream::pair().unwrap();
    let handle = std::thread::spawn(move || {
        let mut rng = AesRng::new();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let mut channel = TrackChannel::new(reader, writer);

        let mut co_ot = ChouOrlandiSender::init(&mut channel, &mut rng).unwrap();
        let _ = co_ot.send(&mut channel, &input_pairs, &mut rng).unwrap();
        println!(
            "Sender communication (read): {:.2} Mb",
            channel.kilobits_read() / 1000.0
        );
        println!(
            "Sender communication (write): {:.2} Mb",
            channel.kilobits_written() / 1000.0
        );
    });

    // Receiver
    let mut rng = AesRng::new();
    let reader = BufReader::new(receiver.try_clone().unwrap());
    let writer = BufWriter::new(receiver);
    let mut channel = TrackChannel::new(reader, writer);
    let mut co_ot = ChouOrlandiReceiver::init(&mut channel, &mut rng).unwrap();
    let _received = co_ot.receive(&mut channel, &choices, &mut rng);
    handle.join().unwrap();
    println!(
        "Receiver communication (read): {:.2} Mb",
        channel.kilobits_read() / 1000.0
    );
    println!(
        "Receiver communication (write): {:.2} Mb",
        channel.kilobits_written() / 1000.0
    );
    //println!("Received {:?}", received);
}

macro_rules! calculate {
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Force types to be integers
            println!("{} = {}", stringify!{$e}, val);
        }
    }};
}

fn test_macro() {
    calculate! {
        eval 1 + 2 // hehehe `eval` is _not_ a Rust keyword!
    }

    calculate! {
        eval (1 + 2) * (3 / 4)
    }
}

fn main() {
    test_macro();
    println!("----------------------------------------------------------");
    test_hash();
    println!("----------------------------------------------------------");
    test_ot(10);
    println!("----------------------------------------------------------");
    semihonest_2pc::run();
}
