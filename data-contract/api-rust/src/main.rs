use api_rust::api::Scalar;
use num::bigint::{BigInt, Sign};
use prost::{self, Message};
use std::fs::File;
use std::io::prelude::*;
use std::{fs, str::FromStr};

fn main() {
    let mode = "read"; // "read" or "write"
    let path = "../protos/serialized-rs.bin";

    let n = BigInt::from_str("123400000000000000000000000000050000000000000000000000000006789")
        .expect("should always parse");

    if mode == "write" {
        // TODO add the sign
        let (sign, data) = n.to_bytes_be();
        let s = Scalar { data };

        let mut out: Vec<u8> = vec![];
        s.encode(&mut out).expect("marshal");
        let mut file = File::create(path).expect("create the bin file");
        file.write_all(&out).expect("write the content");
    } else {
        let contents = fs::read(path).expect("Should have been able to read the file");
        let s = Scalar::decode(&contents[..]).expect("unmarshal");
        let m = BigInt::from_bytes_be(Sign::Plus, &s.data);
        if m != n {
            panic!("want {:?}, got {:?}", n, m);
        }
    }
}
