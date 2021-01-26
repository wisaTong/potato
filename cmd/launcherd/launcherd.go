package main

import (
	"github.com/wisatong/potato/launcherd"
)

func main() {
	services := []string{
		"demuxerd",
		"publisherd",
	}
	d := launcherd.Launcherd{Services: services}
	d.Start()
}
