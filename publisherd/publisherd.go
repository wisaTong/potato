package publisherd

import (
	"os"
)

func GetStaticFile(filename string) (fp *os.File) {
	dir := os.Getenv("ASSET_DIR")
	fp, err := os.Open(dir + "/" + filename)
	if err != nil {
		panic(err)
	}

	defer func() {
		if err := fp.Close(); err != nil {
			panic(err)
		}
	}()

	return fp
}
