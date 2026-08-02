use alloc::format;
use embassy_net::{IpListenEndpoint, Runner, Stack, tcp::TcpSocket};
use embassy_time::{Duration, with_timeout};
use esp_hal::gpio::{Input, Level, Output};
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
    button: Input<'static>,
    mut led: Output<'static>,
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

    let mut led_state = false;
    let mut prev_button_level = Level::Low;

    const POLL_LIMIT: Duration = Duration::from_millis(50);

    loop {
        let button_level = button.level();

        if button_level != prev_button_level {
            if button_level == Level::High {
                led.toggle();
                led_state = !led_state;
                info!("toggled led via button");
            }
            prev_button_level = button_level;
        }

        let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);

        if let Err(_) = with_timeout(
            POLL_LIMIT,
            socket.accept(IpListenEndpoint {
                addr: None,
                port: 80,
            }),
        )
        .await
        {
            socket.close();
            continue;
        }

        // if we get here, there was a request made!

        let mut buf = [0; 1024];
        if let Ok(n) = socket.read(&mut buf).await {
            let request = core::str::from_utf8(&buf[..n]).unwrap_or("");

            let response = if request.contains("GET /toggle") {
                led.toggle();
                led_state = !led_state;
                info!("toggled led via web");
                format!("HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
            } else if request.contains("GET /state") {
                let state_str = if led_state { "1" } else { "0" };
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                    state_str
                )
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
