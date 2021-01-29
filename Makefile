PROGRAMS = demuxerd publisherd launcherd
.PHONY: clean build demuxerd publisherd launcherd

build: demuxerd publisherd launcherd

demuxerd:
	go build -o build/demuxerd cmd/demuxerd/demuxerd.go

publisherd:
	go build -o build/publisherd cmd/publisherd/publisherd.go

launcherd:
	go build -o build/launcherd cmd/launcherd/launcherd.go

clean:
	rm -rf build

install: build
	cp build/* /usr/bin

uninstall:
	@for prog in $(PROGRAMS); do \
		rm -v /usr/bin/$$prog; \
	done

