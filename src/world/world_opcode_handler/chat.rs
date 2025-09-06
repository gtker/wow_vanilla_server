use crate::world::world::client::Client;
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
        CMSG_MESSAGECHAT_ChatType::Whisper { target_player } => {
            whisper(client, clients, target_player, m.message.clone()).await;
            return;
        }
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

async fn whisper(
    sender: &mut Client,
    clients: &mut [Client],
    target_player: String,
    message: String,
) {
    if sender.character().name.eq_ignore_ascii_case(&target_player) {
        sender
            .send_system_message("You cannot whisper to yourself.")
            .await;
        return;
    }

    let target = clients
        .iter_mut()
        .find(|c| c.character().name.eq_ignore_ascii_case(&target_player));

    if let Some(target) = target {
        let inform = SMSG_MESSAGECHAT {
            chat_type: SMSG_MESSAGECHAT_ChatType::WhisperInform {
                sender2: target.character().guid,
            },
            language: Language::Universal,
            message: message.clone(),
            tag: PlayerChatTag::None,
        };
        sender.send_message(inform).await;

        let whisper = SMSG_MESSAGECHAT {
            chat_type: SMSG_MESSAGECHAT_ChatType::Whisper {
                sender2: sender.character().guid,
            },
            language: Language::Universal,
            message,
            tag: PlayerChatTag::None,
        };
        target.send_message(whisper).await;
    }
}
