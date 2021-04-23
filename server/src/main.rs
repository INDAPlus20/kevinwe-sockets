use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

/* Address to server. */
const SERVER_ADDR: &str = "127.0.0.1:7777";

/* Max message size in characters. */
const MSG_SIZE: usize = 32;

fn main(){
    //binding server to port
    let server = match TcpListener::bind(SERVER_ADDR) {
        Ok(_client) => {
            println!("Opened server at: {}", SERVER_ADDR);
            _client
        },
        Err(_) => {
            println!("Failed to connect to socket at: {}", SERVER_ADDR);
            std::process::exit(1)
        }
    };
    server.set_nonblocking(true).expect("Failed to initiate non-blocking!");

    let mut clients = vec![];

    // create channel for communication between threads
    let (sender, receiver) = mpsc::channel::<String>();

    loop {
        /* Start listening thread on new connecting client. */
        if let Ok((mut socket, addr)) = server.accept() {

            println!("Client {} connected. Client no. {}", addr, clients.len());

            let _sender = sender.clone();

            clients.push(
                socket.try_clone().expect("Failed to clone client! Client wont receive messages!"));

            socket.write(&[(clients.len()-1) as u8]);
            thread::spawn(move || loop {

                let mut msg_buff = vec![0; MSG_SIZE];

                /* Read and relay message from client. */
                match socket.read_exact(&mut msg_buff) {
                    // received message
                    Ok(_) => {
                        let _msg = msg_buff
                            .into_iter()
                            .take_while(|&x| x != 255)
                            .collect::<Vec<_>>();
                        let msg = String::from_utf8(_msg).expect("Invalid UTF-8 message!");

                        println!("{}: {:?}", addr, msg);
                        _sender.send(msg).expect("Failed to relay message!");
                    }, 
                    // no message in stream
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    // connection error
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                thread::sleep(::std::time::Duration::from_millis(1000));
            });
        }
        /* Broadcast incoming messages. */
        
        if let Ok(mut msg) = receiver.try_recv() {
            println!("{}", msg);
            let client_return = msg.chars().nth(0).unwrap().to_digit(10).unwrap() as usize;
            msg.remove(0);
            // private message functionality
            if msg.starts_with("send "){
                // if the message starts with a number < the amount of clients, send it to that specific client
                let mut client_target = msg.chars().nth(5).unwrap().to_digit(10).unwrap_or(25565) as usize;
                if client_target > clients.len()-1{
                    client_target = 25565;
                }
                let mut msg_buff = msg.clone().into_bytes();
                if client_target == 25565{
                    msg_buff = format!("Error! That is not a connected client!").into_bytes();
                    msg_buff.resize(MSG_SIZE, 255);
                    clients[client_return].write_all(&msg_buff);
                }   else{
                    // add zero character to mark end of message
                    msg_buff.resize(MSG_SIZE, 255);
                    clients[client_target].write_all(&msg_buff); //result must be used?
                }
                
            }
            // user query to see amount of connected clients
            else if msg.starts_with("connections"){
                let mut msg_buff = format!("{} connected clients.", clients.len()).into_bytes();
                // add zero character to mark end of message
                msg_buff.resize(MSG_SIZE, 255);
                clients[client_return].write_all(&msg_buff); //result must be used?
            }
            
            // broadcast functionality
            // send message to all clients
            else {
                println!("else");
                clients = clients.into_iter().filter_map(|mut client| {
                let mut msg_buff = msg.clone().into_bytes();
                // add zero character to mark end of message
                msg_buff.resize(MSG_SIZE, 255);
                client.write_all(&msg_buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
            }
        }
        thread::sleep(::std::time::Duration::from_millis(1000));
    }
}