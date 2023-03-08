use std::env;
use std::io::Result;

fn main() -> Result<()> {
    env::set_var("OUT_DIR", "./src");
    prost_build::compile_protos(&["../protos/types.proto"], &["../protos/"])?;
    Ok(())
}
