# Why Slurm-Web

As of today, the cluster I'm working on uses Rocky 9 official version of SLURM (22.05), and this version doesn't provide any web API out of the box.

So I wrote this very basic web app to serve the `squeue` results both in JSON and HTML format.

# How to use

## Run the app

```bash
# Clone the repo
git clone https://github.com/SteampunkIslande/basic-slurm-web.git
cd slurm-web
cargo run --release
```

When running, this app should have access to the `squeue` command, otherwise only mock data will be served.

## Configuration

- By default, port 8080 is used, but you can change it either by setting the `ROCKET_PORT` environment variable, or by editing the `Rocket.toml` file.
- Be sure the current working directory is the root of the project, as the static route is relative to the current working directory.
- By default, the app only binds to `127.0.0.1`.


