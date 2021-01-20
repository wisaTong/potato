package publisherd

import "io/ioutil"

// Publisherd structure
type Publisherd struct {
	StaticDir string
}

// GetStaticFile to get file in asset directory
func (d *Publisherd) GetStaticFile(filename string, reply *[]byte) error {
	data, err := ioutil.ReadFile(d.StaticDir + "/" + filename)
	if err != nil {
		return err
	}

	*reply = data

	return nil
}
