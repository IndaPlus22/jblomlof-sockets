/**
 * Modified chat server example
 * --------
 * Chat Server Example
 *
 * Simple broadcast single-line text-only chat server.
 *
 * Author: Tensor-Programming, Viola Söderlund <violaso@kth.se>
 * Last updated: 2021-01-21
 *
 * See: https://github.com/tensor-programming/Rust_client-server_chat
 */
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;


mod user;

/* Address to server. */
const SERVER_ADDR: &str = "127.0.0.1:6000";

/* Max message size in characters. */
const MSG_SIZE: usize = 32;

/**
 * Sleep (current thread) for 100 milliseconds.
 */
fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    // connect to server
    let server = match TcpListener::bind(SERVER_ADDR) {
        Ok(_client) => {
            println!("Opened server at: {}", SERVER_ADDR);
            _client
        }
        Err(_) => {
            println!("Failed to connect to socket at: {}", SERVER_ADDR);
            std::process::exit(1)
        }
    };
    // prevent io stream operation from blocking sockets in case of slow communication
    server
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking!");

    let mut users = vec![];
    let mut user_id: usize = 1;
    // create channel for communication between threads
    let (sender, receiver) = mpsc::channel::<String>();

    loop {
        /* Start listening thread on new connecting client. */
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected.", addr);

            let _sender = sender.clone();

            println!("{}", user_id);
            users.push(user::User::new(
                &format!("Guest{}", user_id),
                user_id,
                socket
                    .try_clone()
                    .expect("Failed to clone client! Client wont receive messages!"),
            ));

            let user_for_thread = user_id;
            user_id += 1;
            thread::spawn(move || loop {
                let mut msg_buff = vec![0; MSG_SIZE];

                /* Read and relay message from client. */
                match socket.read_exact(&mut msg_buff) {
                    // received message
                    Ok(_) => {
                        let _msg = msg_buff
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect::<Vec<_>>();
                        let msg = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

                        println!("{}: {:?}", user_for_thread, msg);

                        _sender
                            .send(format!("{}:{}", user_for_thread, msg))
                            .expect("Failed to relay message!");
                    }
                    // no message in stream
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    // connection error
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        /* Broadcast incoming messages. */
        if let Ok(msg) = receiver.try_recv() {
            let msg_split = msg.split_once(':').unwrap();
            let mut username: String = String::from("NaN");
            let mut current_index = usize::MAX; // setting to max as a fail safe, altough not 100% correct. But will never have usize::MAX clients
            for i in 0..users.len() {
                if users[i].id == msg_split.0.parse::<usize>().unwrap() {
                    current_index = i;
                    username = users[i].username.clone();
                    break;
                }
            }

            if msg_split.1.starts_with('/') {
                //the message is a command
                //handle it.
                if current_index != usize::MAX {
                handle_command(&mut users, current_index, msg_split.1);
                } else {
                    println!("Couldnt do command, got to many USERS!!");
                }
            } else {

                let correct_msg = format!("{}: {}", username, msg_split.1);
                // send message to all clients
                users = users.into_iter().filter_map(|mut user| {
                    if user.id == msg_split.0.parse().unwrap() {
                        // we dont want to send the message back to the sender, just ignore it.
                        Some(user)
                    } else {
                        let mut msg_buff = correct_msg.clone().into_bytes();
                        // add zero character to mark end of message
                        msg_buff.resize(MSG_SIZE, 0);
                        user.client.write_all(&msg_buff).map(|_| user).ok()
                    }
                })
                .collect::<Vec<_>>();
            }
        }
        // so this is also a cleaner ^^
        // it tries to send messages to clients, if there is an error (when writing) the error is turned to a None
        // which in turn is ignored by filter_map()

        sleep();
    }

    fn handle_command(_users: &mut Vec<user::User>, index: usize, command: &str) {
        let split_commands: Vec<&str> = command.split_whitespace().collect();
        match split_commands[0] {
            "/login" => {
                _users[index].username = split_commands[1].to_string();
            },
            "/ping" => {
                let mut pong = "pong".to_string().into_bytes();
                pong.resize(MSG_SIZE, 0);
                _users[index].client.write_all(&pong);
            },
            "/aboutme" => {
                let mut msg = format!("Username: {}, ID: {}", _users[index].username, _users[index].id).into_bytes();
                msg.resize(MSG_SIZE, 0);
                _users[index].client.write_all(&msg);
            }
            _ => println!("Something went wrong. We need to uptade our list of commands.")
        }

    }
}
