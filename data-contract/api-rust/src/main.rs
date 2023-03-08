use api_rust::api::Scalar;

fn main() {
    let s = Scalar {
        data: vec![vec![b'a']],
    };

    println!("Data: {:?}", s.data);
}
