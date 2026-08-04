pub mod server;
pub mod wifi;

use alloc::string::String;
use embassy_net::{Config as NetConfig, Stack, StackResources};
use esp_alloc as _;
use esp_hal::{
    gpio::{self, interconnect::PeripheralOutput},
    interrupt::software::SoftwareInterruptControl,
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace, Number},
        timer::{self, Timer, TimerIFace},
    },
    peripherals::Peripherals,
    rng::Rng,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_radio::wifi::{Config, ControllerConfig, sta::StationConfig};
use esp_rtos::embassy::Executor;
use log::info;
use static_cell::StaticCell;

pub struct RGB<'a> {
    pub red: Channel<'a, LowSpeed>,
    pub green: Channel<'a, LowSpeed>,
    pub blue: Channel<'a, LowSpeed>,
}

fn configure_channel<'a>(
    ledc: &'a Ledc<'a>,
    number: channel::Number,
    pin: impl PeripheralOutput<'a>,
    timer: &'a Timer<'a, LowSpeed>,
) -> Channel<'a, LowSpeed> {
    let mut ch = ledc.channel(number, pin);

    ch.configure(channel::config::Config {
        timer,
        duty_pct: 0,
        drive_mode: gpio::DriveMode::PushPull,
    })
    .unwrap();

    ch
}

pub fn main(peripherals: Peripherals) -> ! {
    // heap for dynamic strings and types
    esp_alloc::heap_allocator!(size: 72 * 1024);

    static LEDC: StaticCell<Ledc<'static>> = StaticCell::new();
    let ledc = LEDC.init(Ledc::new(peripherals.LEDC));
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let red_pin = peripherals.GPIO2;
    let green_pin = peripherals.GPIO4;
    let blue_pin = peripherals.GPIO5;

    static TIMER0: StaticCell<Timer<'static, LowSpeed>> = StaticCell::new();
    let timer0 = TIMER0.init(ledc.timer::<LowSpeed>(timer::Number::Timer0));

    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty8Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(5),
        })
        .unwrap();

    let rgb = RGB {
        red: configure_channel(ledc, Number::Channel0, red_pin, timer0),
        green: configure_channel(ledc, Number::Channel1, green_pin, timer0),
        blue: configure_channel(ledc, Number::Channel2, blue_pin, timer0),
    };

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
        spawner.spawn(server::web_server_task(stack_ref, rgb).unwrap());
    });
}
