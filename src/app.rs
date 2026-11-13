// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Write as _;
use std::time::Duration;

use crate::config::Config;
use crate::fl;
use cosmic::Application;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::Alignment::Center;
use cosmic::iced::mouse;
use cosmic::iced::{Length, Limits, Subscription, window::Id};
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget::{self, Space, about, about::About, column, container, row, scrollable};
use cosmic_ext_applet_dict::{Entry, random_words, search_words};

const APP_LOGO: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/logo.svg");
const STAR_OUTLINE_ICON: &[u8] = include_bytes!("../resources/icons/star-outline-symbolic.svg");
const REPOSITORY_URL: &str = env!("CARGO_PKG_REPOSITORY");
const SEARCH_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    active_view: View,
    about_page: About,
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
    ChangeView(View),
    OpenUrl(String),
    ToggleBookmark(String),
    CopyWord(String),
    ViewBookmark(String),
}

#[derive(Debug, Clone, Default, PartialEq, Copy)]
pub enum View {
    #[default]
    Main,
    About,
    Bookmarks,
}

impl AppModel {
    fn search(&mut self, limit: Option<u8>) {
        self.entries = if self.search_text.is_empty() {
            Vec::new()
        } else {
            search_words(&self.search_text, limit).unwrap_or_default()
        };
    }

    fn save_config(&self) {
        match cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
            Ok(context) => {
                if let Err(err) = self.config.write_entry(&context) {
                    eprintln!("Failed to save config: {err}");
                }
            }
            Err(err) => eprintln!("Failed to access config: {err}"),
        }
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
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        let app = AppModel {
            core,
            config,
            about_page: build_about(),
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
        let spacing = theme::active().cosmic().spacing;
        let content = match self.active_view {
            View::Main => self.search_view(&spacing),
            View::About => self.about_view(&spacing),
            View::Bookmarks => self.bookmarks_view(&spacing),
        };

        self.core.applet.popup_container(content).into()
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
                self.search(None);
            }
            Message::SearchDebounced(generation) => {
                if generation == self.search_generation {
                    self.search(None);
                }
            }
            Message::SearchClear => {
                self.search_generation = self.search_generation.wrapping_add(1);
                self.search_text.clear();
                self.entries.clear();
            }
            Message::Random => {
                self.entries = random_words(Some(1)).unwrap_or_default();
                self.search_text = String::new();
                self.search_generation = self.search_generation.wrapping_add(1);
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    cosmic::surface::surface_task(cosmic::surface::action::destroy_popup(p))
                } else {
                    cosmic::surface::surface_task(cosmic::surface::action::app_popup(
                        |_| cosmic::surface::action::LiveSettings::default(),
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
            Message::ChangeView(new_view) => {
                self.active_view = new_view;
            }
            Message::OpenUrl(url) => {
                return Task::future(async move {
                    if let Err(err) = open_url(url).await {
                        eprintln!("Failed to open URL: {err}");
                    }
                })
                .discard()
                .map(cosmic::Action::App);
            }
            Message::ToggleBookmark(word) => {
                self.config.toggle_bookmark(&word);
                self.save_config();
            }
            Message::CopyWord(word) => {
                if let Some(entry) = self.entries.iter().find(|entry| entry.word == word) {
                    return cosmic::iced::clipboard::write(entry_to_text(entry))
                        .map(cosmic::Action::App);
                }
            }
            Message::ViewBookmark(word) => {
                self.search_text = word;
                self.active_view = View::Main;
                self.search_generation = self.search_generation.wrapping_add(1);
                self.search(Some(1));
            }
        }
        Task::none()
    }

    /* Styling */
    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn page_column<'a>(spacing: &Spacing) -> widget::Column<'a, Message, Theme> {
        column!()
            .padding([
                spacing.space_xs,
                spacing.space_s,
                spacing.space_s,
                spacing.space_s,
            ])
            .spacing(0)
            .width(Length::Fill)
    }

    fn nav_bar<'a>() -> Element<'a, Message> {
        container(row!(
            widget::button::icon(widget::icon::from_name("dialog-information-symbolic"))
                .on_press(Message::ChangeView(View::About)),
            Space::new().width(Length::Fill),
            widget::button::icon(widget::icon::from_name("insert-star-symbolic"))
                .on_press(Message::ChangeView(View::Bookmarks)),
        ))
        .into()
    }

    fn back_nav_bar<'a>() -> Element<'a, Message> {
        container(row!(
            widget::button::icon(widget::icon::from_name("go-previous-symbolic").size(16))
                .label(fl!("back"))
                .on_press(Message::ChangeView(View::Main)),
        ))
        .into()
    }

    fn search_view<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        let mut view_content = Self::page_column(spacing)
            .push(Self::nav_bar())
            .push(Space::new().height(spacing.space_xxs))
            .push(self.search_bar(spacing));

        if !self.entries.is_empty() {
            view_content = view_content
                .push(Space::new().height(spacing.space_s))
                .push(self.entries_view(spacing));
        }

        view_content.into()
    }

