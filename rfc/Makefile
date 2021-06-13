all: target/main.txt target/main.html target/main.pdf

clean:
	rm -r target

target/main.xml: src/main.md
	mkdir -p target
	mmark src/main.md > target/main.xml

target/main.txt: target/main.xml
	xml2rfc --text --v3 target/main.xml

target/main.html: target/main.xml
	xml2rfc --html --v3 target/main.xml

target/main.pdf: target/main.xml
	xml2rfc --pdf --v3 target/main.xml