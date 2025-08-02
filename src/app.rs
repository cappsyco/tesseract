// SPDX-License-Identifier: GPL-3.0

use crate::config::Config;
use crate::fl;
use crate::record::{Record, Solve};
use crate::{
    scrambler::Scramble,
    timer::{Status, Timer, format_from_ms},
};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::{Alignment, Length, Subscription, keyboard};
use cosmic::iced::{Radius, time};
use cosmic::iced_core::text::LineHeight;
use cosmic::iced_widget::{rule, scrollable};
use cosmic::prelude::*;
use cosmic::widget::{self, Space, container, menu, nav_bar, settings};
use cosmic::{cosmic_theme, theme};
use std::collections::HashMap;
use std::time::Duration;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    space_pressed: bool,
    current_scramble: Scramble,
    timer: Timer,
    record: Record,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    LaunchUrl(String),
    Rescramble,
    TimerTick,
    SpacePressed,
    SpaceReleased,
    SpaceHeld,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "uk.co.cappsy.Tesseract";

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
        let nav = nav_bar::Model::default();

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),

            current_scramble: Scramble::new(),
            timer: Timer::default(),
            space_pressed: false,
            record: Record::default(),
        };

        let command = app.update_title();

        (app, command)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    fn view(&self) -> Element<Self::Message> {
        //  Get theme info
        let active_theme = cosmic::theme::active();
        let padding = if self.core.is_condensed() {
            active_theme.cosmic().space_s()
        } else {
            active_theme.cosmic().space_l()
        };

        // Start container
        let mut page_content = widget::column()
            .padding(0.)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        // Scramble
        page_content = page_content
            .push(
                widget::button::icon(widget::icon::from_name("view-refresh-symbolic").size(100))
                    .icon_size(20)
                    .on_press(Message::Rescramble),
            )
            .push(widget::text::text(self.current_scramble.display()).size(40))
            .align_x(Alignment::Center)
            .width(Length::Fill);

        // Timer
        let timer_status = self.timer.status.clone();
        page_content = page_content
            .push(Space::with_height(padding))
            .push(widget::divider::horizontal::default())
            .push(
                widget::text::text(self.timer.display())
                    .size(140)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .push(
                widget::divider::horizontal::heavy()
                    .width(150)
                    .class(theme::Rule::custom(move |theme| {
                        let cosmic = theme.cosmic();
                        let divider_color = match timer_status {
                            Status::Hold => &cosmic.destructive_color(),
                            Status::Ready => &cosmic.success_color(),
                            _ => &cosmic.primary_component_color(),
                        };

                        rule::Style {
                            color: cosmic::iced::Color::from_rgb(
                                divider_color.red,
                                divider_color.green,
                                divider_color.blue,
                            ),
                            width: 15,
                            radius: Radius::new(20),
                            fill_mode: rule::FillMode::Full,
                        }
                    })),
            );

        // Hint
        page_content = page_content.push(Space::with_height(padding)).push(
            widget::text::text(match self.timer.status {
                Status::Running => fl!("tap-space-to-stop"),
                _ => fl!("hold-space-to-start"),
            })
            .size(15)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        );

        // Record
        if !self.record.solves.is_empty() {
            let mut solve_list = settings::section().title(fl!("your-solving-record"));
            let ao5_label: String = String::from("AO5: ");
            let ao12_label: String = String::from("AO12: ");
            let ao100_label: String = String::from("AO100: ");
            let ao5_time = match self.record.ao5 {
                Some(ms) => format_from_ms(ms),
                None => String::from("N/A"),
            };
            let ao12_time = match self.record.ao12 {
                Some(ms) => format_from_ms(ms),
                None => String::from("N/A"),
            };
            let ao100_time = match self.record.ao100 {
                Some(ms) => format_from_ms(ms),
                None => String::from("N/A"),
            };

            // Averages
            solve_list = solve_list.add(
                widget::row()
                    .push(
                        widget::text::title4(ao5_label + &ao5_time)
                            .size(15)
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                    )
                    .push(
                        widget::text::title4(ao12_label + &ao12_time)
                            .size(15)
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                    )
                    .push(
                        widget::text::title4(ao100_label + &ao100_time)
                            .size(15)
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                    ),
            );

            // Solves
            for solve in &self.record.solves {
                solve_list = solve_list.add(
                    widget::row()
                        .push(
                            widget::text::body(format!("{}", solve.scramble.display()))
                                .size(15)
                                .width(Length::Fill),
                        )
                        .push(
                            widget::text::body(format!("{}", solve.time()))
                                .size(18)
                                .width(Length::Fill)
                                .line_height(LineHeight::Relative(2.0))
                                .align_x(Alignment::End)
                                .align_y(Alignment::Center),
                        ),
                );
            }
            page_content = page_content
                .push(Space::with_height(padding))
                .push(solve_list);
        }

        // Combine all elements to finished page
        let page_container = container(page_content)
            .max_width(700)
            .width(Length::Fill)
            .apply(container)
            .center_x(Length::Fill)
            .padding(padding);

        // Display
        let content: Element<_> = scrollable(page_container).into();

        content
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        fn handle_press(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
            match key.as_ref() {
                keyboard::Key::Named(Named::Space) => Some(Message::SpacePressed),
                _ => None,
            }
        }
        fn handle_release(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
            match key.as_ref() {
                keyboard::Key::Named(Named::Space) => Some(Message::SpaceReleased),
                _ => None,
            }
        }

        Subscription::batch(vec![
            keyboard::on_key_press(handle_press),
            keyboard::on_key_release(handle_release),
            match self.timer.status {
                Status::Running => {
                    time::every(Duration::from_millis(10)).map(|_| Message::TimerTick)
                }
                _ => Subscription::none(),
            },
            match self.space_pressed {
                true => time::every(Duration::from_millis(600)).map(|_| Message::SpaceHeld),
                _ => Subscription::none(),
            },
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::Rescramble => {
                self.current_scramble = Scramble::new();
            }

            // TODO: make this cleaner. Move more logic into the timer module
            Message::TimerTick => {
                self.timer.time += 10;
            }
            Message::SpacePressed => {
                self.space_pressed = true;
                if self.timer.status == Status::Running {
                    self.timer.status = Status::Stopped;
                    let solve = Solve::new(self.timer.time, &self.current_scramble);
                    self.record.add_solve(solve);
                    self.current_scramble = Scramble::new();
                } else if self.timer.status == Status::Stopped {
                    self.timer.status = Status::Hold;
                }
            }
            Message::SpaceReleased => {
                self.space_pressed = false;
                if self.timer.status == Status::Ready {
                    self.timer.time = 0;
                    self.timer.status = Status::Running;
                } else {
                    self.timer.status = Status::Stopped;
                }
            }
            Message::SpaceHeld => {
                if self.timer.status == Status::Hold {
                    self.timer.status = Status::Ready;
                }
            }
        }
        Task::none()
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        self.update_title()
    }
}

impl AppModel {
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));
        let title = widget::text::title3(fl!("app-title"));
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
