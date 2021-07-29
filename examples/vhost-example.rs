use bincode::Options;
use std::convert::TryInto;
use vpp_api_gen::interface::*;
use vpp_api_gen::interface_types::*;
use vpp_api_gen::vhost_user::*;
use vpp_api_gen::virtio_types::*;
use vpp_api_gen::ip_types::*;
use vpp_api_gen::reqrecv::*;

use vpp_api_transport::afunix;
use vpp_api_transport::VppApiTransport;

fn get_encoder() -> impl bincode::config::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

fn main(){
    // Connecting to the VPP API socket
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    // Testing Connection and loading Message buffers
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    // Checking Control Ping ID 
    let vl_msg_id = t.get_msg_index("control_ping_51077d14").unwrap();
    println!("Control Ping MSG_ID: {}", vl_msg_id);

    let create_interface: SwInterfaceAddDelAddressReply = send_recv_msg(
        &SwInterfaceAddDelAddress::get_message_name_and_crc(),
        &SwInterfaceAddDelAddress {
            client_index: t.get_client_index(),
            context: 0,
            is_add: true,
            del_all: false,
            sw_if_index: 0,
            prefix: AddressWithPrefix {
                address: Address {
                    af: AddressFamily::ADDRESS_IP4,
                    un: [
                        0xa, 0xa, 1, 2, 7, 0x7a, 0xb, 0xc, 0xd, 0xf, 8, 9, 5, 6, 10, 10,
                    ],
                },
                len: 24,
            },
        },
        &mut *t,
        &SwInterfaceAddDelAddressReply::get_message_name_and_crc(),
    );
    println!("{:?}",create_interface);

    /*let ipaddress:Vec<IpAddressDetails>  = send_bulk_msg(
        "ip_address_dump_2d033de4",
        &IpAddressDump{
            client_index: t.get_client_index(), 
            context: 0,
            sw_if_index: 0, 
            is_ipv6: false
        },
        &mut *t,
        "ip_address_details_b1199745"
    );
    // [0xa,0xa,1,2,7,0x7a,0xb,0xc,0xd,0xf,8,9,5,6,10,10]
    // 10.10.1.2/24
    println!("Show IP Address Reply: {:#?}", &ipaddress);*/
    let vhostDetails:Vec<SwInterfaceVhostUserDetails> = send_bulk_msg(
        &SwInterfaceVhostUserDump::get_message_name_and_crc(), 
        &SwInterfaceVhostUserDump{
            client_index: t.get_client_index(),
            context: 0, 
            sw_if_index: 1
        }, 
        &mut *t, 
        &SwInterfaceVhostUserDetails::get_message_name_and_crc());
    
    println!("Show VhostInterfaceDetails \n {:?}", vhostDetails);

    let swinterfacedetails:Vec<SwInterfaceDetails> = send_bulk_msg(
        &SwInterfaceDump::get_message_name_and_crc(), 
        &SwInterfaceDump{
            client_index: t.get_client_index(),
            context: 0, 
            sw_if_index: 0, 
            name_filter_valid: true, 
            name_filter: "local".try_into().unwrap()
        }, 
        &mut *t, 
        &SwInterfaceDetails::get_message_name_and_crc()
    );
    println!("Available Interfaces are:"); 
    let interfacenames = swinterfacedetails.iter().fold(String::new(), |mut acc, x|{
        acc.push_str(&format!("{:?} \n", &x.interface_name));
        acc
    });
    println!("{}",interfacenames);
    


    t.disconnect();
}