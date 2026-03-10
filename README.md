# Greek Computer

A simulation of a physical puzzle I got from a friend. The original puzzle is
layers of circles. Each layer has numbers and possibly holes exposing underlying
layers. The goal is to rotate the layers so that each column sums to 42.

https://projectgeniusinc.com/products/grecian-computer

Play with the simulation: https://www.omustardo.com/share/greek_computer/

## Build and Run

### Native

Run:
```shell
make run
```

Build (release):
```shell
make build-release-native

# The generated binary is:
./target/release/app
```

### Web

Run:
```shell
make web

# View it at: http://localhost:50051/index.html#dev
# "#dev" prevents caching.
```

Build:
```shell
make build-release-wasm

# The generated files are in:
./src/crates/app/dist/
# You can serve them locally with:
python3 -m http.server 8000 -d ./src/crates/app/dist/
# Or package them up and put them in a static HTTP server.
```

Building for web will output a `index.html` file, and other files to be served
alongside it. It assumes that it will be served at the root URL of your website.
If you want to use a different URL, modify the `build-release-wasm` section
of the `Makefile`. There is more detail in the comments there.
