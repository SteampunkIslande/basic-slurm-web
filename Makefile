distrib:
	cargo build --release
	mkdir -p distrib
	mv target/x86_64-unknown-linux-musl/release/slurm-web distrib
	cp Rocket.toml distrib
	cp -r static distrib
	zip -r distrib.zip distrib
	rm -R distrib

