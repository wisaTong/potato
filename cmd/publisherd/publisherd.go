package main

import (
	"os"

	"github.com/wisatong/potato/publisherd"
)

func main() {
	d := publisherd.Publisherd{
		StaticDir: os.Getenv("STATIC_DIR"),
		Assets:    make(map[string]publisherd.StaticAsset),
	}
	d.Start(7525)
}
