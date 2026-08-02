use alloc::format;
use embassy_net::{IpListenEndpoint, Runner, Stack, tcp::TcpSocket};
use esp_hal::ledc::channel::ChannelIFace;
use esp_radio::wifi::Interface;
use log::info;

use crate::circuit_6_rgb_led::RGB;

// background task handling incoming and outgoing network packets
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}

// background task running http server on port 80
#[embassy_executor::task]
pub async fn web_server_task(stack: &'static Stack<'static>, rgb: RGB<'static>) {
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
            let mut is_update = false;

            let string = "GET /?color=";
            if let Some(start) = request.find(string) {
                let hex_start = start + string.len();
                if request.len() >= hex_start + 6 {
                    let hex = &request[hex_start..hex_start + 6];

                    if let Ok(color_val) = u32::from_str_radix(hex, 16) {
                        let r = ((((color_val >> 16) & 0xFF) as u32) * 100 / 255) as u8;
                        let g = ((((color_val >> 8) & 0xFF) as u32) * 100 / 255) as u8;
                        let b = (((color_val & 0xFF) as u32) * 100 / 255) as u8;

                        info!("got rgb {} {} {}", r, g, b);

                        let _ = rgb.red.set_duty(r);
                        let _ = rgb.green.set_duty(g);
                        let _ = rgb.blue.set_duty(b);

                        is_update = true;
                    }
                }
            }

            // If it's a background fetch from the sliders, just return 204 No Content to save time.
            let response = if is_update {
                format!("HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
            } else {
                let body = include_str!("index.html");
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
            };

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
