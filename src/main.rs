use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::iced::widget::text;
use cosmic::Element;
use std::time::Duration;
use notify_rust::Notification;
use std::process::Command as StdCommand;

mod fetch;

const ID: &str = "com.user.AuroraApplet";

pub fn main() -> cosmic::iced::Result {
    // Initialize logger
    let _ = env_logger::builder().filter_level(log::LevelFilter::Info).try_init();
    cosmic::applet::run::<AuroraApplet>(())
}

struct AuroraApplet {
    core: Core,
    kp_value: f32,
    threshold: f32,
    critical: f32,
    loading: bool,
}

#[derive(Clone, Debug)]
enum Message {
    Tick,
    // UpdateData,
    DataUpdated(Result<fetch::AuroraData, String>),
    OpenImages,
    // ShowData, // Removing for now if not used in view
    // Refresh,  // Removing for now if not used in view
}

impl Default for AuroraApplet {
    fn default() -> Self {
        Self {
            core: Core::default(),
            kp_value: 0.0,
            threshold: 3.99,
            critical: 6.99,
            loading: true,
        }
    }
}

impl cosmic::Application for AuroraApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let applet = AuroraApplet {
            core,
            ..Default::default()
        };
        (
            applet, 
            Task::perform(fetch_data_action(), |res| cosmic::Action::App(Message::DataUpdated(res)))
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                return Task::perform(fetch_data_action(), |res| cosmic::Action::App(Message::DataUpdated(res)));
            }
            /* Message::UpdateData => { // Re-using UpdateData for manual refresh if needed
                self.loading = true;
                return Task::perform(fetch_data_action(), |res| cosmic::Action::App(Message::DataUpdated(res)));
            } */
            Message::DataUpdated(res) => {
                self.loading = false;
                match res {
                    Ok(data) => {
                        self.kp_value = data.value;
                        
                        let _ = Notification::new()
                            .summary("Aurora Forecast Updated")
                            .body(&format!("New Kp: {}", self.kp_value))
                            .show();
                    }
                    Err(_e) => {
                        // Handle error?
                    }
                }
            }
            Message::OpenImages => {
                if let Ok(home) = std::env::var("HOME") {
                     let cache = format!("{}/.cache", home);
                     let viewline = format!("{}/aurora.png", cache);
                     let latest = format!("{}/aurora_latest.jpg", cache);
                     let kindex = format!("{}/aurora_kindex.png", cache);
                     
                     let _ = StdCommand::new("mpv")
                         .args(&["--title=Aurora", "--idle", "--autofit=50%", "--background=none", "--loop-playlist", "--image-display-duration=10"])
                         .arg(viewline)
                         .arg(latest)
                         .arg(kindex)
                         .spawn();
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let status_color = if self.kp_value > self.critical {
            cosmic::iced::Color::from_rgb(1.0, 0.0, 0.0) // Red
        } else if self.kp_value > self.threshold {
            cosmic::iced::Color::from_rgb(1.0, 0.5, 0.0) // Orange
        } else {
            cosmic::iced::Color::from_rgb(0.0, 0.8, 0.0) // Green
        };
        
        let label = if self.loading {
            "..."
        } else {
            "🌈" 
        };
        
        let text_element = text(format!("{} {}", label, self.kp_value))
            .size(14)
            .class(status_color);
            
        // Use iced button (standard)
        cosmic::iced::widget::button(text_element)
            .on_press(Message::OpenImages)
            .padding(4)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Use full path to avoid issues if imports are weird
        cosmic::iced::time::every(Duration::from_secs(15 * 60)).map(|_| Message::Tick)
    }
}

async fn fetch_data_action() -> Result<fetch::AuroraData, String> {
    fetch::update_data().await.map_err(|e| e.to_string())
}
