/**
 * Modified chat client example
 * Example provided by Viola Söderlund
 * ------
 * Chat Client Example
 * 
 * Simple broadcast single-line text-only chat client. 
 * 
 * Author: Tensor-Programming, Viola Söderlund <violaso@kth.se>
 * Last updated: 2021-01-21
 * 
 * See: https://github.com/tensor-programming/Rust_client-server_chat
 */

use std::io::{self, ErrorKind, Read, Write, BufRead};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

/* Address to server. */
const SERVER_ADDR: &str = "127.0.0.1:6000";

/* Max message size in characters. */
const MSG_SIZE: usize = 64;

//Im to lazy this function was originally to get login,
//but now its both log in and create user
fn get_user_name_and_password(ask_for_confirmation: bool) -> Option<(String, String)>{

    if ask_for_confirmation {
        let mut input: String = String::new();
        println!("SYSTEM: Do you want to log in: (y/n)");
        std::io::stdin().lock().read_line(&mut input).expect("Could not read!");
        if input.trim() != "y" {
            return None;
        }
    }

    // This is not secure in anyway but anyways
    // Getting username and password from user
    let mut lines = std::io::stdin().lines();
    println!("SYSTEM: Write your username: (\":cancel\" to stop)");
    let username = lines.next().unwrap().unwrap().trim().to_lowercase();
    if username == ":cancel" {
        println!("SYSTEM: Canceled process.");
        return None;
    }
    println!("SYSTEM: Write your password: (\":cancel\" to stop)");
    let password  = lines.next().unwrap().unwrap().trim().to_lowercase();
    if password == ":cancel" {
        println!("SYSTEM: Canceled process.");
        return None;
    }

    if (!username.is_ascii()) || (!password.is_ascii()) {
        println!("SYSTEM: Failed process. Use ascii next time!");
        return None;
    }

    Some((username, password))
}

fn main() {
    // calling username info before connection to server.
    let pre_call_function = get_user_name_and_password(true);

    // connect to server
    let mut client = match TcpStream::connect(SERVER_ADDR) {
        Ok(_client) => {
            println!("Connected to server at: {}", SERVER_ADDR);
            _client
        },
        Err(_) => {
            println!("Failed to connect to server at: {}", SERVER_ADDR);
            std::process::exit(1)
        }
    };
    // prevent io stream operation from blocking socket in case of slow communication
    client.set_nonblocking(true).expect("Failed to initiate non-blocking!");

    // create channel for communication between threads
    let (sender, receiver) = mpsc::channel::<String>();

    /* Start thread that listens to server. */
    thread::spawn(move || loop {
        let mut msg_buffer = vec![0; MSG_SIZE];

        /* Read message from server. */
        match client.read_exact(&mut msg_buffer) {
            // received message
            Ok(_) => {
                // read until end-of-message (zero character)
                let _msg = msg_buffer
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                let msg = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

                println!("{}", msg);
            },
            // no message in stream
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            // connection error
            Err(_) => {
                println!("SYSTEM: Lost connection with server!");
                break;
            }
        }

        /* Send message in channel to server. */
        match receiver.try_recv() {
            // received message from channel
            Ok(msg) => {
                let mut msg_buffer = msg.clone().into_bytes();
                // add zero character to mark end of message
                msg_buffer.resize(MSG_SIZE, 0);

                if client.write_all(&msg_buffer).is_err() {
                    println!("SYSTEM: Failed to send message!")
                }
            }, 
            // no message in channel
            Err(TryRecvError::Empty) => (),
            // channel has been disconnected (main thread has terminated)
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    //ask for login as we start the client
    //precalling before server connection. So that it doesn't recieve messages during the function call.
    if let Some((username, password)) = pre_call_function {
        if sender.send(format!("/login {} {}", username, password)).is_err() {
            println!("SYSTEM: Couldn't send login");
            std::process::exit(1)
        };
    }

    /* Listen for and act on user messages. */
    println!("Chat open:");
    loop {
        let mut msg_buffer = String::new();

        // wait for user to write message
        io::stdin().read_line(&mut msg_buffer).expect("Failed to read user message!");

        let mut msg = msg_buffer.trim().to_string();

        if msg.starts_with('/') {
            //handle command
            if let Some(_msg)  = check_command(msg) {
                msg = _msg;
            }
            else {
                //command was not valid, dont send to server
                continue;
            }
        }

        // quit on message ":quit" or on connection error
        if msg == ":quit" || sender.send(msg).is_err() {break}        
    }

    println!("Closing chat...");
}

//Returns the valid command. So if the input is already valid, no diff. It can fix a command that is lacking args.
fn check_command(_message: String) -> Option<String>{
    
    let message_split: Vec<&str> = _message.split_ascii_whitespace().collect();
    match message_split[0] {
        "/whisper" => {
            if message_split.len() >= 3 {
                return Some(_message);
            } else {
                println!("SYSTEM: WRONG FORMAT. USE /whisper <user> <message>");
            }
        }
        "/login" => {
            if message_split.len() == 3 {
                return Some(_message);
            } else {
                if let Some((username, password)) = get_user_name_and_password(false) {
                    return Some(format!("/login {} {}", username, password));
                }
            }
        }
        "/create" => {
            if message_split.len() == 3 {
                return Some(_message);
            } else {
                if let Some((username, password)) = get_user_name_and_password(false) {
                    return Some(format!("/create {} {}", username, password));
                }
            }
        }

        //no length checks are needed, since server doesnt do anything with it.
        "/ping" => return Some(_message),
        "/aboutme" => return Some(_message),
        "/listall" => return  Some(_message),

        _ => println!("Unkown command!")   
    }
    None
}