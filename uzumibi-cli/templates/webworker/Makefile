.PHONY: build copy wasm clean serve

# Default target
wasm: build copy

# Build WASM module
build:
	cargo build --target wasm32-unknown-unknown --release

# Copy WASM to public directory
copy:
	cp target/wasm32-unknown-unknown/release/*.wasm public/app.wasm

# Clean build artifacts
clean:
	cargo clean
	rm -f public/app.wasm

# Start development server
serve:
	python3 -m http.server 8080 -d public
