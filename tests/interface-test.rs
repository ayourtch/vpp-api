#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_must_use,
    non_camel_case_types,
    unused_imports
)]
use bincode::Options;
use clap::Clap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::mem::drop;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use vpp_api_encoding::typ::*;
use vpp_api_gen::interface::*;
use vpp_api_gen::reqrecv::*;
use vpp_api_transport::*;

use typenum::{U10, U24, U256, U32, U64};

use vpp_api_transport::afunix;
use vpp_api_transport::shmem;
use vpp_api_transport::VppApiTransport;

fn get_encoder() -> impl bincode::config::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

#[test]
fn test_vpp_functions() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    // dbg!(t.connect("api-test", None, 256));
    let vl_msg_id = t.get_msg_index("control_ping_51077d14").unwrap();
    assert_eq!(vl_msg_id, 571);
}
#[test]
fn test_sw_interface_add_del_address() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);

    let create_interface: SwInterfaceAddDelAddressReply = send_recv_msg(
        &SwInterfaceAddDelAddress::get_message_id(),
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
        &SwInterfaceAddDelAddressReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_flags() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);

    let create_interface: SwInterfaceSetFlagsReply = send_recv_msg(
        &SwInterfaceSetFlags::get_message_id(),
        &SwInterfaceSetFlags {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 0,
            flags: IfStatusFlags::IF_STATUS_API_FLAG_LINK_UP,
        },
        &mut *t,
        &SwInterfaceSetFlagsReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_promisc() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    // dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);

    let create_interface: SwInterfaceSetPromiscReply = send_recv_msg(
        &SwInterfaceSetPromisc::get_message_id(),
        &SwInterfaceSetPromisc {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 0,
            promisc_on: false,
        },
        &mut *t,
        &SwInterfaceSetPromiscReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_hw_interface_set_mtu() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&HwInterfaceSetMtu::get_message_id()).unwrap();

    let create_interface: HwInterfaceSetMtuReply = send_recv_msg(
        &HwInterfaceSetMtu::get_message_id(),
        &HwInterfaceSetMtu {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 0,
            mtu: 50,
        },
        &mut *t,
        &HwInterfaceSetMtuReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_mtu() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetMtu::get_message_id()).unwrap();

    let create_interface: SwInterfaceSetMtuReply = send_recv_msg(
        &SwInterfaceSetMtu::get_message_id(),
        &SwInterfaceSetMtu {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 0,
            mtu: 50,
        },
        &mut *t,
        &SwInterfaceSetMtuReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_ip_directed_broadcast() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetIpDirectedBroadcast::get_message_id()).unwrap();

    let create_interface: SwInterfaceSetIpDirectedBroadcastReply = send_recv_msg(
        &SwInterfaceSetIpDirectedBroadcast::get_message_id(),
        &SwInterfaceSetIpDirectedBroadcast {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 0,
            enable: true,
        },
        &mut *t,
        &SwInterfaceSetIpDirectedBroadcastReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_want_interface_events() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&WantInterfaceEvents::get_message_id()).unwrap();

    let create_interface: WantInterfaceEventsReply = send_recv_msg(
        &WantInterfaceEvents::get_message_id(),
        &WantInterfaceEvents {
            client_index: t.get_client_index(),
            context: 0,
            enable_disable: 32,
            pid: 32,
        },
        &mut *t,
        &WantInterfaceEventsReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_address_replace_begin() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&WantInterfaceEvents::get_message_id()).unwrap();

    let create_interface: SwInterfaceAddressReplaceBeginReply = send_recv_msg(
        &SwInterfaceAddressReplaceBegin::get_message_id(),
        &SwInterfaceAddressReplaceBegin {
            client_index: t.get_client_index(),
            context: 0,
        },
        &mut *t,
        &SwInterfaceAddressReplaceBeginReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_address_replace_end() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&WantInterfaceEvents::get_message_id()).unwrap();

    let create_interface: SwInterfaceAddressReplaceEndReply = send_recv_msg(
        &SwInterfaceAddressReplaceEnd::get_message_id(),
        &SwInterfaceAddressReplaceEnd {
            client_index: t.get_client_index(),
            context: 0,
        },
        &mut *t,
        &SwInterfaceAddressReplaceEndReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_table() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceSetTableReply = send_recv_msg(
        &SwInterfaceSetTable::get_message_id(),
        &SwInterfaceSetTable {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            is_ipv6: false,
            vrf_id: 32,
        },
        &mut *t,
        &SwInterfaceSetTableReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_get_table() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceGetTableReply = send_recv_msg(
        &SwInterfaceGetTable::get_message_id(),
        &SwInterfaceGetTable {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            is_ipv6: false,
        },
        &mut *t,
        &SwInterfaceGetTableReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_unnumbered() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceSetUnnumberedReply = send_recv_msg(
        &SwInterfaceSetUnnumbered::get_message_id(),
        &SwInterfaceSetUnnumbered {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            unnumbered_sw_if_index: 2,
            is_add: false,
        },
        &mut *t,
        &SwInterfaceSetUnnumberedReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_clear_stats() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceClearStatsReply = send_recv_msg(
        &SwInterfaceClearStats::get_message_id(),
        &SwInterfaceClearStats {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
        },
        &mut *t,
        &SwInterfaceClearStatsReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_tag_add_del() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceTagAddDelReply = send_recv_msg(
        &SwInterfaceTagAddDel::get_message_id(),
        &SwInterfaceTagAddDel {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            is_add: false,
            tag: "Faisal".try_into().unwrap(),
        },
        &mut *t,
        &SwInterfaceTagAddDelReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_add_del_mac_address() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceAddDelMacAddressReply = send_recv_msg(
        &SwInterfaceAddDelMacAddress::get_message_id(),
        &SwInterfaceAddDelMacAddress {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            is_add: 0,
            addr: [0, 0x01, 0x02, 0x03, 0x04, 0x05],
        },
        &mut *t,
        &SwInterfaceAddDelMacAddressReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_mac_address() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceSetMacAddressReply = send_recv_msg(
        &SwInterfaceSetMacAddress::get_message_id(),
        &SwInterfaceSetMacAddress {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            mac_address: [0, 0x01, 0x02, 0x03, 0x04, 0x05],
        },
        &mut *t,
        &SwInterfaceSetMacAddressReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_get_mac_address() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceGetMacAddressReply = send_recv_msg(
        &SwInterfaceGetMacAddress::get_message_id(),
        &SwInterfaceGetMacAddress {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
        },
        &mut *t,
        &SwInterfaceGetMacAddressReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
#[test]
fn test_sw_interface_set_rx_mode() {
    let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    dbg!(t.connect("api-test", None, 256));
    t.set_nonblocking(false);
    // let vl_msg_id = t.get_msg_index(&SwInterfaceSetTable::get_message_id()).unwrap();

    let create_interface: SwInterfaceGetMacAddressReply = send_recv_msg(
        &SwInterfaceGetMacAddress::get_message_id(),
        &SwInterfaceGetMacAddress {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
        },
        &mut *t,
        &SwInterfaceGetMacAddressReply::get_message_id(),
    );

    assert_eq!(create_interface.context, 0);
    t.disconnect();
    // drop(t);
    // share_vpp(t);
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
