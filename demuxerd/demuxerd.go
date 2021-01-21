package demuxerd

import (
	"fmt"
	"log"
	"net/http"
	"net/rpc"
)

// Demuxerd A demultiplexer daemon
type Demuxerd struct {
	ServiceURL string
}

// Start starts a demuxer daemon
func (d *Demuxerd) Start(port uint16) {
	address := fmt.Sprintf(":%d", port)
	http.HandleFunc("/", d.rpcHandler)

	log.Printf("demuxerd running. listening on http port%s", address)
	http.ListenAndServe(address, nil)
}

func (d *Demuxerd) rpcHandler(w http.ResponseWriter, r *http.Request) {
	content := d.rpcClient(r.URL.Path[1:])
	fmt.Fprint(w, string(content[:]))
}

// send some endpoint and grab static asset from publisherd
func (d *Demuxerd) rpcClient(args string) (reply []byte) {
	client, _ := rpc.Dial("tcp", ":7525")
	//Call the publisherd method
	err := client.Call("Publisherd.GetStaticFile", args, &reply)
	if err != nil {
		log.Fatal(err)
	}
	return
}

func BytesToString(data []byte) string {
	return string(data[:])
}
