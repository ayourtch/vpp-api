use vpp_macros::Message;
#[derive(Message)]
#[message_name_and_crc(Idiot)]
struct InterfaceAPIAddress{
    uid: i32, 
}

fn main(){
    println!("{}", InterfaceAPIAddress::get_message_name_and_crc());
    assert_eq!(1,1);
    
}