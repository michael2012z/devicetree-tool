# Design

## General

`devicetree-tool` is both:
- A Rust crate that can be used for manipulating device trees
- A command line tool based on the crate that can be used to encode and decode device tree files

The center of `devicetree-tool` is the meta data consisting the device tree:
- `DeviceTree` - The top level structure of the device tree meta data
- `Device` - A node in the device tree, representing a device
- `Attribute` - An attribute item of the device node

The device tree meta data can be created from the source code of the crate consumer, or be built from the content of DTS or DTB files. The meta data can also be accessed in user source code, or be encoded to DTS or DTB format.

## Folder structure

```
├── Cargo.lock
├── Cargo.toml
├── docs
│   ├── design.md
│   └── functionality.md
├── src
│   ├── attribute.rs
│   ├── device.rs
│   ├── devicetree.rs
│   ├── devicetree-tool.rs
│   ├── dtb.rs
│   └── dts.rs
├── LICENSE
└── README.md
```

## Core structs

### DeviceTree (`devicetree.rs`)

`DeviceTree` is the top level struct of the device tree meta data. It presents the whole device tree, holding all the device nodes of the tree.

`DeviceTree` export functions:
- `new_empty()`: Create an empty device tree
- `new_nodes()`: Create a device tree with a group of device node
- `add_device()`: Add a device node to the tree
- `find_device()`: Find a device node with specified name
- `remove_device()`: Remove a device node
- `from_dts()`: Build device tree meta data from DTS text
- `to_dts()`: Generate DTS text from the meta data
- `from_dtb()`: Build device tree meta data from DTB binary
- `to_dtb()`: Generate DTB binary from the meta data

### Device (`device.rs`)

`Device` is a device node in the tree meta data.

### Attribute (`attribute.rs`)

`Attribute` is the attribute of a device node.

### DTS (`dts.rs`)

`Dts` provides the functionality for handling DTS format.

### DTB (`dtb.rs`)

`Dtb` provides the functionality for handling DTB format.

### DeviceTree-Tool (`devicetree-tool.rs`)

`DeviceTree-Tool` is the helper struct for `devicetree-tool` binary. The only user of `DeviceTree-Tool` is the `main()` function of the program.
