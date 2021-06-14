# COVID Vaccination Poll

This application polls COVID vaccination registration websites
for free appointments. The structure is modular. Different
websites can be polled. The application supports several
backends to deliver notifications.

## Run

### Cargo

Install the toolchain:

```shell
rustup target add x86_64-unknown-linux-gnu
```

Build the application:

```shell
cargo build --release
```
