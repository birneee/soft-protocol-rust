all: rfc app

.PHONY: rfc

clean:
	cd rfc; make clean
	cargo clean

rfc:
	cd rfc; make

public:
	mkdir public && cd public
	wget https://unsplash.com/photos/nR2GyL1vEPs/download?force=true\&w=640 -O test.jpg
	echo "Test data available in public/test.jpg"
	cd ..

app:
	cargo build --release

test:
	cargo test
