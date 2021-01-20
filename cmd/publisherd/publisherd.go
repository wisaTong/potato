package main

import (
	"os"

	"github.com/wisatong/potato/publisherd"
)

func main() {
	d := publisherd.Publisherd{StaticDir: os.Getenv("STATIC_DIR")}
	d.Start(7525)
}
