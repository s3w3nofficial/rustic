// use std::{collections::HashMap, sync::Arc};
// use async_std::task;
// use rustic_proxy::SharedData;

// // use num_cpus;

// #[async_std::main]
// async fn main() -> Result<(), std::io::Error> {
//     //let cpu_count = num_cpus::get(); // Get the number of available CPUs
//     let cpu_count = 2;
//     let mut tasks = Vec::with_capacity(cpu_count); // Create a vector to hold the threads

//     let data = Arc::new(SharedData::new());

//     let mut d = HashMap::<String, String>::new();
//     d.insert(String::from("/abcd"), String::from("http://localhost:8002"));

//     data.write(d);
    
//     // Spawn a thread for each CPU
//     for i in 0..cpu_count {
//         let data = data.clone();
//         let i = i.clone();
//         let task = task::spawn(async move {

//             let app = rustic::new();

//             println!("Thread {} running", i);
//             for (source, destination) in data.read() {
//                 println!("source: {} -> destination: {}", source, destination);
//             }

//             app.listen("127.0.0.1:8080").await.unwrap();
//         });
//         tasks.push(task);
//     }

//     // Wait for all threads to finish
//     futures::future::join_all(tasks).await;

//     Ok(())
// }

use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use libc::{c_int, setsockopt, SOL_SOCKET, SO_REUSEADDR, SO_REUSEPORT, sockaddr_in, bind, listen};
use std::mem;
use std::os::fd::FromRawFd;
use std::os::unix::io::{AsRawFd, RawFd};

async fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    // Handle the client connection
    println!("{}", stream.peer_addr().unwrap());

    Ok(())
}

async fn start_server(port: u16) -> std::io::Result<()> {
    let listener = create_listener(port).await?;
    let mut incoming = listener.incoming();
    
    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(stream) => {
                // Spawn a new task to handle the connection

                let socket_fd = listener.as_raw_fd();
                println!("Socket file descriptor number: {}", socket_fd);

                task::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn create_listener(port: u16) -> std::io::Result<TcpListener> {

    let sockfd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    if sockfd == -1 {
        panic!("Error creating socket");
    }

    let mut serv_addr: sockaddr_in = unsafe { mem::zeroed() };
    serv_addr.sin_family = libc::AF_INET as u16;
    serv_addr.sin_port = socket::htons(port);
    serv_addr.sin_addr.s_addr = socket::htonl(libc::INADDR_ANY as u32);

    let optval: c_int = 1;
    unsafe {
        setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &optval as *const _ as *const _, std::mem::size_of_val(&optval) as u32);
        setsockopt(sockfd, SOL_SOCKET, SO_REUSEPORT, &optval as *const _ as *const _, std::mem::size_of_val(&optval) as u32);
    }

    let status = unsafe { bind(sockfd, &serv_addr as *const _ as *const _, mem::size_of_val(&serv_addr) as u32) };
    if status == -1 {
        panic!("Error binding socket to address");
    }

    let status = unsafe { listen(sockfd, 5) };
    if status == -1 {
        panic!("Error listening for incoming connections");
    }

    println!("Startint tcp listener on socket: {}", sockfd);

    let listener = unsafe { TcpListener::from_raw_fd(sockfd) };
    //set_reuseaddr(&listener)?;
    Ok(listener)
}

fn set_reuseaddr(listener: &TcpListener) -> std::io::Result<()> {
    let fd = listener.as_raw_fd();
    let optval: c_int = 1;
    unsafe {
        setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &optval as *const _ as *const _, std::mem::size_of_val(&optval) as u32);
        setsockopt(fd, SOL_SOCKET, SO_REUSEPORT, &optval as *const _ as *const _, std::mem::size_of_val(&optval) as u32);
    }
    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let port = 8080;

    let mut tasks = Vec::with_capacity(2);

    let task1 = start_server(port);
    let task2 = start_server(port);

    tasks.push(task1);
    tasks.push(task2);

    futures::future::join_all(tasks).await;

    Ok(())
}
