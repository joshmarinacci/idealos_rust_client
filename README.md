# idealos_rust_client

install rust via [rustup](https://www.rust-lang.org/tools/install), 
then clone the repo, `cd` into `idealos_rust_client`, and run `cargo run`.

if it fails with an openssl error, install libssl-dev first (I had to do this on Raspberry Pi)

```
sudo apt-get install libssl-dev
```

the same with SDL and it's many parts

```
sudo apt-get update
sudo apt-get install libsdl2-2.0-0 libsdl2-dev libsdl2-gfx-dev libsdl2-image-dev libsdl2-ttf-dev
```
this will pull in a *lot* of stuff.


Once it builds you'll need to already have the OS server running for this client to connect to it.
