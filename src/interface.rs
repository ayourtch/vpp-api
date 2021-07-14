/* Autogenerated data. Do not edit */
#![allow(non_camel_case_types)]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use vpp_api_encoding::typ::*;
use vpp_api_transport::*;
use typenum::{U10, U24, U256, U32, U64};
use serde_repr::{Serialize_repr, Deserialize_repr};
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Address { 
	pub af : AddressFamily, 
	pub un : AddressUnion, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Prefix { 
	pub address : Address, 
	pub len : u8, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Ip4AddressAndMask { 
	pub addr : Ip4Address, 
	pub mask : Ip4Address, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Ip6AddressAndMask { 
	pub addr : Ip6Address, 
	pub mask : Ip6Address, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Mprefix { 
	pub af : AddressFamily, 
	pub grp_address_length : u16, 
	pub grp_address : AddressUnion, 
	pub src_address : AddressUnion, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Ip6Prefix { 
	pub address : Ip6Address, 
	pub len : u8, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct Ip4Prefix { 
	pub address : Ip4Address, 
	pub len : u8, 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct PrefixMatcher { 
	pub le : u8, 
	pub ge : u8, 
} 
pub type AddressUnion = [u8;16];
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u32)]
pub enum IfStatusFlags { 
	 IF_STATUS_API_FLAG_ADMIN_UP=1, 
	 IF_STATUS_API_FLAG_LINK_UP=2, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u32)]
pub enum MtuProto { 
	 MTU_PROTO_API_L3=0, 
	 MTU_PROTO_API_IP4=1, 
	 MTU_PROTO_API_IP6=2, 
	 MTU_PROTO_API_MPLS=3, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u32)]
pub enum LinkDuplex { 
	 LINK_DUPLEX_API_UNKNOWN=0, 
	 LINK_DUPLEX_API_HALF=1, 
	 LINK_DUPLEX_API_FULL=2, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u32)]
pub enum SubIfFlags { 
	 SUB_IF_API_FLAG_NO_TAGS=1, 
	 SUB_IF_API_FLAG_ONE_TAG=2, 
	 SUB_IF_API_FLAG_TWO_TAGS=4, 
	 SUB_IF_API_FLAG_DOT1AD=8, 
	 SUB_IF_API_FLAG_EXACT_MATCH=16, 
	 SUB_IF_API_FLAG_DEFAULT=32, 
	 SUB_IF_API_FLAG_OUTER_VLAN_ID_ANY=64, 
	 SUB_IF_API_FLAG_INNER_VLAN_ID_ANY=128, 
	 SUB_IF_API_FLAG_MASK_VNET=254, 
	 SUB_IF_API_FLAG_DOT1AH=256, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u32)]
pub enum RxMode { 
	 RX_MODE_API_UNKNOWN=0, 
	 RX_MODE_API_POLLING=1, 
	 RX_MODE_API_INTERRUPT=2, 
	 RX_MODE_API_ADAPTIVE=3, 
	 RX_MODE_API_DEFAULT=4, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u32)]
