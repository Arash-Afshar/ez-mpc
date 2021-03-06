use security_mode::security_mode;

fn assign(_inp: u8, sec: &str) -> bool {
    sec == "AAA"
}

#[security_mode(AAA)]
fn protocol() {
    assign(1);
}

fn main() {
    protocol()
}
