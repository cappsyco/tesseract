// SPDX-License-Identifier: GPL-3.0

use crate::fl;
use crate::record::{Cube, Record, Solve};
use crate::timer::{Status, Timer, format_from_ms};
use cosmic::app::context_drawer::{self, ContextDrawer};
use cosmic::cosmic_config::{Config, ConfigGet, ConfigSet};
use cosmic::iced::{self, Alignment, Border, Event, Length, Subscription, event, keyboard, time};
use cosmic::iced_widget::scrollable;
use cosmic::prelude::*;
use cosmic::widget::{
    self, Space, about, about::About, container, dropdown, menu, nav_bar, settings,
};
use cube_scrambler::generate_scramble;
use hrsw::Stopwatch;
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tracing;

const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

#[derive(Clone, Debug)]
pub enum DialogPage {
    RemoveAllSolves,
    RemoveSolve(usize),
}

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    dialog_pages: VecDeque<DialogPage>,
    space_pressed: bool,
    current_cube: Cube,
    cube_options: Vec<Cube>,
    cube_options_labels: Vec<String>,
    current_scramble: Vec<String>,
    timer: Timer,
    record: Record,
    stopwatch: Stopwatch,
    about_page: About,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleContextPage(ContextPage),
    Rescramble,
    TimerTick,
    SpacePressed,
    SpaceReleased,
    SpaceHeld,
    OpenUrl(String),
    CubeUpdate(usize),
    DialogCancel,
    DialogRemoveAllSolves,
    DialogRemoveSolve(usize),
    RemoveSolve(usize),
    RemoveAllSolves,
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
        let config = cosmic::cosmic_config::Config::new(Self::APP_ID, 1).unwrap();

        // cube values
        let current_cube = config.get::<Cube>("current_cube").unwrap_or_default();
        let cube_options = vec![
            Cube::Two,
            Cube::Three,
            Cube::Four,
            Cube::Five,
            Cube::Six,
            Cube::Seven,
        ];
        let cube_options_labels: Vec<String> = cube_options.iter().map(|t| t.as_string()).collect();

        // load record for selected cube
        let record = config
            .get::<Record>(current_cube.config_key())
            .unwrap_or_default();

        // trim the last 'xN' of current_cube to pass to cube_scrambler, used in current_scramble
        // and duplicated in rescramble(){}, because I haven't found a way to reuse the same code
        let cube_str = current_cube.as_string().clone();
        let re = Regex::new(r"x\d+$").unwrap();
        let normalized = re.replace(&cube_str, "").to_string();

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            config,
            dialog_pages: VecDeque::new(),
            current_cube: current_cube.clone(),
            cube_options,
            cube_options_labels,
            current_scramble: generate_scramble(None, Some(normalized)).unwrap_or_default(),
            timer: Timer::default(),
            space_pressed: false,
            record,
            stopwatch: Stopwatch::new(),
            about_page: build_about(),
        };

        let command = app.update_title();

        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        match self.context_page {
            ContextPage::About => Some(ContextDrawer {
                title: Some(fl!("about").into()),
                content: about(&self.about_page, |s| Message::OpenUrl(s.to_string())),
                on_close: Message::ToggleContextPage(ContextPage::About),
                header: None,
                actions: None,
                footer: None,
            }),
            ContextPage::Settings => Some(ContextDrawer {
                title: Some(fl!("settings").into()),
                content: about(&self.about_page, |s| Message::OpenUrl(s.to_string())),
                on_close: Message::ToggleContextPage(ContextPage::About),
                header: None,
                actions: None,
                footer: None,
            }), //app::context_drawer::context_drawer(app.settings(), Message::ToggleContextPage).title(app.cosmic.context_page.title()),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
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

        // Cube header
        // TODO: Make this fancier. Maybe an icon and integration with the picker
        page_content = page_content.push(widget::text::title1(self.current_cube.as_string()));

        // Cube picker
        let selected_cube = self
            .cube_options
            .iter()
            .position(|r| *r == self.current_cube)
            .unwrap_or(1);

        page_content = page_content
            .push(
                widget::row()
                    .push(dropdown(
                        &self.cube_options_labels,
                        Some(selected_cube),
                        move |value| Message::CubeUpdate(value),
                    ))
                    .push(
                        widget::button::icon(
                            widget::icon::from_name("view-refresh-symbolic").size(100),
                        )
                        .on_press(Message::Rescramble),
                    ),
            )
            .push(container(
                widget::text::text(self.current_scramble.join(" ")).size(28),
            ));

        // Timer
        let timer_status = self.timer.status.clone();
        let divider_color = match timer_status {
            Status::Hold => active_theme.cosmic().destructive_color(),
            Status::Ready => active_theme.cosmic().success_color(),
            _ => active_theme.cosmic().accent_color(),
        };
        page_content = page_content
            .push(Space::new().height(padding))
            .push(widget::divider::horizontal::default())
            .push(
                widget::text::text(self.timer.display())
                    .size(140)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .push(
                container("")
                    .height(13)
                    .width(150)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(cosmic::iced::Color::from_rgb(
                            divider_color.red,
                            divider_color.green,
                            divider_color.blue,
                        ))),
                        border: Border {
                            radius: 20.into(),
                            width: 0.0,
                            color: iced::Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }),
            );

        // Hint
        page_content = page_content.push(Space::new().height(padding)).push(
            widget::text::text(match self.timer.status {
                Status::Running => fl!("tap-space-to-stop"),
                _ => fl!("hold-space-to-start"),
            })
            .size(16)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        );

        // Record
        if !self.record.solves.is_empty() {
            let mut solve_list = settings::section();
            let ao5_label: String = String::from("AO5: ");
            let ao12_label: String = String::from("AO12: ");
            let ao100_label: String = String::from("AO100: ");
            let ao5_time = match self.record.ao5 {
                Some(ms) => format_from_ms(ms),
                _ => String::from("N/A"),
            };
            let ao12_time = match self.record.ao12 {
                Some(ms) => format_from_ms(ms),
                _ => String::from("N/A"),
            };
            let ao100_time = match self.record.ao100 {
                Some(ms) => format_from_ms(ms),
                _ => String::from("N/A"),
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
                    )
                    .push(container(
                        widget::button::icon(
                            widget::icon::from_name("edit-delete-symbolic").size(100),
                        )
                        .class(cosmic::style::Button::Destructive)
                        .on_press(Message::DialogRemoveAllSolves),
                    )),
            );

            // Solves
            let mut solve_i = 0;
            for solve in &self.record.solves {
                solve_list = solve_list.add(
                    widget::row()
                        .push(
                            container(
                                widget::text::body(format!("{}", solve.scramble.join(" ")))
                                    .size(16)
                                    .width(Length::Fill),
                            )
                            .padding(active_theme.cosmic().space_s())
                            .align_y(Alignment::Center),
                        )
                        .push(
                            container(
                                widget::text::body(format!("{}", solve.time()))
                                    .size(22)
                                    .align_x(Alignment::Center),
                            )
                            .padding(active_theme.cosmic().space_s()),
                        )
                        .push(
                            container(
                                widget::button::icon(
                                    widget::icon::from_name("edit-delete-symbolic").size(100),
                                )
                                .on_press(Message::DialogRemoveSolve(solve_i)),
                            )
                            .padding([
                                ((active_theme.cosmic().space_s() / 2) + 2),
                                0,
                                0,
                                0,
                            ]),
                        ),
                );
                solve_i += 1;
            }

            page_content = page_content
                .push(Space::new().height(padding))
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
        Subscription::batch(vec![
            event::listen_with(|event, _status, _window_id| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key.as_ref() {
                    cosmic::iced::keyboard::Key::Character(" ") => Some(Message::SpacePressed),
                    _ => None,
                },
                Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => match key.as_ref() {
                    cosmic::iced::keyboard::Key::Character(" ") => Some(Message::SpaceReleased),
                    _ => None,
                },
                // TODO: Add mouse / touch bindings
                _ => None,
            }),
            match self.timer.status {
                Status::Running => {
                    time::every(Duration::from_millis(100)).map(|_| Message::TimerTick)
                }
                _ => Subscription::none(),
            },
            match self.space_pressed {
                true => time::every(Duration::from_millis(500)).map(|_| Message::SpaceHeld),
                _ => Subscription::none(),
            },
        ])
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let dialog = match dialog_page {
            DialogPage::RemoveSolve(i) => widget::dialog()
                .title(fl!("remove-solve"))
                .primary_action(
                    widget::button::destructive(fl!("remove")).on_press(Message::RemoveSolve(*i)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .apply(Element::from),

            DialogPage::RemoveAllSolves => widget::dialog()
                .title(fl!("remove-all-solves-for-puzzle"))
                .primary_action(
                    widget::button::destructive(fl!("remove")).on_press(Message::RemoveAllSolves),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .apply(Element::from),
        };

        Some(dialog)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::OpenUrl(url) => match open::that_detached(&url) {
                Ok(_) => (),
                Err(err) => tracing::error!("Failed to open URL: {err}"),
            },

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }

            Message::DialogRemoveSolve(i) => {
                self.dialog_pages.push_front(DialogPage::RemoveSolve(i));
            }

            Message::DialogRemoveAllSolves => {
                self.dialog_pages.push_front(DialogPage::RemoveAllSolves);
            }

            // TODO: refactor all this
            Message::TimerTick => {
                self.timer.time = self.stopwatch.elapsed().as_millis() as u32;
            }
            Message::SpacePressed => {
                self.space_pressed = true;
                if self.timer.status == Status::Running {
                    self.timer.time = self.stopwatch.elapsed().as_millis() as u32;
                    let solve = Solve::new(self.timer.time, &self.current_scramble);
                    self.timer.status = Status::Stopped;
                    self.record.add_solve(solve);
                    self.save_record();
                    self.rescramble();
                } else if self.timer.status == Status::Stopped {
                    self.timer.status = Status::Hold;
                }
            }
            Message::SpaceReleased => {
                self.space_pressed = false;
                if self.timer.status == Status::Ready {
                    self.timer.time = 0;
                    self.stopwatch.reset_and_start();
                    self.timer.status = Status::Running;
                } else {
                    self.timer.status = Status::Stopped;
                    self.stopwatch.stop();
                }
            }
            Message::SpaceHeld => {
                if self.timer.status == Status::Hold {
                    self.timer.status = Status::Ready;
                }
            }
            Message::CubeUpdate(uid) => {
                self.current_cube = self.cube_options[uid].clone();
                self.record = self
                    .config
                    .get::<Record>(self.current_cube.config_key())
                    .unwrap_or_default();
                let _ = self.config.set("current_cube", &self.current_cube);
                self.rescramble();
            }
            Message::Rescramble => {
                self.rescramble();
            }
            Message::RemoveSolve(uid) => {
                self.record.solves.remove(uid);
                self.save_record();
                self.dialog_pages.pop_front();
            }
            Message::RemoveAllSolves => {
                self.record.solves = vec![];
                self.save_record();
                self.dialog_pages.pop_front();
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

    fn rescramble(&mut self) {
        let cube_str = self.current_cube.as_string().clone();
        let re = Regex::new(r"x\d+$").unwrap();
        let normalized = re.replace(&cube_str, "").to_string();
        self.current_scramble = generate_scramble(None, Some(normalized)).unwrap_or_default();
    }
    fn save_record(&mut self) {
        let _ = self
            .config
            .set(self.current_cube.config_key(), &self.record);
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
        }
    }
}

pub fn build_about() -> About {
    About::default()
        .developers([("Jonathan Capps", "cappsy@gmail.com")])
        .version(env!("CARGO_PKG_VERSION"))
        .name(fl!("app-title"))
        .icon(widget::icon::from_svg_bytes(APP_ICON))
        .license(env!("CARGO_PKG_LICENSE"))
        .author("Jonathan Capps")
        .links([(fl!("repository"), env!("CARGO_PKG_REPOSITORY"))])
        .links([
            (fl!("repository"), env!("CARGO_PKG_REPOSITORY")),
            (
                fl!("contributors"),
                "https://github.com/cappsyco/tesseract/graphs/contributors",
            ),
        ])
}
