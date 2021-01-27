package publisherd

import (
	"fmt"
	"io/ioutil"
	"log"
	"net"
	"net/rpc"
	"os"
)

// Publisherd structure
type Publisherd struct {
	StaticDir string
	Map       map[string][]byte
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
	_, found := d.Map[filename]
	fmt.Println(found)
	info, _ := os.Stat(d.StaticDir + "/" + filename)

	if found && int64(len(d.Map[filename])) == info.Size() {
		*reply = d.Map[filename]

	} else {
		data, err := ioutil.ReadFile(d.StaticDir + "/" + filename)
		if err != nil {
			return err
		}
		d.Map[filename] = data
		*reply = data
	}
	return nil
}
