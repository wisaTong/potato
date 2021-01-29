package main

import (
	"os"

	"github.com/wisatong/potato/publisherd"
)

func main() {
	d := publisherd.Publisherd{StaticDir: os.Getenv("STATIC_DIR"), Map: make(map[string]publisherd.FileDetails)}
	d.Start(7525)
}
