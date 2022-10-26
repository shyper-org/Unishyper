# Unishyper Demo Version

Unishyper is a research unikernel, targeting a scalable and predictable runtime for embedded devices. Unikernel means, you bundle your application directly with the kernel library, so that it can run without any installed operating system.

## Boards and Platforms

Unishyper  now supports following platforms.

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
6. Terminal support.
## Toolchains

1. Nightly Rust (`nightly-2022-09-14` tested)
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

## Terminal Support

The unishyper support command line interaction through a simple terminal implementation with "terminal" feature enabled. The simple terminal supports basic input and output, and some basic file operations, including ls, mkdir, cat, e.g.

You can input "help" in terminal for more information.

```bash
☻ SHELL➜ help
This is unishyper,
a research unikernel targeting a scalable and predictable runtime for embedded devices.
List of classes of commands:

cat [FILE]      -- Concatenate files and print on the standard output, "fs" feature is required.
free            -- Dump memory usage info.
kill [TID]      -- Kill target thread according to TID, you can use "ps" command to check running threads.
ls [DIR]        -- List information about the FILEs (the current directory by default), "fs" feature is required.
mkdir [DIR]     -- Create the DIRECTORY, if they do not already exist, "fs" feature is required.
ps              -- Report a snapshot of the current threads, you can use "run [TID]" to wake the ready ones.
run [TID]       -- Run target thread according to TID, you can use "ps" command to check available threads.
help            -- Print this message.

☻ SHELL➜

```

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

Add the feature `tcp` in the `Cargo.toml`. Unishyper use the network stack [smoltcp](https://github.com/smoltcp-rs/smoltcp) to offer TCP/UDP communication.
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

By default, Unishyper's network interface uses `10.0.0.2` as IP address, `10.0.0.1`
for the gateway and `255.255.255.0` as network mask.

Currently, Unishyper does only support network interfaces through [virtio-net](https://www.redhat.com/en/blog/introduction-virtio-networking-and-vhost-net).
To use it, you have to start Unishyper in Qemu with following parameters:

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

 ## FS Support

 Currently Unishyper provides two types of file system, including Fatfs based on virtio and Unilib-fs based on Unilib API running on rust hypervisor. 

### Fatfs

To enable Fatfs, we need to enable the features `fs` and `fat`, and prefare a disk.img for it. The following command make a disk image from zero, or you can just run `make disk` to prepare a disk img.
```Makefile
dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
mkfs.fat -F 32 disk.img
```

Unishyper use the [fatfs](https://github.com/rafalh/rust-fatfs) to offer fatfs support.

```toml
[dependencies.fatfs]
version = "0.4"
git = "https://github.com/rafalh/rust-fatfs"
default-features = false
features = ["lfn", "alloc", "unicode", "log_level_trace"]
```
Currently, Unishyper does only support blk operation through [virtio-blk](https://www.qemu.org/2021/01/19/virtio-blk-scsi-configuration/).
To use it, you have to start Unishyper in Qemu with following parameters:

```Makefile
QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
```
You may see Makefile for details.

### Unilib-FS

Unishyper's Unilib-FS is based on the support of rust-hypervisor. Unishyper's fs operation request is passed to the MVM by a HVC request from Unishyper and a IPI request inside hypervisor. The MVM will perform the file operation on its user daemon process and send the result back to Unishyper.

Through the thought of unilib, Unishyper can get rid of the huge code size of file system and block device driver, replaced with just a few lines of HVC calls.

The unilib is still under developing.