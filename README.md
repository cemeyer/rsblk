# rsblk
List block devices on FreeBSD

This tool is (aspirationally) similar to the
[lsblk](https://manpages.debian.org/unstable/util-linux/lsblk.8.en.html)
utility from [util-linux](https://github.com/karelzak/util-linux).  However, it
is still a work in progress.  It is written in the Rust programming language.

## Example

```
$ rsblk
NAME      CLASS  SIZE
ada0             
└─ada0p1  DEV    1000204845056
nvd1             
├─nvd1p2  DEV    989849763840
└─nvd1p1  DEV    34359738368
md0              
└─md0p1   DEV    102400
nvd0             
├─nvd0p4  DEV    262144
├─nvd0p3  DEV    8124087808
├─nvd0p2  DEV    491773755392
└─nvd0p1  DEV    209715200
```

By default (and very much subject to change), `rsblk` prints three columns: the
NAME of a device; its
[GEOM(4)](https://www.freebsd.org/cgi/man.cgi?query=geom&sektion=4) CLASS
(which will always be "DEV"); and the object's SIZE, in bytes.
