use security_mode::security_mode;

fn assign(_inp: u8, sec: &str) -> bool {
    sec == "AAA"
}

#[security_mode(AAA)]
fn protocol() -> bool {
    let a = assign(1);
    assign(2);
    assign(3)
}

fn main() {
    protocol();
}
