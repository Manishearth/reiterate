## reiterate

[![Build Status](https://travis-ci.org/Manishearth/reiterate.svg?branch=master)](https://travis-ci.org/Manishearth/reiterate)
[![Current Version](https://img.shields.io/crates/v/reiterate.svg)](https://crates.io/crates/reiterate)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/reiterate.svg)](#license)

An adaptor around an iterator that can produce multiple iterators sharing an underlying cache.

The underlying iterator must produce heap-allocated StableDeref values,
e.g. Box or String. If you have an iterator that produces Copy values,
use `CopyReiterator` instead.

```rust
use reiterate::Reiterate;
let x = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
let reiterate = Reiterate::new(x);
for i in &reiterate {
    println!("{}", i);    
}
for i in &reiterate {
    // will reuse cached values
    println!("{}", i);    
}
```
