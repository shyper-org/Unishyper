# Rust-ShyperOS Demo Version

Rust-ShyperOS is a research unikernel, targeting a scalable and predictable runtime for embedded devices. Unikernel means, you bundle your application directly with the kernel library, so that it can run without any installed operating system.

## Boards and Platforms

Rust-ShyperOS  now supports following platforms.

| MACHINE | ARCH                    | Description                             |
|---------|-------------------------|-----------------------------------------|
| virt    | **aarch64** (AArch64)   | QEMU (qemu-system-aarch64) |
| shyper     | **aarch64** (AArch64)   |  Type-1 Hypervisor   |
| tx2     | **aarch64** (AArch64)   | NVIDIA TX2                              |


## Features

1. Singal address space, singal priveledge level.
2. SMP support, multi-thread support.
3. Semaphores and synchronization mechanisms support.
4. Network stack and file system support.
5. Virtio drivers (virtio-net, virtio-blk).
## Toolchains

1. Nightly Rust (`nightly-2022-05-04` tested)
2. `rust-src` component (use `make dependencies` to install)
3. QEMU emulator version 5.0.0, `qemu-system-aarch64`.
4. Linaro GCC 7.5-2019.12

## Applications

The examples directory contains example demos.

use this lines to build and emulate:

```
# for examples/net_demo
make net_server | net_client
# for examples/user
make user
# for examples/fs
make fs
```
use this lines to build and debug:

```
# for examples/net_demo
make net_server_debug | net_client_debug
# for examples/user
make user_debug
# for examples/fs
make fs_debug
```

## Booting

> wait for implementation...

## Network Support
To enable an ethernet device, we have to setup a tap device on the
host system. For instance, the following command establish the tap device
`tap0` on Linux,  or you can just run `make tap_setup` to set up tap device for network support.

```bash
sudo ip tuntap add tap0 mode tap
sudo ip addr add 10.0.0.1/24 broadcast 10.0.0.255 dev tap0
sudo ip link set dev tap0 up
sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap0/proxy_arp'
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

By default, Rust-ShyperOS's network interface uses `10.0.0.2` as IP address, `10.0.0.1`
for the gateway and `255.255.255.0` as network mask.

Currently, Rust-ShyperOS does only support network interfaces through [virtio-net](https://www.redhat.com/en/blog/introduction-virtio-networking-and-vhost-net).
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
 * run `sudo tcpdump -i tap0 -vvv -nn -e -p net 10.0.0 and not proto \\udp` to trace useful network packets
 * run `make net_emu`
 * run `/examples/net_test/client `

 ## Fatfs Support

 Rust-ShyperOS's Fatfs support based on virtio-blk. To enable fatfs, we need to prefare a disk.img for it. The following command make a disk image from zero, or you can just run `make disk` to prepare a disk img.
```Makefile
dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
mkfs.fat -F 32 disk.img
```

Add the feature `fs` in the Cargo.toml. Rust-ShyperOS use the [fatfs](https://github.com/rafalh/rust-fatfs) to offer fatfs support.

```toml
[dependencies.fatfs]
version = "0.4"
git = "https://github.com/rafalh/rust-fatfs"
default-features = false
features = ["lfn", "alloc", "unicode", "log_level_trace"]
```
Currently, Rust-ShyperOS does only support blk operation through [virtio-blk](https://www.qemu.org/2021/01/19/virtio-blk-scsi-configuration/).
To use it, you have to start Rust-ShyperOS in Qemu with following parameters:

```Makefile
QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
```
 You may see Makefile for details.
