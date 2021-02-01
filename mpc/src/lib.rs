//use curve25519_dalek::{
//constants::RISTRETTO_BASEPOINT_TABLE, ristretto::RistrettoPoint, scalar::Scalar,
//};
//use rand::{CryptoRng, Rng};
////use scuttlebutt::Block;

//pub struct Sender {
//y: Scalar,
//s: RistrettoPoint,
//}

//enum Error {}

//impl Sender {
//fn init<RNG: CryptoRng + Rng>(mut rng: &mut RNG) -> Result<Self, Error> {
//let y = Scalar::random(&mut rng);
//let s = &y * &RISTRETTO_BASEPOINT_TABLE;
//Ok(Self { y, s })
//}
//}

trait Gate {
    fn inputs(&self) -> Vec<usize>;
}

struct AndGate {
    left: usize,
    right: usize,
}

impl Gate for AndGate {
    fn inputs(&self) -> Vec<usize> {
        vec![self.left, self.right]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_and() {
        let c = AndGate { left: 2, right: 3 };
        assert_eq!(c.inputs(), vec![2, 3]);
    }
}
