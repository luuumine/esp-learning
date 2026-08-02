use esp_radio::wifi::WifiController;
use log::info;

// background task to keep the wifi connected
// loops forever and auto-reconnects if the signal drops
#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>, ssid: &'static str) {
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
