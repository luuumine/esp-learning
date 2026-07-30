use alloc::format;
use alloc::string::String;
use esp_alloc as _;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::peripherals::Peripherals;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;

use esp_radio::wifi::{Config, ControllerConfig, Interface, WifiController, sta::StationConfig};
use esp_rtos::embassy::Executor;
use log::info;
use static_cell::StaticCell;

use embassy_net::IpListenEndpoint;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config as NetConfig, Runner, Stack, StackResources};

pub fn main(peripherals: Peripherals) -> ! {
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let red_light = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let yellow_light = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());

    const SSID: &str = env!("WIFI_SSID");
    const PASSWORD: &str = env!("WIFI_PASS");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    info!("Starting RTOS scheduler...");
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let sta_config = Config::Station(
        StationConfig::default()
            .with_ssid(SSID)
            .with_password(String::from(PASSWORD)),
    );

    let controller_config = ControllerConfig::default().with_initial_config(sta_config);

    info!("Initializing Wi-Fi hardware...");
    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, controller_config).unwrap();

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let net_config = NetConfig::dhcpv4(Default::default());
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    static STACK: StaticCell<Stack<'static>> = StaticCell::new();

    let (stack, runner) = embassy_net::new(
        interfaces.station,
        net_config,
        RESOURCES.init(StackResources::<3>::new()),
        seed,
    );
    let stack_ref = STACK.init(stack);

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(wifi_task(wifi_controller, SSID).unwrap());
        spawner.spawn(net_task(runner).unwrap());
        spawner.spawn(web_server_task(stack_ref, red_light, yellow_light).unwrap());
    });
}

#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>, ssid: &'static str) {
    loop {
        info!("Connecting to {}...", ssid);
        match controller.connect_async().await {
            Ok(_) => {
                info!("Radio connected to {}!", ssid);
                let _ = controller.wait_for_disconnect_async().await;
                info!("Wi-Fi connection lost! Reconnecting...");
            }
            Err(e) => info!("Failed to connect: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn web_server_task(
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

        info!("Browser connected!");

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

            let body = "<!DOCTYPE html><html><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>ESP32 Control</title></head><body style=\"text-align: center; font-family: sans-serif; background-color: #121212; color: white; margin-top: 50px;\"><h1>ESP32 Mission Control</h1><p><a href=\"/red/toggle\"><button style=\"font-size: 24px; padding: 20px 40px; margin: 10px; border-radius: 10px; border: none; background: #e74c3c; color: white; cursor: pointer;\">Toggle Red</button></a></p><p><a href=\"/yellow/toggle\"><button style=\"font-size: 24px; padding: 20px 40px; margin: 10px; border-radius: 10px; border: none; background: #f1c40f; color: black; cursor: pointer;\">Toggle Yellow</button></a></p></body></html>";

            // dynamically adds content length
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

        // wait for the browser to close its end of the connection
        // stops the socket from being dropped prematurely and sending a RST packet
        let mut trash = [0; 16];
        while let Ok(n) = socket.read(&mut trash).await {
            if n == 0 {
                break;
            }
        }
    }
}
