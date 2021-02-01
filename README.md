# ez-mpc
Making Secure Multi-Party Computation Easy and Fun!




Learning:

Learn basic Yao (lindel's paper, or some other video)
Learn a simple OT (CO13)

Focus on different garbling schemes:
  - Point and permute, FreeXOR, half-gate
    - Theory
    - How to realize them efficiently in practice
        - AES-NI or not?
        - Random Oracle?


Circuit compilers

OT extensions


Deployment modes
  - Websocket
  - Server socket
  - REST

Base on deployment mode
  - Investigate pipelining


Gaps:
- Converter between different circuit formats
- Run circuit formats plainly to provide correctness guarantees
- texample.net style of placing examples of usages of protocols
- random circuit generator for testing and benchmarking