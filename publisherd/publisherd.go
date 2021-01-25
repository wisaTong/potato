package publisherd

import (
	"fmt"
	"io/ioutil"
	"log"
	"net"
	"net/rpc"
)

// Publisherd structure
type Publisherd struct {
	StaticDir string
}

// Start starts publisher daemon listening for tcp connection on specified port
func (d Publisherd) Start(port uint16) {
	address := fmt.Sprintf(":%d", port)
	tcpAddress, err := net.ResolveTCPAddr("tcp", address)
	if err != nil {
		log.Fatal(err)
	}
	inbound, err := net.ListenTCP("tcp", tcpAddress)
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("publisherd running. listening on port%s", tcpAddress)
	rpc.Register(&d)
	rpc.Accept(inbound)
}

// GetStaticFile to get file in asset directory
func (d *Publisherd) GetStaticFile(filename string, reply *[]byte) error {
	data, err := ioutil.ReadFile(d.StaticDir + "/" + filename)
	if err != nil {
		return err
	}

	*reply = data
	return nil
}
