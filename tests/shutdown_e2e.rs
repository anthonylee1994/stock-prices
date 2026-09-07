#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn serves_http_and_exits_cleanly_on_sigterm_and_sigint() {
    for signal in ["-TERM", "-INT"] {
        let reservation = TcpListener::bind("127.0.0.1:0").expect("reserve test port");
        let port = reservation.local_addr().expect("test address").port();
        drop(reservation);
        let mut server = Server(
            Command::new(env!("CARGO_BIN_EXE_stock-prices"))
                .env("PORT", port.to_string())
                .stdout(Stdio::piped())
                .spawn()
                .expect("start server"),
        );
        let stdout = server.0.stdout.take().expect("server stdout");
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            BufReader::new(stdout).read_line(&mut line).expect("startup line");
            let _ = sender.send(line);
        });
        let startup = receiver.recv_timeout(Duration::from_secs(5)).expect("server starts promptly");
        assert!(startup.contains("Server is running"), "{startup}");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect server");
        stream.set_read_timeout(Some(Duration::from_secs(5))).expect("read timeout");
        stream.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").expect("request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("response");
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert!(response.contains("Stock Prices API"));

        assert!(Command::new("kill").args([signal, &server.0.id().to_string()]).status().expect("send signal").success());
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = server.0.try_wait().expect("poll child") {
                break status;
            }
            assert!(Instant::now() < deadline, "shutdown timed out");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success(), "{signal} should shut down cleanly: {status}");
    }
}
