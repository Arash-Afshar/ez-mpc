# ez-mpc
Making Secure Multi-Party Computation Easy and Fun!


## Roadmap

Goals:
- Heavily tested
- Heavily documented
- Easy to use
- Performant (preferably configurable on what to emphasize: memory, computation, or network)
- Extendable using plugins


Roadmap:

1. Implement Obliv-C in pure Rust
   - Macros to handle "obliv" keyword
   - Macros to handle base circuit generations (start with arithmetic and logical operations on u32 and f32).
   - Construct the underlying circuit on the fly.
   - Implement the debug protocol (focus on the functionality, swappable network layer and swappable backend layer).
   - Implement the Yao protocol.
   - Implement the Malicious Yao protocol.
2. Use Obliv-Rust to implement a console app
   - A Rust app that can use
3. Use Obliv-Rust to implement a console web app
   - A React and React-Native app that can run the millioner's problem
   - Use Rust's WASM capabilities
4. Extend Obliv-Rust with latest works in ORAM
5. Extend Obliv-Rust to work with multiple parties
