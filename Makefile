distrib:
	cargo build --release
	zip -r distrib.zip target/x86_64-unknown-linux-musl/release/slurm-web Rocket.toml static

