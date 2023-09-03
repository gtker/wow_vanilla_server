use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use wow_login_messages::all::{
    CMD_AUTH_LOGON_CHALLENGE_Client, CMD_AUTH_RECONNECT_CHALLENGE_Client, ProtocolVersion,
};
use wow_login_messages::errors::ExpectedOpcodeError;
use wow_login_messages::helper::{
    tokio_expect_client_message, tokio_read_initial_message, InitialMessage,
};
use wow_login_messages::ServerMessage;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::server::{SrpProof, SrpServer, SrpVerifier};
use wow_srp::{PublicKey, GENERATOR, LARGE_SAFE_PRIME_LITTLE_ENDIAN};

const EXTERNAL_WORLD_STRING: &str = "vpn.gtker.com:8085";
const INTERNAL_WORLD_STRING: &str = "localhost:8085";

pub async fn auth(users: Arc<Mutex<HashMap<String, SrpServer>>>) {
    let listener = TcpListener::bind("0.0.0.0:3724").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        tokio::spawn(handle(stream, users.clone()));
    }
}

async fn handle(mut stream: TcpStream, users: Arc<Mutex<HashMap<String, SrpServer>>>) {
    let opcode = tokio_read_initial_message(&mut stream).await;
    let opcode = match opcode {
        Ok(o) => o,
        Err(e) => {
            match e {
                ExpectedOpcodeError::Opcode(o) => {
                    println!("invalid opcode {}", o)
                }
                ExpectedOpcodeError::Parse(e) => {
                    println!("parse error {:#?}", e)
                }
                ExpectedOpcodeError::Io(i) => panic!("{}", i),
            }
            return;
        }
    };

    match opcode {
        InitialMessage::Logon(l) => match l.protocol_version {
            ProtocolVersion::Two => login_version_2(stream, l, users).await,
            ProtocolVersion::Three => login_version_3(stream, l, users).await,
            ProtocolVersion::Eight => login_version_8(stream, l, users).await,
            _ => {}
        },
        InitialMessage::Reconnect(r) => match r.protocol_version {
            ProtocolVersion::Two => reconnect_version_2(stream, r, users).await,
            ProtocolVersion::Eight => reconnect_version_8(stream, r, users).await,
            _ => {}
        },
    }
}

