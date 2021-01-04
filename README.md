# rsblk
List block devices on FreeBSD

This tool is (aspirationally) similar to the
[lsblk](https://manpages.debian.org/unstable/util-linux/lsblk.8.en.html)
utility from [util-linux](https://github.com/karelzak/util-linux).  However, it
is still a work in progress.  It is written in the Rust programming language.

## Example

```
$ rsblk
NAME      FSTYPE        LABEL                UUID
ada0
└─ada0p1  freebsd-ufs   freebsd-obj-fs       aaaaaaaa-29d3-11e9-8cc6-7085c25400ea
md0
└─md0p1   freebsd-ufs                        bbbbbbbb-4c3c-11eb-b311-7085c25400ea
nvd0
├─nvd0p1  efi           freebsd-efi          cccccccc-8c5e-11e7-a9ab-7085c25400ea
├─nvd0p2  freebsd-ufs   freebsd-root         dddddddd-8c5e-11e7-a9ab-7085c25400ea
├─nvd0p3  freebsd-swap  freebsd-swap         eeeeeeee-8c5e-11e7-a9ab-7085c25400ea
└─nvd0p4  freebsd-boot  freebsd-boot         ffffffff-f1fa-11e8-9b63-7085c25400ea
nvd1
├─nvd1p1  freebsd-swap  freebsd-swap-660p    00000000-79e7-11e9-b158-7085c25400ea
└─nvd1p2  freebsd-ufs   freebsd-obj-660p-fs  11111111-79e7-11e9-b158-7085c25400ea
```

By default (and very much subject to change), `rsblk` prints four columns: the
NAME of a device; and for partitions, its filesystem type, partition label, and
UUID.
