import api_py.core.types_pb2 as types_pb2

mode = "read"  # "read" or "write"
path = "../protos/serialized-py.bin"

n = int("123400000000000000000000000000050000000000000000000000000006789")

if mode == "write":
    s = types_pb2.Scalar()
    s.data = n.to_bytes(32, "big")

    with open(path, "wb") as f:
        f.write(s.SerializeToString())
else:
    with open(path, "rb") as f:
        s = types_pb2.Scalar()
        s.ParseFromString(f.read())

        m = int.from_bytes(s.data, "big")

        if m != n:
            print(f"incorrect data: want {n}, got {m}")
            exit(1)
