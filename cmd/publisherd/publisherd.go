package main

import (
	"fmt"

	"github.com/wisatong/potato/publisherd"
)

func main() {

	fmt.Println(publisherd.GetStaticFile("index.html").Name())
}
