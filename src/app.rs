// SPDX-License-Identifier: GPL-3.0-only

use crate::config::Config;
use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Length;
use cosmic::iced::{window::Id, Limits, Subscription};
use cosmic::iced_widget::{column, container, row, scrollable};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic_ext_applet_dict::{fetch_words, Entry};

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    search_text: String,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateConfig(Config),
    Search(String),
    Random,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "dev.cappsy.CosmicExtAppletDict";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button_from_handle(
                cosmic::widget::icon::from_svg_bytes(include_bytes!(
                    "../resources/icons/hicolor/scalable/apps/icon.svg"
                ))
                .symbolic(true),
            )
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut content_list = widget::column().padding(8).spacing(0);

        // quick search
        content_list = content_list.push(
            container(row!(widget::text_input(
                fl!("search"),
                self.search_text.clone()
            )
            .on_input(move |value| Message::Search(value.clone()))
            .width(Length::Fill),))
            .padding([8, 8, 10, 8]),
        );

        // build entries in scrollable list
        let mut entries_list = widget::column().padding(8).spacing(0);

        for entry in &self.entries {
            let mut entry_content = column!(
                widget::text::title4(&entry.word),
                widget::text(&entry.wordtype),
            );

            let mut def_i = 1;
            for def in &entry.defs {
                entry_content = entry_content.push(widget::text(format!("{}. {}", def_i, def)));
                def_i += 1;
            }

            entries_list = entries_list.push(entry_content.padding([10, 10, 15, 10]));
            entries_list = entries_list.push(widget::divider::horizontal::default());
        }
        entries_list = entries_list.push(
            container(
                widget::button::text("Search above or click for something random")
                    .on_press(Message::Random)
                    .width(Length::Fill)
                    .padding(5),
            )
            .padding([10, 0, 0, 0]),
        );

        content_list = content_list.push(scrollable(entries_list));

        self.core.applet.popup_container(content_list).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![self
            .core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Search(query) => {
                self.search_text = query;

                // execute the search if it's long enough
                if !self.search_text.is_empty() {
                    self.entries = fetch_words(if &self.search_text != "" {
                        Some(&self.search_text)
                    } else {
                        None
                    })
                    .unwrap_or(vec![]);
                } else {
                    self.entries = vec![];
                }
            }
            Message::Random => {
                self.entries = fetch_words(None).unwrap_or(vec![]);
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
