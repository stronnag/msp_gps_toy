
build:
	cargo build --release

install:
	cargo install --path . --root $$HOME/.local/ --force

debug:
	cargo build

windows:
	cargo build --target x86_64-pc-windows-gnu --release

static:
	cargo build --target=x86_64-unknown-linux-musl --release

clean:
	cargo clean
