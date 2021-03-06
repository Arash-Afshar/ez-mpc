use security_mode::security_mode;

fn assign(sec: String) -> bool {
    sec == "AAA"
}

#[security_mode(AAA)]
fn protocol() -> bool {
    assign()
}

fn main() {
    assert!(protocol());
}
