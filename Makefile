all: rfc app

.PHONY: rfc

clean:
	cd rfc; make clean
	cargo clean

rfc:
	cd rfc; make

app:
	cargo build --release