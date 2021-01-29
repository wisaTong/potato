package main

import (
	"os"

	"github.com/wisatong/potato/publisherd"
)

func main() {
	d := publisherd.Publisherd{StaticDir: os.Getenv("STATIC_DIR"), StaticInfo: make(map[string]publisherd.StaticAsset)}
	d.Start(7525)
}
