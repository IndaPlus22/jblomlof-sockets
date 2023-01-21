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
const MSG_SIZE: usize = 32;

fn login(ask_for_confirmation: bool) -> Option<(String, String)>{

    if ask_for_confirmation {
        let mut input: String = String::new();
        println!("Do you want to log in: (y/n)");
        std::io::stdin().lock().read_line(&mut input).expect("Could not read!");
        if input.trim() != "y" {
            return None;
        }
    }

    // This is not secure in anyway but anyways
    // Getting username and password from user
    let mut lines = std::io::stdin().lines();
    println!("Write your username: (\":cancel\" to stop logging in)");
    let username = lines.next().unwrap().unwrap().trim().to_lowercase();
    if username == ":cancel" {
        println!("Canceled log in.");
        return None;
    }
    println!("Write your password: (\":cancel\" to stop logging in)");
    let password  = lines.next().unwrap().unwrap().trim().to_lowercase();
    if password == ":cancel" {
        println!("Canceled log in.");
        return None;
    }

    if (!username.is_ascii()) || (!password.is_ascii()) {
        println!("Failed to log in. Use ascii next time!");
        return None;
    }

    Some((username, password))
}

fn main() {

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
                println!("Lost connection with server!");
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
                    println!("Failed to send message!")
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
    if let Some((username, password)) = login(true) {
        if sender.send(format!("/login {} {}", username, password)).is_err() {
            println!("Couldn't establish connection");
            std::process::exit(1)
        };
    }

    /* Listen for and act on user messages. */
    println!("Chat open:");
    loop {
        let mut msg_buffer = String::new();

        // wait for user to write message
        io::stdin().read_line(&mut msg_buffer).expect("Failed to read user message!");

        let msg = msg_buffer.trim().to_string();

        if msg.starts_with('/') {
            let message_split: Vec<&str> = msg.split_ascii_whitespace().collect();
            match message_split[0] {
                "/login" => {
                    if message_split.len() == 3 {
                        if sender.send(msg).is_err() {break};
                    } else {
                        if let Some((username, password)) = login(false) {
                            if sender.send(format!("/login {} {}", username, password)).is_err() {break};
                        }
                    }
                },

                "/ping" => {if sender.send(msg).is_err() {break};},
                "/aboutme" => {if sender.send(msg).is_err() {break};},

                _ => {println!("Unkown command!")}
                
            }
        } else {

            // quit on message ":quit" or on connection error
            if msg == ":quit" || sender.send(msg).is_err() {break}
        }
    }

    println!("Closing chat...");
}