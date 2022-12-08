use core::fmt::Debug;

pub trait VppApiMessage {
    fn get_message_name_and_crc() -> String
    where
        Self: Sized;
}

pub trait VppApiRequestMessageBody: Debug + VppApiMessage {}

pub trait VppApiReplyMessageBody: Debug + VppApiMessage {}

#[derive(Debug)]
pub struct VppApiRequest {
    client_index: u32,
    context: u32,
}

#[derive(Debug)]
pub struct VppApiReply {
    context: u32,
}

#[derive(Debug)]
pub enum VppMessage<'a> {
    Request(VppApiRequest, &'a dyn VppApiRequestMessageBody),
    Reply(VppApiReply, &'a dyn VppApiReplyMessageBody),
}
