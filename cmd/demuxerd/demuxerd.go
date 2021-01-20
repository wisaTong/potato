package main

import (
	"github.com/wisatong/potato/demuxerd"
)

func main() {
	d := demuxerd.Demuxerd{}
	d.ListenRequest()
}