async fn reconnect_version_8(
    mut stream: TcpStream,
    r: CMD_AUTH_RECONNECT_CHALLENGE_Client,
    users: Arc<Mutex<HashMap<String, SrpServer>>>,
) {
    use wow_login_messages::version_8::*;

    println!("Reconnect version: {}", r.protocol_version);

    let server_reconnect_challenge_data = *users
        .lock()
        .unwrap()
        .get(&r.account_name)
        .unwrap()
        .reconnect_challenge_data();

    CMD_AUTH_RECONNECT_CHALLENGE_Server {
        result: CMD_AUTH_RECONNECT_CHALLENGE_Server_LoginResult::Success {
            challenge_data: server_reconnect_challenge_data,
            checksum_salt: [0; 16],
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();

    let l = tokio_expect_client_message::<CMD_AUTH_RECONNECT_PROOF_Client, _>(&mut stream)
        .await
        .unwrap();

    let success = {
        match users.lock().unwrap().get_mut(&r.account_name) {
            None => false,
            Some(server) => server.verify_reconnection_attempt(l.proof_data, l.client_proof),
        }
    };

    if !success {
        CMD_AUTH_RECONNECT_PROOF_Server {
            result: LoginResult::FailBanned,
        }
        .tokio_write(&mut stream)
        .await
        .unwrap();

        return;
    }

    CMD_AUTH_RECONNECT_PROOF_Server {
        result: LoginResult::Success,
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();

    print_version_8_realm_list(stream).await;
}

async fn reconnect_version_2(
    mut stream: TcpStream,
    r: CMD_AUTH_RECONNECT_CHALLENGE_Client,
    users: Arc<Mutex<HashMap<String, SrpServer>>>,
) {
    use wow_login_messages::version_2::*;

    println!("Reconnect version: {}", r.protocol_version);

    let server_reconnect_challenge_data = *users
        .lock()
        .unwrap()
        .get(&r.account_name)
        .unwrap()
        .reconnect_challenge_data();

    CMD_AUTH_RECONNECT_CHALLENGE_Server {
        result: CMD_AUTH_RECONNECT_CHALLENGE_Server_LoginResult::Success {
            challenge_data: server_reconnect_challenge_data,
            checksum_salt: [0; 16],
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();

    let l = tokio_expect_client_message::<CMD_AUTH_RECONNECT_PROOF_Client, _>(&mut stream)
        .await
        .unwrap();

    let success = {
        match users.lock().unwrap().get_mut(&r.account_name) {
            None => false,
            Some(server) => server.verify_reconnection_attempt(l.proof_data, l.client_proof),
        }
    };

    if !success {
        CMD_AUTH_RECONNECT_PROOF_Server {
            result: LoginResult::FailBanned,
        }
        .tokio_write(&mut stream)
        .await
        .unwrap();

        return;
    }

    CMD_AUTH_RECONNECT_PROOF_Server {
        result: LoginResult::Success,
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();

    print_version_2_3_realm_list(stream).await;
}

async fn login_version_2(
    mut stream: TcpStream,
    l: CMD_AUTH_LOGON_CHALLENGE_Client,
    users: Arc<Mutex<HashMap<String, SrpServer>>>,
) {
    use wow_login_messages::version_2::*;

    println!("Login version: {}", l.protocol_version);
    let p = get_proof(&l.account_name);

    let username = l.account_name;

    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::Success {
            server_public_key: *p.server_public_key(),
            generator: vec![GENERATOR],
            large_safe_prime: LARGE_SAFE_PRIME_LITTLE_ENDIAN.into(),
            salt: *p.salt(),
            crc_salt: [0; 16],
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();
    println!("Sent Logon Challenge");

    let l = tokio_expect_client_message::<CMD_AUTH_LOGON_PROOF_Client, _>(&mut stream)
        .await
        .unwrap();

    let (p, proof) = p
        .into_server(
            PublicKey::from_le_bytes(l.client_public_key).unwrap(),
            l.client_proof,
        )
        .unwrap();

    CMD_AUTH_LOGON_PROOF_Server {
        result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::Success {
            server_proof: proof,
            hardware_survey_id: 0,
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();
    println!("Sent Logon Proof");

    users.lock().unwrap().insert(username, p);

    print_version_2_3_realm_list(stream).await;
}

fn get_proof(username: &str) -> SrpProof {
    let username = NormalizedString::new(username.to_string()).unwrap();
    let password = NormalizedString::new(username.to_string()).unwrap();
    SrpVerifier::from_username_and_password(username, password).into_proof()
}

async fn login_version_3(
    mut stream: TcpStream,
    l: CMD_AUTH_LOGON_CHALLENGE_Client,
    users: Arc<Mutex<HashMap<String, SrpServer>>>,
) {
    use wow_login_messages::version_3::*;

    println!("Login version: {}", l.protocol_version);
    let p = get_proof(&l.account_name);
    let username = l.account_name;

    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::Success {
            server_public_key: *p.server_public_key(),
            generator: vec![GENERATOR],
            large_safe_prime: LARGE_SAFE_PRIME_LITTLE_ENDIAN.into(),
            salt: *p.salt(),
            crc_salt: [0; 16],
            security_flag: CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag::None,
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();
    println!("Sent Logon Challenge");

    let l = tokio_expect_client_message::<CMD_AUTH_LOGON_PROOF_Client, _>(&mut stream)
        .await
        .unwrap();

    let (p, proof) = p
        .into_server(
            PublicKey::from_le_bytes(l.client_public_key).unwrap(),
            l.client_proof,
        )
        .unwrap();

    CMD_AUTH_LOGON_PROOF_Server {
        result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::Success {
            server_proof: proof,
            hardware_survey_id: 0,
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();
    println!("Sent Logon Proof");

    users.lock().unwrap().insert(username.to_string(), p);

    print_version_2_3_realm_list(stream).await;
}
async fn login_version_8(
    mut stream: TcpStream,
    l: CMD_AUTH_LOGON_CHALLENGE_Client,
    users: Arc<Mutex<HashMap<String, SrpServer>>>,
) {
    use wow_login_messages::version_8::*;

    println!("Login version: {}", l.protocol_version);
    let p = get_proof(&l.account_name);
    let username = l.account_name;

    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::Success {
            server_public_key: *p.server_public_key(),
            generator: vec![GENERATOR],
            large_safe_prime: LARGE_SAFE_PRIME_LITTLE_ENDIAN.into(),
            salt: *p.salt(),
            crc_salt: [0; 16],
            security_flag: CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag::empty(),
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();
    println!("Sent Logon Challenge");

    let l = tokio_expect_client_message::<CMD_AUTH_LOGON_PROOF_Client, _>(&mut stream)
        .await
        .unwrap();

    let (p, server_proof) = p
        .into_server(
            PublicKey::from_le_bytes(l.client_public_key).unwrap(),
            l.client_proof,
        )
        .unwrap();

    CMD_AUTH_LOGON_PROOF_Server {
        result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::Success {
            account_flag: AccountFlag::empty(),
            server_proof,
            hardware_survey_id: 0,
            unknown_flags: 0,
        },
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();
    println!("Sent Logon Proof");

    users.lock().unwrap().insert(username.to_string(), p);

    print_version_8_realm_list(stream).await;
}

fn get_world_server_string(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(i) => {
            if i != &Ipv4Addr::new(127, 0, 0, 1) {
                EXTERNAL_WORLD_STRING.to_string()
            } else {
                INTERNAL_WORLD_STRING.to_string()
            }
        }
        IpAddr::V6(_) => EXTERNAL_WORLD_STRING.to_string(),
    }
}
async fn print_version_2_3_realm_list(mut stream: TcpStream) {
    use wow_login_messages::version_2::*;

    let addr = get_world_server_string(&stream.peer_addr().unwrap().ip());

    while (tokio_expect_client_message::<CMD_REALM_LIST_Client, _>(&mut stream).await).is_ok() {
        CMD_REALM_LIST_Server {
            realms: vec![Realm {
                realm_type: RealmType::PlayerVsEnvironment,
                flag: RealmFlag::empty(),
                name: "Location Realm".to_string(),
                address: addr.clone(),
                population: Default::default(),
                number_of_characters_on_realm: 0,
                category: Default::default(),
                realm_id: 0,
            }],
        }
        .tokio_write(&mut stream)
        .await
        .unwrap();
        println!("Sent Version 2/3 Realm List");
    }
}

async fn print_version_8_realm_list(mut stream: TcpStream) {
    use wow_login_messages::version_8::*;

    let addr = get_world_server_string(&stream.peer_addr().unwrap().ip());

    while (tokio_expect_client_message::<CMD_REALM_LIST_Client, _>(&mut stream).await).is_ok() {
        let mut realms = Vec::new();
        for i in 0..9 {
            realms.push(Realm {
                realm_type: RealmType::PlayerVsEnvironment,
                locked: false,
                flag: Default::default(),
                name: i.to_string(),
                address: addr.clone(),
                population: Default::default(),
                number_of_characters_on_realm: i,
                category: RealmCategory::One,
                realm_id: i,
            })
        }

        CMD_REALM_LIST_Server { realms }
            .tokio_write(&mut stream)
            .await
            .unwrap();
        println!("Sent Version 8 Realm List");
    }
}
