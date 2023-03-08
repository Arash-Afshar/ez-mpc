SRC_DIR="$(pwd)/../protos"
GO_DST_DIR="$(pwd)"
GO_DST_PROTO_DIR="${GO_DST_DIR}/core"
mkdir -p "${GO_DST_PROTO_DIR}"

protoc -I="${SRC_DIR}" --go_opt=paths=source_relative --go_out="${GO_DST_PROTO_DIR}" "${SRC_DIR}/types.proto"