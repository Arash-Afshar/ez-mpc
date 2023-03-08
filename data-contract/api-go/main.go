package main

import (
	"fmt"

	"github.com/Arash-Afshar/ez-mpc/data-contract/api-go/core"
)

func main() {
	s := core.Scalar{}
	s.Data = append(s.Data, []byte("a"))

	fmt.Printf("Data: %s\n", s.Data)
}
