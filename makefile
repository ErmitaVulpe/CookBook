.PHONY: make clean docker

make: build/db.sqlite build/cook-book build/.env

build: 
	mkdir build

build/db.sqlite: build
	diesel migration run --database-url build/db.sqlite

build/cook-book: build
	cargo leptos build --release
	cp target/release/cook-book build/
	cp -r target/site/ build/

build/.env: build
	printf "%s\n" \
	"DATABASE_URL=db.sqlite" \
	"CDN_PATH=cdn/" \
	"LEPTOS_SITE_ROOT=site/" \
	"LEPTOS_SITE_ADDR=0.0.0.0:3000" \
	"ADMIN_USERNAME=admin" \
	"ADMIN_PASSWORD=admin" \
	> build/.env

# Example use:
# PUBLIC_URL="https://example.com/example" make docker
docker:
	docker build -t cook-book:latest $(if $(PUBLIC_URL),--build-arg PUBLIC_URL=$(PUBLIC_URL)) .

clean:
	cargo clean
	rm -r build/
