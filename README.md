# Rust-ShyperOS Demo Version

Rust-ShyperOS is a research unikernel, targeting a scalable and predictable runtime for embedded devices. Unikernel means, you bundle your application directly with the kernel library, so that it can run without any installed operating system.

## Boards and Platforms

Rust-ShyperOS  now supports following platforms.

| MACHINE | ARCH                    | Description                             |
|---------|-------------------------|-----------------------------------------|
| virt    | **aarch64** (AArch64)   | QEMU virt machine (qemu-system-aarch64) |
| tx2     | **aarch64** (AArch64)   | NVIDIA TX2, wait for implementation     |

## Toolchains

1. Nightly Rust (`nightly-2021-06-15-x86_64-unknown-linux-gnu` tested)
2. `rust-src` component (use `make dependencies` to install)
3. QEMU emulator version 5.0.0, `qemu-system-aarch64`.
4. Linaro GCC 7.5-2019.12

## Applications

The examples directory contains example demos.

use this lines to build and emulate:

```
# for examples/net_demo
make net_emu
# for examples/user
make user_emu
```
use this lines to build and debug:

```
# for examples/net_demo
make net_debug
# for examples/user
make user_debug
```

## Booting

> wait for implementation...

## Network Support
To enable an ethernet device, we have to setup a tap device on the
host system. For instance, the following command establish the tap device
`tap0` on Linux:

```bash
sudo ip tuntap add tap10 mode tap
sudo ip addr add 10.0.5.1/24 broadcast 10.0.5.255 dev tap10
sudo ip link set dev tap10 up
sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap10/proxy_arp'
```

Add the feature `tcp` in the `Cargo.toml`. Rust-ShyperOS use the network stack [smoltcp](https://github.com/smoltcp-rs/smoltcp) to offer TCP/UDP communication.
```toml
# Cargo.toml

[dependencies.smoltcp]
version = "0.7"
optional = true
default-features = false
features = [
    "alloc",
    "async",
    "ethernet",
    "proto-ipv4",
    "proto-ipv6",
    "socket-tcp",
    # "socket-udp",
    # "std",
    # Enable for increased output
    # "log",
    # "verbose",
]
```

Per default, Rust-ShyperOS's network interface uses `10.0.5.3` as IP address, `10.0.5.1`
for the gateway and `255.255.255.0` as network mask.

Currently, Rust-ShyperOS does only support network interfaces through [virtio](https://www.redhat.com/en/blog/introduction-virtio-networking-and-vhost-net).
To use it, you have to start Rust-ShyperOS in Qemu with following parameters:

```Makefile
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
```
 You may see Makefile for details.

 You can use **ping** or **traceroute** tools to check if network stacks works.

 To run network test, you may follow these steps:

 * use gcc to compile socket_client.c in examples/net_test dir
   * `gcc examples/net_test/socket_client.c -o examples/net_test/client`
 * spawn `netdemo_server ` thread in examples/net_demo/main.c
 * run `sudo tcpdump -i tap10 -vvv -nn -e -p net 10.0.5 and not proto \\udp` to trace useful network packets
 * run `make net_emu`
 * run `/examples/net_test/client `