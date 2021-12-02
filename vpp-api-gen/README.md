# VPP-API-GEN 
This is a work-in-progress repo for low-level VPP API in Rust. At present all it does is load the `.api.json` files and generates
a package in Rust with their respective bindings, tests and examples. Currently, Support for enumflags is still under development while 
rest of the bindings have been tested to work with no problems. 

## Prerequisite
To use this crate to generate low level VPP API in Rust, You need to ensure that the following has been installed in your system
- **Rust** >= 1.5.0 
- **VPP** >= 21.01 
- **Ubuntu** >= 18.04 

## Tutorial 
To generate the package, enter the following command
```
cargo run -- --in-file <build-dir> --parse-type Tree --create-package --package-name <package-name> 
```
Here **build-dir** refers to the build directory of VPP where VPP API JSONs can be found, You can also alternatively use `testdata` 
however it has been tested with VPP version 21.01, It could potentially have problems when interacting with different releases of VPP 

To test the generated package, You can run the example **progressive-vpp** 

``` 
cargo run --example progressive-vpp
``` 

To ensure that you do not encounter a permission denied, either ensure that your user is in group "vpp" (preferred) or relax the write
permission on the VPP sockets:
```
sudo chmod o+w /run/vpp/api.sock
sudo chmod o+w /run/vpp/cli.sock
```

To verify that everything has worked correctly, hop into vppctl and type
```
show int addr
``` 
If there are no problems, then the ideal output would look like this 
``` 
host-vpp1out (dn):
  L3 10.10.1.2/24
local0 (dn):
``` 
## Example Message 
Sending a message to VPP without using a builder, using low level function:
```rust 
let create_interface: SwInterfaceAddDelAddressReply = send_recv_msg(
        &SwInterfaceAddDelAddress::get_message_name_and_crc(),
        &SwInterfaceAddDelAddress {
            client_index: t.get_client_index(),
            context: 0,
            is_add: true,
            del_all: false,
            sw_if_index: 1,
            prefix: AddressWithPrefix {
                address: Address {
                    af: AddressFamily::ADDRESS_IP4,
                    un: AddressUnion::new_Ip4Address([10,10,1,2]),
                },
                len: 24,
            },
        },
        &mut *t,
        &SwInterfaceAddDelAddressReply::get_message_name_and_crc(),
    );
```

Sending a message to VPP using a builder 
```rust
let create_host_interface: CliInbandReply = send_recv_msg(
        &CliInband::get_message_name_and_crc(),
        &CliInband::builder()
            .client_index(t.get_client_index())
            .context(0)
            .cmd("create host-interface name vpp1out".try_into().unwrap())
            .build()
            .unwrap(),
        &mut *t,
        &CliInbandReply::get_message_name_and_crc(),
    );
```

Sending a message to VPP using a builder, using trait (thanks to Jim Pepin for the idea):

```rust
let create_host_interface: CliInbandReply = send_recv_one(
        &CliInband::builder()
            .client_index(t.get_client_index())
            .context(0)
            .cmd("create host-interface name vpp1out".try_into().unwrap())
            .build()
            .unwrap(),
        &mut *t,
    );
```

(This method works without the builder as well)

## File Architecture
**alias.rs** 
- This file contains everything related to aliases in the binary apis ( or typedefs) 
- Structures for parsing and generating code from api json files 

**basetypes.rs**
- This file is responsible for finding the size of the types which is ultimately used in defining union 

**codegen.rs** 
- This file contains functions for generating package for VPP api  bindings and also helper functions responsible for creating **Lib** file and **Cargo.toml** file 

**enum.rs** 
- This file contains structures related to enum and enumflags defintions in the binary APIs 
- Contains functions for generating code out of the parsed structure 

**file_schema.rs** 
- This file contains structures that hold down the entire structure of the binary APIs such as Types, Enums etc, Also having functions to generate bindings for the entire API json file. 

**types.rs**
- This file contains structure and functions for parsing structs inside API json files and generate code respectively 

**message.rs** 
- This file contains structures and functions for parsing messages that interact with VPP. 

## VPP Macros 
These macros help improve the code readability of the bindings and reduce the amount of code - [vpp-api-macros](https://github.com/ayourtch/vpp-api-macros)
Currently, Macros are being used for **Builder** of Messages and for handling **Unions**


