package launcherd

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"syscall"
)

// Launcherd a launcher daemon for starting webservices
type Launcherd struct {
	//array of program wanna run string
	List []string
}

// Start starts a launcherd
func (d *Launcherd) Start() {
	// [ ] prepare lib (rootfs)
	// [ ] fork and chroot each service

	for _, service := range d.List {
		err := d.launch(service)
		if err != nil {
			log.Fatal(err)
		}
	}

	// NEAR FUTURE
	// [ ] set namespace and cgroups
	// [ ] monitor health status of running service
}

func (d *Launcherd) launch(serviceName string) error {
	// StdOut Stdin StdErr ???
	path := fmt.Sprintf("./%schroot", serviceName)
	err := syscall.Chroot(path)
	execPath := fmt.Sprintf("./%s/%s", path, serviceName)
	cmd := exec.Command(execPath)
	if err != nil {
		return err
	}
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	err = cmd.Run()
	if err != nil {
		return err
	}
	return nil
}

// func (d *Launcherd) CopyDependencies(serviceName string) error {
// 	c := fmt.Sprintf(`list="$(ldd ./%schroot/%s | egrep -o '/lib.*\.[0-9]')"`, serviceName, serviceName)
// 	cmd := exec.Command(c)
// 	err := cmd.Run()
// 	if err != nil {
// 		return err
// 	}
// 	c = fmt.Sprintf(`for i in $list; do cp -v --parents "$i" "./%schroot"; done`, serviceName)
// 	cmd = exec.Command(c)
// 	err = cmd.Run()
// 	if err != nil {
// 		return err
// 	}
// 	return nil
// }
