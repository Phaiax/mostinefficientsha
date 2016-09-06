Most Inefficient Sha
========

[Documentation](https://phaiax.github.io/mostinefficientsha/mostinefficientsha/index.html)

This crate tries to break SHA256. (But fails unfortunately^^).

It implements SHA-256 in the most inefficient fashion ever. Usually, SHA-256 operates on integers with 32 bits (u32). This implementation uses one double (f64) for each bit of each integer, thereby allowing to use 8 fuzzy input bits for each input byte.

## Benchmarks

Processor: Intel(R) Core(TM)2 Duo CPU     P8400  @ 2.26GHz, 3072KB Cache

* Calculating SHA256 with one byte of input data takes 10 ms.
* Calculating SHA256 with one hundred bytes of input data takes 78 ms.

## Why

For fun. It seemed unlikely to work out but I wanted to do some rust.
