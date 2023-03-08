SRC_DIR="$(pwd)/../protos"
PY_DST_DIR="$(pwd)"
PY_DST_PROTO_DIR="${PY_DST_DIR}/api_py/core"
mkdir -p "${PY_DST_PROTO_DIR}"
touch "${PY_DST_PROTO_DIR}/__init__.py"

protoc -I="${SRC_DIR}" --python_out="${PY_DST_PROTO_DIR}" --pyi_out="${PY_DST_PROTO_DIR}" "${SRC_DIR}/types.proto"