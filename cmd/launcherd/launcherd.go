package main

import (
	"github.com/wisatong/potato/launcherd"
)

func main() {
	list := []string{
		"demuxerd",
		"publisherd",
	}
	d := launcherd.Launcherd{List: list}
	d.Start()
}
