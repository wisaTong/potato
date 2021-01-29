package publisherd

import (
	"fmt"
	"io/ioutil"
	"log"
	"net"
	"net/rpc"
	"os"
	"time"
)

// Publisherd structure
type Publisherd struct {
	StaticDir  string
	StaticInfo map[string]StaticAsset
}

// StaticAsset structure
type StaticAsset struct {
	Data    []byte
	ModTime time.Time
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
	file, found := d.StaticInfo[filename]
	info, _ := os.Stat(d.StaticDir + "/" + filename)
	modTime := info.ModTime()

	if found && modTime == file.ModTime {
		*reply = file.Data
	} else {
		data, err := ioutil.ReadFile(d.StaticDir + "/" + filename)
		if err != nil {
			return err
		}

		d.StaticInfo[filename] = StaticAsset{data, modTime}
		*reply = data
	}
	return nil
}
