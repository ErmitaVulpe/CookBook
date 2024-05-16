.PHONY: make clean

make: build/db.sqlite build/cook-book build/.env

build: 
	mkdir build

build/db.sqlite: build
	diesel migration run
	cp db.sqlite build/

build/cook-book: build
	cargo leptos build --release
	cp target/release/cook-book build/
	cp -r target/site/ build/

build/.env: build
	printf "%s\n" \
	"DATABASE_URL=db.sqlite" \
	"CDN_PATH=cdn/" \
	"LEPTOS_SITE_ROOT=site/" \
	"ADMIN_USERNAME=admin" \
	"ADMIN_PASSWORD=admin" \
	>> build/.env

clean:
	cargo clean
	rm -r build/
