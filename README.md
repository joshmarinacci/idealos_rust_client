# idealos_rust_client

install rust via [rustup](https://www.rust-lang.org/tools/install), 
then clone the repo, `cd` into `idealos_rust_client`, and run `cargo run`.

if it fails with an openssl error, install libssl-dev first (I had to do this on Raspberry Pi)

```
sudo apt-get install libssl-dev
```


Once it builds you'll need to already have the OS server running for this client to connect to it.