pub enum IfType { 
	 IF_API_TYPE_HARDWARE=0, 
	 IF_API_TYPE_SUB=1, 
	 IF_API_TYPE_P2P=2, 
	 IF_API_TYPE_PIPE=3, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
pub enum Direction { 
	 RX=0, 
	 TX=1, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
pub enum AddressFamily { 
	 ADDRESS_IP4=0, 
	 ADDRESS_IP6=1, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
pub enum IpFeatureLocation { 
	 IP_API_FEATURE_INPUT=0, 
	 IP_API_FEATURE_OUTPUT=1, 
	 IP_API_FEATURE_LOCAL=2, 
	 IP_API_FEATURE_PUNT=3, 
	 IP_API_FEATURE_DROP=4, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
pub enum IpEcn { 
	 IP_API_ECN_NONE=0, 
	 IP_API_ECN_ECT0=1, 
	 IP_API_ECN_ECT1=2, 
	 IP_API_ECN_CE=3, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
pub enum IpDscp { 
	 IP_API_DSCP_CS0=0, 
	 IP_API_DSCP_CS1=8, 
	 IP_API_DSCP_AF11=10, 
	 IP_API_DSCP_AF12=12, 
	 IP_API_DSCP_AF13=14, 
	 IP_API_DSCP_CS2=16, 
	 IP_API_DSCP_AF21=18, 
	 IP_API_DSCP_AF22=20, 
	 IP_API_DSCP_AF23=22, 
	 IP_API_DSCP_CS3=24, 
	 IP_API_DSCP_AF31=26, 
	 IP_API_DSCP_AF32=28, 
	 IP_API_DSCP_AF33=30, 
	 IP_API_DSCP_CS4=32, 
	 IP_API_DSCP_AF41=34, 
	 IP_API_DSCP_AF42=36, 
	 IP_API_DSCP_AF43=38, 
	 IP_API_DSCP_CS5=40, 
	 IP_API_DSCP_EF=46, 
	 IP_API_DSCP_CS6=48, 
	 IP_API_DSCP_CS7=50, 
} 
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] 
#[repr(u8)]
pub enum IpProto { 
	 IP_API_PROTO_HOPOPT=0, 
	 IP_API_PROTO_ICMP=1, 
	 IP_API_PROTO_IGMP=2, 
	 IP_API_PROTO_TCP=6, 
	 IP_API_PROTO_UDP=17, 
	 IP_API_PROTO_GRE=47, 
	 IP_API_PROTO_ESP=50, 
	 IP_API_PROTO_AH=51, 
	 IP_API_PROTO_ICMP6=58, 
	 IP_API_PROTO_EIGRP=88, 
	 IP_API_PROTO_OSPF=89, 
	 IP_API_PROTO_SCTP=132, 
	 IP_API_PROTO_RESERVED=255, 
} 
pub type InterfaceIndex=u32; 
pub type MacAddress=[u8;6]; 
pub type Ip4Address=[u8;4]; 
pub type Ip6Address=[u8;16]; 
pub type AddressWithPrefix=Prefix; 
pub type Ip4AddressWithPrefix=Ip4Prefix; 
pub type Ip6AddressWithPrefix=Ip6Prefix; 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetFlags { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub flags : IfStatusFlags, 
} 
impl SwInterfaceSetFlags { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_flags_6a2b491a") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetFlagsReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetFlagsReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_flags_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetPromisc { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub promisc_on : bool, 
} 
impl SwInterfaceSetPromisc { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_promisc_d40860d4") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetPromiscReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetPromiscReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_promisc_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct HwInterfaceSetMtu { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub mtu : u16, 
} 
impl HwInterfaceSetMtu { 
	 pub fn get_message_id() -> String { 
	 	 String::from("hw_interface_set_mtu_e6746899") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct HwInterfaceSetMtuReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl HwInterfaceSetMtuReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("hw_interface_set_mtu_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetMtu { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub mtu : u32, 
} 
impl SwInterfaceSetMtu { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_mtu_5cbe85e5") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetMtuReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetMtuReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_mtu_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetIpDirectedBroadcast { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub enable : bool, 
} 
impl SwInterfaceSetIpDirectedBroadcast { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_ip_directed_broadcast_ae6cfcfb") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetIpDirectedBroadcastReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetIpDirectedBroadcastReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_ip_directed_broadcast_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceEvent { 
	pub client_index : u32, 
	pub pid : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub flags : IfStatusFlags, 
	pub deleted : bool, 
} 
impl SwInterfaceEvent { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_event_f709f78d") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct WantInterfaceEvents { 
	pub client_index : u32, 
	pub context : u32, 
	pub enable_disable : u32, 
	pub pid : u32, 
} 
impl WantInterfaceEvents { 
	 pub fn get_message_id() -> String { 
	 	 String::from("want_interface_events_476f5a08") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct WantInterfaceEventsReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl WantInterfaceEventsReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("want_interface_events_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceDetails { 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub sup_sw_if_index : u32, 
	pub l2_address : MacAddress, 
	pub flags : IfStatusFlags, 
	pub typ : IfType, 
	pub link_duplex : LinkDuplex, 
	pub link_speed : u32, 
	pub link_mtu : u16, 
	pub mtu : u32, 
	pub sub_id : u32, 
	pub sub_number_of_tags : u8, 
	pub sub_outer_vlan_id : u16, 
	pub sub_inner_vlan_id : u16, 
	pub sub_if_flags : SubIfFlags, 
	pub vtr_op : u32, 
	pub vtr_push_dot1q : u32, 
	pub vtr_tag1 : u32, 
	pub vtr_tag2 : u32, 
	pub outer_tag : u16, 
	pub b_dmac : MacAddress, 
	pub b_smac : MacAddress, 
	pub b_vlanid : u16, 
	pub i_sid : u32, 
	pub interface_name : FixedSizeString<U64>, 
	pub interface_dev_type : FixedSizeString<U64>, 
	pub tag : FixedSizeString<U64>, 
} 
impl SwInterfaceDetails { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_details_17b69fa2") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceDump { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub name_filter_valid : bool, 
	pub name_filter : VariableSizeString, 
} 
impl SwInterfaceDump { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_dump_aa610c27") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddDelAddress { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub is_add : bool, 
	pub del_all : bool, 
	pub prefix : AddressWithPrefix, 
} 
impl SwInterfaceAddDelAddress { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_add_del_address_5803d5c4") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddDelAddressReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceAddDelAddressReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_add_del_address_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddressReplaceBegin { 
	pub client_index : u32, 
	pub context : u32, 
} 
impl SwInterfaceAddressReplaceBegin { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_address_replace_begin_51077d14") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddressReplaceBeginReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceAddressReplaceBeginReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_address_replace_begin_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddressReplaceEnd { 
	pub client_index : u32, 
	pub context : u32, 
} 
impl SwInterfaceAddressReplaceEnd { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_address_replace_end_51077d14") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddressReplaceEndReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceAddressReplaceEndReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_address_replace_end_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetTable { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub is_ipv6 : bool, 
	pub vrf_id : u32, 
} 
impl SwInterfaceSetTable { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_table_df42a577") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetTableReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetTableReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_table_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceGetTable { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub is_ipv6 : bool, 
} 
impl SwInterfaceGetTable { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_get_table_2d033de4") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceGetTableReply { 
	pub context : u32, 
	pub retval : i32, 
	pub vrf_id : u32, 
} 
impl SwInterfaceGetTableReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_get_table_reply_a6eb0109") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetUnnumbered { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub unnumbered_sw_if_index : InterfaceIndex, 
	pub is_add : bool, 
} 
impl SwInterfaceSetUnnumbered { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_unnumbered_938ef33b") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetUnnumberedReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetUnnumberedReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_unnumbered_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceClearStats { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl SwInterfaceClearStats { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_clear_stats_f9e6675e") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceClearStatsReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceClearStatsReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_clear_stats_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceTagAddDel { 
	pub client_index : u32, 
	pub context : u32, 
	pub is_add : bool, 
	pub sw_if_index : InterfaceIndex, 
	pub tag : FixedSizeString<U64>, 
} 
impl SwInterfaceTagAddDel { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_tag_add_del_426f8bc1") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceTagAddDelReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceTagAddDelReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_tag_add_del_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddDelMacAddress { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : u32, 
	pub addr : MacAddress, 
	pub is_add : u8, 
} 
impl SwInterfaceAddDelMacAddress { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_add_del_mac_address_638bb9f4") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceAddDelMacAddressReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceAddDelMacAddressReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_add_del_mac_address_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetMacAddress { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub mac_address : MacAddress, 
} 
impl SwInterfaceSetMacAddress { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_mac_address_6aca746a") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetMacAddressReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetMacAddressReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_mac_address_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceGetMacAddress { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl SwInterfaceGetMacAddress { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_get_mac_address_f9e6675e") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceGetMacAddressReply { 
	pub context : u32, 
	pub retval : i32, 
	pub mac_address : MacAddress, 
} 
impl SwInterfaceGetMacAddressReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_get_mac_address_reply_40ef2c08") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetRxMode { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub queue_id_valid : bool, 
	pub queue_id : u32, 
	pub mode : RxMode, 
} 
impl SwInterfaceSetRxMode { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_rx_mode_780f5cee") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetRxModeReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetRxModeReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_rx_mode_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetRxPlacement { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub queue_id : u32, 
	pub worker_id : u32, 
	pub is_main : bool, 
} 
impl SwInterfaceSetRxPlacement { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_rx_placement_db65f3c9") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceSetRxPlacementReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl SwInterfaceSetRxPlacementReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_set_rx_placement_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceRxPlacementDump { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl SwInterfaceRxPlacementDump { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_rx_placement_dump_f9e6675e") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct SwInterfaceRxPlacementDetails { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub queue_id : u32, 
	pub worker_id : u32, 
	pub mode : RxMode, 
} 
impl SwInterfaceRxPlacementDetails { 
	 pub fn get_message_id() -> String { 
	 	 String::from("sw_interface_rx_placement_details_f6d7d024") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct InterfaceNameRenumber { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub new_show_dev_instance : u32, 
} 
impl InterfaceNameRenumber { 
	 pub fn get_message_id() -> String { 
	 	 String::from("interface_name_renumber_2b8858b8") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct InterfaceNameRenumberReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl InterfaceNameRenumberReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("interface_name_renumber_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateSubif { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub sub_id : u32, 
	pub sub_if_flags : SubIfFlags, 
	pub outer_vlan_id : u16, 
	pub inner_vlan_id : u16, 
} 
impl CreateSubif { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_subif_cb371063") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateSubifReply { 
	pub context : u32, 
	pub retval : i32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl CreateSubifReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_subif_reply_5383d31f") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateVlanSubif { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub vlan_id : u32, 
} 
impl CreateVlanSubif { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_vlan_subif_af34ac8b") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateVlanSubifReply { 
	pub context : u32, 
	pub retval : i32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl CreateVlanSubifReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_vlan_subif_reply_5383d31f") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct DeleteSubif { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl DeleteSubif { 
	 pub fn get_message_id() -> String { 
	 	 String::from("delete_subif_f9e6675e") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct DeleteSubifReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl DeleteSubifReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("delete_subif_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateLoopback { 
	pub client_index : u32, 
	pub context : u32, 
	pub mac_address : MacAddress, 
} 
impl CreateLoopback { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_loopback_42bb5d22") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateLoopbackReply { 
	pub context : u32, 
	pub retval : i32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl CreateLoopbackReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_loopback_reply_5383d31f") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateLoopbackInstance { 
	pub client_index : u32, 
	pub context : u32, 
	pub mac_address : MacAddress, 
	pub is_specified : bool, 
	pub user_instance : u32, 
} 
impl CreateLoopbackInstance { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_loopback_instance_d36a3ee2") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CreateLoopbackInstanceReply { 
	pub context : u32, 
	pub retval : i32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl CreateLoopbackInstanceReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("create_loopback_instance_reply_5383d31f") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct DeleteLoopback { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
} 
impl DeleteLoopback { 
	 pub fn get_message_id() -> String { 
	 	 String::from("delete_loopback_f9e6675e") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct DeleteLoopbackReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl DeleteLoopbackReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("delete_loopback_reply_e8d4e804") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CollectDetailedInterfaceStats { 
	pub client_index : u32, 
	pub context : u32, 
	pub sw_if_index : InterfaceIndex, 
	pub enable_disable : bool, 
} 
impl CollectDetailedInterfaceStats { 
	 pub fn get_message_id() -> String { 
	 	 String::from("collect_detailed_interface_stats_5501adee") 
	 } 
} 
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct CollectDetailedInterfaceStatsReply { 
	pub context : u32, 
	pub retval : i32, 
} 
impl CollectDetailedInterfaceStatsReply { 
	 pub fn get_message_id() -> String { 
	 	 String::from("collect_detailed_interface_stats_reply_e8d4e804") 
	 } 
} 
