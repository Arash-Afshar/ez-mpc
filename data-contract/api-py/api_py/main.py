import api_py.core.types_pb2 as types_pb2

s = types_pb2.Scalar()
s.data.append(b"a")

print(f"Data: {s.data}")
