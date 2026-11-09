# Description
Rust zero cost abstraction, adding types for data sizes like BitsU32 or BytesUSize, for better code readability. Also implementing const conversion functions between every type and between every variation.
# Example
```rust
struct MyStruct {
    index: usize,
    values: [usize; 10],
}

let size = ByteSizeUSize::of::<MyStruct>();
assert_eq!(size.get(), 11 * size_of::<usize>());

let a = BitSizeU8::of::<u32>();
asser_eq!(a.get(), 32);

let b = WordSizeU8::of::<u128>();
assert_eq!(b.get(), 4);

let c = b + a.into();
assert_eq!(c.get(), 5);
```
# Types
|Type|Size (Bits)|Feature|Default Feature|
|----|----|----|----|
|Bits|1|bits|yes|
|Bytes|8|bytes|yes|
|HWords|16|hwords|no|
|Words|32|words|yes|
|DWords|64|dwords|no|
|QWords|128|qwords|no|
|YWords|256|ywords|no|
|ZWords|512|zwords|no|
# Variations
|Inner Type|Feature|Defafult Feature|
|----|----|----|
|u8|u8|yes|
|u16|u16|no|
|u32|u32|yes|
|u64|u64|no|
|u128|u128|no|
|usize|usize|yes|
