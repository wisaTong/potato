package demuxerd

import (
	"fmt"
	"log"
	"net/http"
	"net/rpc"
)

type Demuxerd struct {
	ServiceUrl string
}

func (d *Demuxerd) ListenRequest() {
	http.HandleFunc("/", d.rpcHandler)
	http.ListenAndServe(":8080", nil)
}

func (d *Demuxerd) rpcHandler(w http.ResponseWriter, r *http.Request) {
	content := d.rpcClient(r.URL.Path[1:])
	fmt.Fprint(w, BytesToString(content))
}

// send some endpoint and grab static asset from publisherd
func (d *Demuxerd) rpcClient(args string) (reply []byte) {
	client, _ := rpc.Dial("tcp", ":12345")
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
