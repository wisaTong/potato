package demuxerd

import (
	"fmt"
	"log"
	"net/http"
	"net/rpc"
)

// Demuxerd A demultiplexer daemon
type Demuxerd struct {
	ServiceURL []string
}

// Start starts a demuxer daemon
func (d *Demuxerd) Start(port uint16) {
	address := fmt.Sprintf(":%d", port)
	http.HandleFunc("/", d.rpcHandler)

	log.Printf("demuxerd running. listening on http port%s", address)
	http.ListenAndServe(address, nil)
}

func (d *Demuxerd) rpcHandler(w http.ResponseWriter, r *http.Request) {
	content, err := d.rpcClient(r.URL.Path[1:])
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte("500 - Something bad happened!"))
	}
	fmt.Fprint(w, string(content[:]))
}

// send some endpoint and grab static asset from publisherd
func (d *Demuxerd) rpcClient(args string) ([]byte, error) {
	var reply []byte
	client, _ := rpc.Dial("tcp", ":7525")
	//Call the publisherd method
	err := client.Call("Publisherd.GetStaticFile", args, &reply)
	client.Close()
	if err != nil {
		log.Printf("%s", err)
	}
	return reply, err
}
