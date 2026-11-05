// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use crate::config::Config;
use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::{Length, Limits, Subscription, window::Id};
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget::{self, Space};
use cosmic::widget::{column, container, row, scrollable};
use cosmic_ext_applet_dict::{Entry, fetch_words};

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    search_text: String,
    search_generation: u64,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateConfig(Config),
    SearchInput(String),
    Search,
    SearchDebounced(u64),
    SearchClear,
    Random,
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn results_background(theme: &cosmic::Theme) -> cosmic::iced::Color {
    theme
        .cosmic()
        .background(theme.transparent)
        .component
        .base
        .into()
}

type TextInputAppearanceFn =
    fn(&cosmic::Theme, &theme::TextInput) -> cosmic::widget::text_input::Appearance;
type ButtonAppearanceFn =
    fn(&cosmic::Theme, bool, bool, &theme::Button) -> cosmic::widget::button::Style;

fn results_search_appearance(
    theme: &cosmic::Theme,
    appearance: TextInputAppearanceFn,
) -> cosmic::widget::text_input::Appearance {
    let mut style = appearance(theme, &theme::TextInput::Search);
    style.background = results_background(theme).into();
    style
}

fn results_search_style() -> theme::TextInput {
    theme::TextInput::Custom {
        active: Box::new(|theme| {
            results_search_appearance(theme, cosmic::widget::text_input::StyleSheet::active)
        }),
        error: Box::new(|theme| {
            results_search_appearance(theme, cosmic::widget::text_input::StyleSheet::error)
        }),
        hovered: Box::new(|theme| {
            results_search_appearance(theme, cosmic::widget::text_input::StyleSheet::hovered)
        }),
        focused: Box::new(|theme| {
            results_search_appearance(theme, cosmic::widget::text_input::StyleSheet::focused)
        }),
        disabled: Box::new(|theme| {
            results_search_appearance(theme, cosmic::widget::text_input::StyleSheet::disabled)
        }),
    }
}

fn results_button_appearance(
    theme: &cosmic::Theme,
    focused: bool,
    appearance: ButtonAppearanceFn,
) -> cosmic::widget::button::Style {
    let mut style = appearance(theme, focused, false, &theme::Button::Standard);
    style.background = Some(results_background(theme).into());
    style
}

fn results_button_style() -> theme::Button {
    theme::Button::Custom {
        active: Box::new(|focused, theme| {
            results_button_appearance(theme, focused, cosmic::widget::button::Catalog::active)
        }),
        disabled: Box::new(|theme| {
            let mut style =
                cosmic::widget::button::Catalog::disabled(theme, &theme::Button::Standard);
            style.background = Some(results_background(theme).into());
            style
        }),
        hovered: Box::new(|focused, theme| {
            results_button_appearance(theme, focused, cosmic::widget::button::Catalog::hovered)
        }),
        pressed: Box::new(|focused, theme| {
            results_button_appearance(theme, focused, cosmic::widget::button::Catalog::pressed)
        }),
    }
}

fn results_container_style(theme: &cosmic::Theme) -> cosmic::iced::widget::container::Style {
    let cosmic = theme.cosmic();
    let bg = cosmic.background(theme.transparent);

    cosmic::iced::widget::container::Style {
        background: Some(results_background(theme).into()),
        text_color: Some(bg.component.on.into()),
        border: cosmic::iced::Border {
            radius: cosmic.corner_radii.radius_s.into(),
            width: 1.0,
            color: bg.divider.into(),
        },
        ..Default::default()
    }
}

impl AppModel {
    fn search(&mut self) {
        self.entries = if self.search_text.is_empty() {
            Vec::new()
        } else {
            fetch_words(Some(&self.search_text)).unwrap_or_default()
        };
    }
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
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let Spacing {
            space_xxxs,
            space_xxs,
            space_xs,
            space_s,
            ..
        } = theme::active().cosmic().spacing;

        let mut content_list = widget::column([]).padding(space_s).spacing(0);

        // search
        content_list = content_list.push(container(row!(
            widget::search_input(fl!("search"), self.search_text.clone())
                .style(results_search_style())
                .on_input(move |value| Message::SearchInput(value.clone()))
                .on_submit(move |_| Message::Search)
                .on_clear(Message::SearchClear),
            Space::new().width(space_xxs),
            widget::button::icon(widget::icon::from_name("media-playlist-shuffle-symbolic"))
                .class(results_button_style())
                .on_press(Message::Random),
        )));

        // build entries in scrollable list
        let mut entries_list = widget::column([]).spacing(0);

        if !self.entries.is_empty() {
            content_list = content_list.push(Space::new().height(space_s));
            let mut entry_i = 1;
            for entry in &self.entries {
                let mut entry_content =
                    column!(widget::text::title4(capitalize_first(&entry.word)),);

                let mut def_i = 1;
                let mut last_sp = "";
                for def in &entry.defs {
                    if last_sp != def.speech_part {
                        entry_content = entry_content.push(
                            container(widget::text::body(capitalize_first(&def.speech_part)))
                                .padding([space_xxs, 0, 0, 0]),
                        );
                        def_i = 1;
                        last_sp = &def.speech_part;
                    }
                    entry_content = entry_content.push(
                        container(widget::text::body(format!(
                            "{}. {}",
                            def_i,
                            capitalize_first(&def.def)
                        )))
                        .padding([space_xxxs, 0, 0, space_xxs]),
                    );
                    def_i += 1;
                }

                entries_list = entries_list.push(entry_content.padding([space_xxs, space_xs]));
                if entry_i != self.entries.len() {
                    entries_list = entries_list.push(widget::divider::horizontal::default());
                }

                entry_i += 1;
            }

            let results = container(scrollable(entries_list))
                .width(Length::Fill)
                .style(results_container_style);

            content_list = content_list.push(results);
        }

        self.core.applet.popup_container(content_list).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SearchInput(query) => {
                self.search_text = query;
                self.search_generation = self.search_generation.wrapping_add(1);
                let generation = self.search_generation;

                return Task::perform(
                    async move {
                        tokio::time::sleep(SEARCH_DEBOUNCE).await;
                        generation
                    },
                    Message::SearchDebounced,
                )
                .map(cosmic::Action::App);
            }
            Message::Search => {
                self.search_generation = self.search_generation.wrapping_add(1);
                self.search();
            }
            Message::SearchDebounced(generation) => {
                if generation == self.search_generation {
                    self.search();
                }
            }
            Message::SearchClear => {
                self.search_generation = self.search_generation.wrapping_add(1);
                self.search_text.clear();
                self.entries.clear();
            }
            Message::Random => {
                self.search_generation = self.search_generation.wrapping_add(1);
                self.entries = fetch_words(None).unwrap_or(vec![]);
                self.search_text = String::new();
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    cosmic::surface::surface_task(cosmic::surface::action::destroy_popup(p))
                } else {
                    cosmic::surface::surface_task(cosmic::surface::action::app_popup(
                        |_| Default::default(),
                        |app: &mut AppModel| {
                            let new_id = Id::unique();
                            app.popup.replace(new_id);
                            let mut popup_settings = app.core.applet.get_popup_settings(
                                app.core.main_window_id().unwrap(),
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
                            popup_settings
                        },
                        None,
                    ))
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

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
