package main

import (
	"math/big"
	"os"

	core "github.com/Arash-Afshar/ez-mpc/data-contract/api-go/core"
	"google.golang.org/protobuf/proto"
)

func main() {
	n := new(big.Int)
	n, ok := n.SetString("123400000000000000000000000000050000000000000000000000000006789", 10)
	if !ok {
		panic("SetString: error")
	}
	mode := "read" // "read" or "write"
	path := "../protos/serialized-go.bin"

	if mode == "write" {
		s := core.Scalar{}
		s.Data = n.Bytes()
		out, err := proto.Marshal(&s)
		if err != nil {
			panic(err)
		}
		err = os.WriteFile(path, out, 0644)
		if err != nil {
			panic(err)
		}
	} else {
		inp, err := os.ReadFile(path)
		if err != nil {
			panic(err)
		}
		s := core.Scalar{}
		err = proto.Unmarshal(inp, &s)
		if err != nil {
			panic(err)
		}
		m := new(big.Int)
		m = m.SetBytes(s.Data)
		if m.Cmp(n) != 0 {
			panic("Not equal!")
		}
	}

}