    fn about_view<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        Self::page_column(spacing)
            .push(Self::back_nav_bar())
            .push(Space::new().height(spacing.space_xxs))
            .push(about::about(&self.about_page, |url| {
                Message::OpenUrl(url.to_owned())
            }))
            .into()
    }

    fn bookmarks_view<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        let mut view_content = Self::page_column(spacing)
            .push(Self::back_nav_bar())
            .push(Space::new().height(spacing.space_xxs))
            .push(widget::text::title4(fl!("bookmarks")));

        if self.config.bookmarks.is_empty() {
            view_content = view_content
                .push(Space::new().height(spacing.space_xs))
                .push(widget::text::heading(fl!("no-bookmarks")));
        } else {
            let mut bookmarks_list = column!().spacing(0).width(Length::Fill);

            for bookmark in &self.config.bookmarks {
                bookmarks_list = bookmarks_list
                    .push(Space::new().height(spacing.space_xs))
                    .push(
                        container(
                            row!(
                                widget::mouse_area(
                                    widget::text::heading(capitalize_first(bookmark))
                                        .width(Length::Fill),
                                )
                                .on_press(Message::ViewBookmark(bookmark.clone()))
                                .interaction(mouse::Interaction::Pointer),
                                widget::button::icon(widget::icon::from_name(
                                    "edit-delete-symbolic"
                                ))
                                .icon_size(15)
                                .on_press(Message::ToggleBookmark(bookmark.clone()))
                            )
                            .align_y(Center),
                        )
                        .class(theme::Container::Card)
                        .padding([spacing.space_xxs, spacing.space_xs]),
                    );
            }

            view_content = view_content.push(
                scrollable(bookmarks_list)
                    .spacing(spacing.space_xxs)
                    .width(Length::Fill),
            );
        }

        view_content.into()
    }

    fn search_bar(&self, spacing: &Spacing) -> Element<'_, Message> {
        container(row!(
            widget::search_input(fl!("search"), self.search_text.as_str())
                .on_input(Message::SearchInput)
                .on_submit(|_| Message::Search)
                .on_clear(Message::SearchClear),
            Space::new().width(spacing.space_xxs),
            widget::button::icon(widget::icon::from_name("media-playlist-shuffle-symbolic"))
                .on_press(Message::Random)
        ))
        .into()
    }

    fn entries_view<'a>(&'a self, spacing: &Spacing) -> Element<'a, Message> {
        let mut entries_list = column!().spacing(0).width(Length::Fill);

        for (index, entry) in self.entries.iter().enumerate() {
            entries_list = entries_list.push(self.entry_view(entry, spacing));

            if index + 1 != self.entries.len() {
                entries_list = entries_list.push(widget::divider::horizontal::default());
            }
        }

        scrollable(
            container(entries_list)
                .width(Length::Fill)
                .class(theme::Container::Card),
        )
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .into()
    }

    fn entry_view<'a>(&'a self, entry: &'a Entry, spacing: &Spacing) -> Element<'a, Message> {
        let bookmark_icon: widget::icon::Handle = if self.config.is_bookmarked(&entry.word) {
            widget::icon::from_name("user-bookmarks-symbolic").into()
        } else {
            widget::icon::from_svg_bytes(STAR_OUTLINE_ICON).symbolic(true)
        };

        let entry_content = column!(
            row!(
                widget::text::title4(capitalize_first(&entry.word)).width(Length::Fill),
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                    .icon_size(15)
                    .on_press(Message::CopyWord(entry.word.clone())),
                widget::button::icon(bookmark_icon)
                    .icon_size(15)
                    .on_press(Message::ToggleBookmark(entry.word.clone()))
            )
            .align_y(Center)
        )
        .push(Self::definition_list(entry, spacing));

        entry_content
            .padding([spacing.space_xxs, spacing.space_xs])
            .into()
    }

    fn definition_list<'a>(
        entry: &'a Entry,
        spacing: &Spacing,
    ) -> widget::Column<'a, Message, Theme> {
        let mut definitions = column!();
        let mut last_speech_part = "";
        let mut definition_index = 1;

        for definition in &entry.defs {
            if last_speech_part != definition.speech_part {
                definitions = definitions.push(
                    container(widget::text::body(capitalize_first(
                        &definition.speech_part,
                    )))
                    .padding([spacing.space_xxs, 0, 0, 0]),
                );
                last_speech_part = &definition.speech_part;
                definition_index = 1;
            }

            definitions = definitions.push(
                container(widget::text::body(format!(
                    "{}. {}",
                    definition_index,
                    capitalize_first(&definition.def)
                )))
                .padding([spacing.space_xxxs, 0, 0, spacing.space_xxs]),
            );

            definition_index += 1;
        }

        definitions
    }
}

/* Pages */
pub fn build_about() -> About {
    About::default()
        .developers([("Jonathan Capps", "hello@cappsy.dev")])
        .version(env!("CARGO_PKG_VERSION"))
        .name(fl!("app-title"))
        .icon(widget::icon::from_svg_bytes(APP_LOGO).symbolic(false))
        .author("Jonathan Capps")
        .links([
            (fl!("repository"), REPOSITORY_URL.to_owned()),
            (fl!("support"), format!("{REPOSITORY_URL}/issues")),
            (
                fl!("contributors"),
                format!("{REPOSITORY_URL}/graphs/contributors"),
            ),
        ])
        .license(env!("CARGO_PKG_LICENSE"))
        .comments(env!("CARGO_PKG_DESCRIPTION"))
}

/* Helper functions */
async fn open_url(url: String) -> Result<(), ashpd::Error> {
    let uri = ashpd::Uri::parse(&url)?;
    ashpd::desktop::open_uri::OpenFileRequest::default()
        .send_uri(&uri)
        .await?;
    Ok(())
}

fn entry_to_text(entry: &Entry) -> String {
    let mut text = capitalize_first(&entry.word);
    let mut current_speech_part: Option<&str> = None;
    let mut definition_index = 0;

    for definition in &entry.defs {
        if current_speech_part == Some(definition.speech_part.as_str()) {
            definition_index += 1;
        } else {
            current_speech_part = Some(definition.speech_part.as_str());
            definition_index = 1;
            text.push_str("\n\n");
            text.push_str(&capitalize_first(&definition.speech_part));
        }

        text.push('\n');
        let _ = write!(
            text,
            "{definition_index}. {}",
            capitalize_first(&definition.def)
        );
    }

    text
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
