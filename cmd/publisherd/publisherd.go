package main

import (
	"log"
	"net"
	"net/rpc"
	"os"

	"github.com/wisatong/potato/publisherd"
)

func main() {
	address, err := net.ResolveTCPAddr("tcp", "0.0.0.0:12345")
	if err != nil {
		log.Fatal(err)
	}
	inbound, err := net.ListenTCP("tcp", address)
	if err != nil {
		log.Fatal(err)
	}

	d := publisherd.Publisherd{StaticDir: os.Getenv("ASSET_DIR")}
	rpc.Register(&d)
	rpc.Accept(inbound)
}
