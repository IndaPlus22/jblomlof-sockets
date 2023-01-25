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
const MSG_SIZE: usize = 64;

/* File where user login is stored */
const FILE_OF_USERS: &str = "user_file.txt";
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
            println!("SERVER: Opened server at: {}", SERVER_ADDR);
            _client
        }
        Err(_) => {
            println!("SERVER: Failed to connect to socket at: {}", SERVER_ADDR);
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

    //making a thread to listen to input
    let _sen = sender.clone();
    thread::spawn(move || loop {
        let mut std_input = String::new();
        std::io::stdin()
            .read_line(&mut std_input)
            .expect("Couldn't read stdin");
        let command = std_input.trim();
        if is_command(command) {
            _sen.send(format!("0:/{}", command))
                .expect("SERVER: Couldn't relay message to main.");
        }
        sleep();
    });

    // accessing the file and load the contents to mem
    let mut file_contents = load_file_data_to_string();

    loop {
        /* Start listening thread on new connecting client. */
        if let Ok((mut socket, addr)) = server.accept() {
            println!("SERVER: Client {} connected.", addr);

            let _sender = sender.clone();
            users.push(user::User::new(
                &format!("Guest{}", user_id),
                user_id,
                socket
                    .try_clone()
                    .expect("SERVER: Failed to clone client! Client wont receive messages!"),
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
                        println!("SERVER: Closing connection with: {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        /* Broadcast incoming messages. */
        // and handle admin input from stdin.
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
                handle_command(&mut users, current_index, msg_split.1, &mut file_contents);
            } else {
                let correct_msg = format!("{}: {}", username, msg_split.1);
                // send message to all clients
                users = users
                    .into_iter()
                    .filter_map(|mut user| {
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
}

fn handle_command(
    _users: &mut Vec<user::User>,
    index: usize,
    command: &str,
    _file_contents: &mut String,
) {
    let split_commands: Vec<&str> = command.split_whitespace().collect();
    match split_commands[0] {
        "/whisper" => {
            for _inner_index in 0.._users.len() {
                if _users[_inner_index].username == split_commands[1] {
                    let mut msg = format!(
                        "{} whispered: {}",
                        _users[index].username,
                        split_commands[2..split_commands.len()].join(" ")
                    )
                    .into_bytes();
                    msg.resize(MSG_SIZE, 0);
                    _users[_inner_index].client.write_all(&msg);
                    return;
                }
            }
            let mut msg = String::from("Couldn't find user.").into_bytes();
            msg.resize(MSG_SIZE, 0);
            _users[index].client.write_all(&msg);
        }
        "/listall" => {
            let mut msg = String::from("All users currently online listed:").into_bytes();
            msg.resize(MSG_SIZE, 0);
            _users[index].client.write_all(&msg);
            for _inner_index in 0.._users.len() {
                msg = _users[_inner_index].username.clone().into_bytes();
                msg.resize(MSG_SIZE, 0);
                _users[index].client.write_all(&msg);
            }
        }
        "/login" => {
            if index >= _users.len() {
                println!("SERVER: ABORT COMMAND");
                return;
            }
            let mut message = {
                if account_exists(&_file_contents, split_commands[1], split_commands[2]).1 {
                    _users[index].username = split_commands[1].to_string();
                    format!("Welcome back {}", _users[index].username)
                } else {
                    "Log in failed. Incorrect username/password.".to_string()
                }
            }
            .into_bytes();
            message.resize(MSG_SIZE, 0);
            _users[index].client.write_all(&message);
        }
        "/create" => {
            if index >= _users.len() {
                println!("SERVER: ABORT COMMAND");
                return;
            }
            let mut message = {
                if account_exists(&_file_contents, split_commands[1], "NaN").0 {
                    "Account already exists".to_string()
                } else {
                    create_user(_file_contents, split_commands[1], split_commands[2]);
                    _users[index].username = split_commands[1].to_string();
                    format!("Welcome {}. Account created.", _users[index].username)
                }
            }
            .into_bytes();
            message.resize(MSG_SIZE, 0);
            _users[index].client.write_all(&message);
        }
        "/ping" => {
            if index >= _users.len() {
                println!("SERVER: ABORT COMMAND");
                return;
            }
            let mut pong = "pong".to_string().into_bytes();
            pong.resize(MSG_SIZE, 0);
            _users[index].client.write_all(&pong);
        }
        "/aboutme" => {
            if index >= _users.len() {
                println!("SERVER: ABORT COMMAND");
                return;
            }
            let mut msg = format!(
                "Username: {}, ID: {}",
                _users[index].username, _users[index].id
            )
            .into_bytes();
            msg.resize(MSG_SIZE, 0);
            _users[index].client.write_all(&msg);
        }
        "/stop" => {
            //server wants to shutdown
            println!("SERVER: Shutting down...");
            save_file_data(&_file_contents);
            let mut msg = String::from("Server is closing. No messages will be sent.").into_bytes();
            msg.resize(MSG_SIZE, 0);
            for _user in _users {
                _user.client.write_all(&msg);
            }
            thread::sleep(std::time::Duration::from_secs(1));
            std::process::exit(0);
        }
        _ => println!("SERVER: Something went wrong. We need to uptade our list of commands."),
    }
}

//we need to check if the input in server is a command. All commands from clients are already checked client side.
fn is_command(command: &str) -> bool {
    match command {
        "stop" => true,
        _ => false,
    }
}

/**
 * Returns a tuple
 *
 * (true, true) if account exists and password is correct
 *
 * (true, false) if account only exists
 *
 */
fn account_exists(_file: &str, _username: &str, _password: &str) -> (bool, bool) {
    for line in _file.lines() {
        for subsection in line.split(';') {
            let (field, value) = subsection.split_once('=').unwrap_or(("0", "0"));
            if field == "username" && value == _username {
                //found the correct user check for password of this user
                for sub in line.split(';') {
                    let (f, v) = sub.split_once('=').unwrap_or(("0", "0"));
                    if f == "password" && v == _password {
                        return (true, true);
                    }
                }
                return (true, false);
            }
        }
    }
    (false, false)
}

//adding the user to the string that contains the users.
fn create_user(_file_contents: &mut String, _username: &str, _password: &str) {
    let line = format!("\nusername={};password={}", _username, _password);
    _file_contents.push_str(&line);
}

//Outsourcing to make readable.
fn load_file_data_to_string() -> String {
    std::fs::read_to_string(FILE_OF_USERS).unwrap_or("NaN".to_string())
}

//writing the "global" string to a file.
fn save_file_data(_file_contents: &str) {
    let mut file = std::fs::File::create(FILE_OF_USERS).unwrap();
    file.write_all(_file_contents.as_bytes());
}
