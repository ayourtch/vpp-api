use bincode::Options;
use std::convert::TryInto;
use std::process::Command;
use vpp_api_gen::interface::*;
use vpp_api_gen::interface_types::*;
use vpp_api_gen::ip_types::*;
use vpp_api_gen::reqrecv::*;
use vpp_api_gen::vhost_user::*;
use vpp_api_gen::virtio_types::*;
use vpp_api_gen::vlib::CliInband;
use vpp_api_gen::vlib::CliInbandReply;
use vpp_api_gen::vlib::*;
use vpp_api_transport::afunix;
use vpp_api_transport::VppApiTransport;

fn get_encoder() -> impl bincode::config::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

fn main() {
    // Step 1: Create Veth Connection
    let mut create_veth = Command::new("sh");
    create_veth
        .arg("-c")
        .arg("sudo ip link add name vpp1out type veth peer name vpp1host");
    let mut create_veth_response = create_veth.output().expect("failed to execute process");
    println!("{:?}", create_veth_response);

    // Step 2: Link up both the end points and assign ip to host address
    create_veth = Command::new("sh");
    create_veth.arg("-c")
                  .arg("sudo ip link set dev vpp1out up && sudo ip link set dev vpp1host up && sudo ip addr add 10.10.1.1/24 dev vpp1host");
    create_veth_response = create_veth.output().expect("failed to execute process");
    println!("{:?}", create_veth_response);

    /* create_veth = Command::new("sh");
    create_veth.arg("-c")
                  .arg("sudo vppctl -s /run/vpp/cli.sock create host-interface name vpp1out && sudo vppctl -s /run/vpp/cli.sock set int ip address host-vpp1out 10.10.1.2/24
                  ");
    create_veth_response = create_veth.output().expect("failed to execute process");
    println!("{:?}", create_veth_response);*/

    // Connecting to the VPP API socket
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    // Testing Connection and loading Message buffers
    println!("Connect result: {:?}", t.connect("api-test", None, 256));

    // Step 3: Create Host interface
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
    println!("{:?}", create_host_interface);

    // Step 4: Set Host Interface State up
    let set_interface_link_up: SwInterfaceSetFlagsReply = send_recv_msg(
        &SwInterfaceSetFlags::get_message_name_and_crc(),
        &SwInterfaceSetFlags::builder().client_index(t.get_client_index()).context(0).sw_if_index(1).flags(vec![IfStatusFlags::IF_STATUS_API_FLAG_ADMIN_UP, IfStatusFlags::IF_STATUS_API_FLAG_LINK_UP].try_into().unwrap()).build().unwrap(),
        &mut *t,
        &SwInterfaceSetFlagsReply::get_message_name_and_crc());
    
    println!("{:?}", create_host_interface);

    // Step 5: Assign IP Address to Host Interface
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
    println!("{:?}", create_interface);

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
    // let vhostuserdump = SwInterfaceVhostUserDump::builder().client_index(t.get_client_index()).context(0).sw_if_index(1).build().unwrap();
    /* let vhostDetails: Vec<SwInterfaceVhostUserDetails> = send_bulk_msg(
        &SwInterfaceVhostUserDump::get_message_name_and_crc(),
        &SwInterfaceVhostUserDump::builder()
            .client_index(t.get_client_index())
            .context(0)
            .sw_if_index(1)
            .build()
            .unwrap(),
        &mut *t,
        &SwInterfaceVhostUserDetails::get_message_name_and_crc(),
    );*/

    // println!("Show VhostInterfaceDetails \n {:?}", vhostDetails);

    // Verify creation of Interface
    // FIXME: Need to implement Deserialize for FixedSizeArray to make this work
    let swinterfacedetails: Vec<SwInterfaceDetails> = send_bulk_msg(
        &SwInterfaceDump::get_message_name_and_crc(),
        &SwInterfaceDump::builder()
            .client_index(t.get_client_index())
            .context(0)
            .sw_if_index(0)
            .name_filter_valid(true)
            .name_filter("host-vpp1".try_into().unwrap())
            .build()
            .unwrap(),
        &mut *t,
        &SwInterfaceDetails::get_message_name_and_crc(),
    );
    println!("{:#?}", swinterfacedetails);
    println!("Interface IDX:");
    let interfaceids = swinterfacedetails.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(&format!("{:?} \n", &x.sw_if_index));
        acc
    });
    println!("{}", interfaceids);
    // If non empty, Test out by pinging from host

    t.disconnect();
}
