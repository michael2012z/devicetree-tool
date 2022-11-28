# Functionality

`devicetree-tool` provides a command line tool for handling device tree files, and a Rust crate for manipulating device tree meta data.

The following sections illustrate how the functionalities should work.

## Creating DTS from meta data

Sample source code for creating the device tree from meta data:

``` Rust
let dt = DeviceTree::new();
let device0 = Device::new();

device0.setName("foo");
device0.addAttribute(Attribute::new_u32("attr0", 0u32));
device0.addAttribute(Attribute::new_str("attr1", "hello"));

let device1 = Device::new_attributes("bar", [
    Attribute::new_u32("attr0", 0u32),
    Attribute::new_str("attr1", "hello"),
]);
dt.addDevice(device0);
dt.addDevice(device1);

let dts = dt.to_dts();
println!("{dts}");
```

## Parsing DTS to meta data

Sample source code for loading a DTS file from file system and parsing the text into device tree meta data:

``` Rust
let dts_bytes = vec![0u8]; // Pretend to be the DTS content
let dt = DeviceTree::from_dts(dts_bytes);
```

## Creating DTB from meta data

Sample source code for generating the DTB binary content from device tree meta data:

``` Rust
let dt = DeviceTree::new();
let dtb = dt.to_dtb();
println!("{dtb}");
```

## Parsing DTB to meta data

Sample source code for loading a DTB file from file system and parse the binary data into device tree meta data:

``` Rust
let dtb_bytes = vec![0u8]; // Pretend to be the DTB content
let dt = DeviceTree::from_dtb(dtb_bytes);
```

## Encoding DTS to DTB

Example command to generate a DTB file from a DTS file:

```
devicetree-tool --input-format=DTS --input-file=foo.dts --output-format=DTB --output-file=foo.dtb
```

## Decoding DTB to DTS

Example command to parsing a DTB file to a DTS file:

```
devicetree-tool --input-format=DTB --input-file=foo.dtb --output-format=DTS --output-file=foo.dts
```