use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::iced::widget::text;
use cosmic::Element;
use std::time::Duration;
use notify_rust::Notification;
use std::process::Command as StdCommand;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{list_column, settings};

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
    popup: Option<Id>,
}

#[derive(Clone, Debug)]
enum Message {
    Tick,
    // UpdateData,
    DataUpdated(Result<fetch::AuroraData, String>),
    OpenImages,
    ShowData, // Re-enabling for popup
    Refresh,  // Re-enabling for popup
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
}

impl Default for AuroraApplet {
    fn default() -> Self {
        Self {
            core: Core::default(),
            kp_value: 0.0,
            threshold: 3.99,
            critical: 6.99,
            loading: true,
            popup: None,
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
            Message::Refresh => {
                self.loading = true;
                return Task::perform(fetch_data_action(), |res| cosmic::Action::App(Message::DataUpdated(res)));
            }
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
            Message::ShowData => {
                 if let Ok(home) = std::env::var("HOME") {
                     let forecast = format!("{}/.cache/aurora.txt", home);
                     let editor = std::env::var("EDITOR").unwrap_or("nano".to_string());
                     let _ = StdCommand::new("gnome-terminal")
                         .arg("--")
                         .arg(&editor)
                         .arg(&forecast)
                         .spawn();
                 }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
        }
        Task::none()
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
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
        
        // We need to clone specific values to move into closures if needed, 
        // but for now relying on Copy types or references.
        let have_popup = self.popup.clone();
        
        // Use button_from_element to get a button that supports on_press_with_rectangle
        let btn = self.core.applet.button_from_element(
            text(format!("{} {}", label, self.kp_value))
                .size(14)
                .class(status_color),
            false // use_symbolic_size
        )
        .padding(4)
        .on_press_with_rectangle(move |offset, bounds| {
             if let Some(id) = have_popup {
                 Message::Surface(destroy_popup(id))
             } else {
                 Message::Surface(app_popup::<AuroraApplet>(
                     move |state: &mut AuroraApplet| {
                         let new_id = Id::unique();
                         state.popup = Some(new_id);
                         let mut popup_settings = state.core.applet.get_popup_settings(
                             state.core.main_window_id().unwrap(),
                             new_id,
                             None,
                             None,
                             None,
                         );
                         
                         popup_settings.positioner.anchor_rect = Rectangle {
                            x: (bounds.x - offset.x) as i32,
                            y: (bounds.y - offset.y) as i32,
                            width: bounds.width as i32,
                            height: bounds.height as i32,
                        };
                        popup_settings
                     },
                     Some(Box::new(move |state: &AuroraApplet| {
                         let content = list_column()
                             .padding(5)
                             .spacing(0)
                             .add(settings::item(
                                 "Forecast",
                                 text(format!("Kp Index: {}", state.kp_value))
                             ))
                             .add(
                                 cosmic::widget::button::text("Open Images")
                                    .on_press(Message::OpenImages)
                                    .width(Length::Fill)
                             )
                             .add(
                                 cosmic::widget::button::text("Raw Data")
                                    .on_press(Message::ShowData)
                                    .width(Length::Fill)
                             )
                             .add(
                                 cosmic::widget::button::text("Refresh")
                                    .on_press(Message::Refresh)
                                    .width(Length::Fill)
                             );
                             
                         Element::from(state.core.applet.popup_container(content))
                            .map(cosmic::Action::App)
                     }))
                 ))
             }
        });
        
        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            "Aurora Forecast",
            self.popup.is_some(),
            |a| Message::Surface(a),
            None,
        ))
    }

    fn subscription(&self) -> Subscription<Message> {
        // Use full path to avoid issues if imports are weird
        cosmic::iced::time::every(Duration::from_secs(15 * 60)).map(|_| Message::Tick)
    }
}

async fn fetch_data_action() -> Result<fetch::AuroraData, String> {
    fetch::update_data().await.map_err(|e| e.to_string())
}
