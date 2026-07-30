use alloc::format;
use embassy_net::{IpListenEndpoint, Runner, Stack, tcp::TcpSocket};
use esp_hal::gpio::Output;
use esp_radio::wifi::Interface;
use log::info;

// background task handling incoming and outgoing network packets
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}

// background task running http server on port 80
#[embassy_executor::task]
pub async fn web_server_task(
    stack: &'static Stack<'static>,
    mut red_light: Output<'static>,
    mut yellow_light: Output<'static>,
) {
    info!("Waiting for DHCP to assign an IP address...");
    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        info!(
            "Success! Connect your browser to: http://{}",
            config.address.address()
        );
    }

    let mut rx_buffer = [0; 2048];
    let mut tx_buffer = [0; 2048];

    loop {
        let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);

        if let Err(e) = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 80,
            })
            .await
        {
            info!("Accept error: {:?}", e);
            continue;
        }

        let mut buf = [0; 1024];
        if let Ok(n) = socket.read(&mut buf).await {
            let request = core::str::from_utf8(&buf[..n]).unwrap_or("");

            if request.contains("GET /red/toggle") {
                red_light.toggle();
                info!("Toggled Red LED!");
            } else if request.contains("GET /yellow/toggle") {
                yellow_light.toggle();
                info!("Toggled Yellow LED!");
            }

            let body = include_str!("index.html");

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );

            let mut bytes_written = 0;
            let payload = response.as_bytes();

            while bytes_written < payload.len() {
                if let Ok(written) = socket.write(&payload[bytes_written..]).await {
                    if written == 0 {
                        break;
                    }
                    bytes_written += written;
                } else {
                    break;
                }
            }
            let _ = socket.flush().await;
        }

        socket.close();

        let mut trash = [0; 16];
        while let Ok(n) = socket.read(&mut trash).await {
            if n == 0 {
                break;
            }
        }
    }
}
