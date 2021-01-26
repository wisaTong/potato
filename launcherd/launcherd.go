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
	List []string
}

// Start starts a launcherd
func (d *Launcherd) Start() {
	// [ ] prepare lib (rootfs)
	// [ ] fork and chroot each service
	for _, service := range d.List {
		err := d.launch(service)
		if err != nil {
			log.Printf("Failed to start %s", service)
		} else {
			log.Fatalf("Succesfully start %s", service)
		}
	}
	// NEAR FUTURE
	// [ ] set namespace and cgroups
	// [ ] monitor health status of running service
}

func (d *Launcherd) launch(serviceName string) error {
	// StdOut Stdin StdErr ???
	chrPath := fmt.Sprintf("./%schroot", serviceName)
	err := syscall.Chroot(chrPath)
	execPath := fmt.Sprintf("/%s", serviceName)
	cmd := exec.Command(execPath)
	if err != nil {
		return err
	}
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	err = cmd.Start()
	if err != nil {
		return err
	}
	return nil
}
