pub mod server;
pub mod wifi;

use alloc::string::String;
use embassy_net::{Config as NetConfig, Stack, StackResources};
use esp_alloc as _;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    peripherals::Peripherals,
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_radio::wifi::{Config, ControllerConfig, sta::StationConfig};
use esp_rtos::embassy::Executor;
use log::info;
use static_cell::StaticCell;

pub fn main(peripherals: Peripherals) -> ! {
    // head for dynamic strings and types
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let red_light = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let yellow_light = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());

    // get ssid and passwd from env vars at compile time
    const SSID: &str = env!("WIFI_SSID");
    const PASSWORD: &str = env!("WIFI_PASS");

    // for rtos scheduler
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

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    static STACK: StaticCell<Stack<'static>> = StaticCell::new();

    let (stack, runner) = embassy_net::new(
        interfaces.station,
        NetConfig::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::<3>::new()),
        seed,
    );
    let stack_ref = STACK.init(stack);

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        // Notice the .unwrap() is INSIDE the spawn parentheses!
        spawner.spawn(wifi::wifi_task(wifi_controller, SSID).unwrap());
        spawner.spawn(server::net_task(runner).unwrap());
        spawner.spawn(server::web_server_task(stack_ref, red_light, yellow_light).unwrap());
    });
}
