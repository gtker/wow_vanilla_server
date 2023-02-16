use crate::world::client::Client;
use wow_world_base::geometry;
use wow_world_messages::vanilla::{
    CMSG_MESSAGECHAT_ChatType, Language, PlayerChatTag, SMSG_MESSAGECHAT_ChatType,
    CMSG_MESSAGECHAT, SMSG_MESSAGECHAT,
};

pub async fn handle_message(client: &mut Client, clients: &mut [Client], m: CMSG_MESSAGECHAT) {
    let sender = client.character().guid;

    let f = match m.chat_type {
        CMSG_MESSAGECHAT_ChatType::Say => |a: &Client, b: &Client| -> bool {
            if let Some(v) = a.distance_to_center(b) {
                v < geometry::SAY
            } else {
                false
            }
        },
        CMSG_MESSAGECHAT_ChatType::Yell => |a: &Client, b: &Client| -> bool {
            if let Some(v) = a.distance_to_center(b) {
                v < geometry::YELL
            } else {
                false
            }
        },
        _ => {
            dbg!(m);
            return;
        }
    };

    let chat_type = match m.chat_type {
        CMSG_MESSAGECHAT_ChatType::Say => SMSG_MESSAGECHAT_ChatType::Say {
            chat_credit: sender,
            speech_bubble_credit: sender,
        },
        CMSG_MESSAGECHAT_ChatType::Yell => SMSG_MESSAGECHAT_ChatType::Yell {
            chat_credit: sender,
            speech_bubble_credit: sender,
        },
        _ => unreachable!(),
    };

    let message = SMSG_MESSAGECHAT {
        chat_type,
        language: Language::Universal,
        message: m.message.clone(),
        tag: PlayerChatTag::None,
    };

    client.send_message(message.clone()).await;

    for c in clients {
        if f(client, c) {
            c.send_message(message.clone()).await;
        }
    }
}
