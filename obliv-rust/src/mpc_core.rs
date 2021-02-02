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
    AddU32,
    MulU32,
}

//pub struct CircutOperation {
//    pub input_1: WireName,
//    pub input_2: WireName,
//    pub output: WireName,
//    pub op: Operation,
//}

pub struct Key([u8; 32]);

pub enum Bit {
    Plain(bool),
    Garbled(Key),
}

pub struct GarblingWire {
    pub zero_bits: Vec<Bit>,
    pub one_bits: Vec<Bit>,
}

pub struct EvaluatingWire {
    pub bits: Vec<Bit>,
}

//pub struct WireName(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_circuit() {
        let alice = Party { id: 1 };
        let bob = Party { id: 1 };

        let protocol = Protocol {
            parties: vec![alice.clone(), bob],
            me: alice,
            role: Role::Garbler,
        };

        // alice's input: a1, a2
        // bob's   input: b1, b2
        // function: ( a1 + b1 ) * (a2 + b2)

        // obliv!(c = a + b) -> based on the role, do the garbling or evaluating
        // reveal!(c)
        // assign!(a <- party 1, value)

        let a1 = GarblingWire { bits: vec![] };
        let a2 = GarblingWire { bits: vec![] };
        let b1 = GarblingWire { bits: vec![] };
        let b2 = GarblingWire { bits: vec![] };

        // assign!(a1 <- party 1, 10);
        // assign!(a2 <- party 1, 20);
        // assign!(b1 <- party 2, 30);
        // assign!(b2 <- party 2, 40);

        // obliv!(c = a1 + b1) as garbler
        let (c, gates) = garble(a1, b1, Operation::AddU32);
        send(gates);

        // obliv!(c = a1 + b1) as evaluator
        let gates = receive();
        let c = evaluate(a1, b1, gates);

        // obliv!(d = a2 + b2);
        // obliv!(e = c + d);

        // reveal!(e) as garbler
        send(e.mapping);
        let plain_e = receive();

        // reveal!(e) as evaluator
        let mapping = receive();
        let plain_e = decode(e, mapping);
        send(plain_e);
    }
}
