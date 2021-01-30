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
	StaticDir string
	Assets    map[string]StaticAsset
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
	path := fmt.Sprintf("%s/%s", d.StaticDir, filename)
	info, err := os.Stat(path)
	if err != nil {
		return err
	}
	modTime := info.ModTime()

	file, found := d.Assets[filename]
	if found && modTime.Equal(file.ModTime) {
		*reply = file.Data
	} else {
		data, err := ioutil.ReadFile(path)
		if err != nil {
			return err
		}

		d.Assets[filename] = StaticAsset{data, modTime}
		*reply = data
	}
	return nil
}
