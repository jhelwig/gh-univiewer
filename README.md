# gh-univiewer

Rust app for generating (and visualizing) various metrics & stats about GitHub
repositories.

The app is intended to be run on a Raspberry PI with the
[Pimoroni Unicorn HAT HD](https://shop.pimoroni.com/products/unicorn-hat-hd). In
order to help speed up development of the various metrics & stats, the project
can be built without the "`unicorn`" feature to only display to stdout, instead
of also displaying to the Unicorn HAT HD.

## Building

### Building (non-Raspberry PI)

```bash
cargo build
```

### Building (Raspberry PI native)

```bash
cargo build --features=unicorn
```

### Building (Raspberry PI cross-compile)

This is the most involved setup as it requires having an appropriate
cross-compilation environment set up, including having access to a
cross-compiled OpenSSL.

It's helpful to set several environment variables to help `cargo` and friends
find where everything is.

The following example assumes a [`crosstool-ng`]() setup that has been installed
to `/Volumes/rpi-xtools/${CT_TARGET}`, and a cross-compiled OpenSSL that has
been installed to `/Volumes/rpi-xtools/pi-openssl`.

```bash
export CC_armv7_unknown_linux_gnueabihf=/Volumes/rpi-xtools/armv7-rpi2-linux-gnueabihf/bin/armv7-rpi2-linux-gnueabihf-cc
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/Volumes/rpi-xtools/pi-openssl/lib
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_DIR=/Volumes/rpi-xtools/pi-openssl/openssl
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/Volumes/rpi-xtools/pi-openssl/include
```

Cargo will also need to be told what the name of the ARM linker is. For example,
in `~/.cargo/config`:

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "armv7-rpi2-linux-gnueabihf-gcc"
```

`gh-univiewer` can then be built with:

```bash
cargo build --target=armv7-unknown-linux-gnueabihf --features=unicorn
```

## Copyright and license

Copyright (c) 2017 Jacob Helwig. Released under the [BSD license](LICENSE).
