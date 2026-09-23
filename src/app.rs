// SPDX-License-Identifier: GPL-3.0

use crate::fl;
use crate::cube::Cube;
use crate::records::{self, Records};
use crate::timer::{Status, Timer};
use cosmic::app::context_drawer::{self, ContextDrawer};
use cosmic::cosmic_config::{Config, ConfigGet, ConfigSet};
use cosmic::iced::{self, Alignment, Border, Event, Length, Subscription, event, keyboard, time};
use cosmic::prelude::*;
use cosmic::widget::{
    self, Space, about, about::About, container, dropdown, menu, nav_bar, scrollable, settings,
};
use cube_scrambler::generate_scramble;
use hrsw::Stopwatch;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tracing;

const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

#[derive(Clone, Debug)]
pub enum DialogPage {
    RemoveAllSolves,
    RemoveSolve(i64),
}

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    state: Config,
    dialog_pages: VecDeque<DialogPage>,
    space_pressed: bool,
    keybinds: HashMap<String, String>,
    current_cube: Cube,
    cube_options: Vec<Cube>,
    cube_options_labels: Vec<String>,
    current_scramble: Vec<String>,
    timer: Timer,
    records: Records,
    solves: Vec<records::Solve>,
    pb: Option<i64>,
    ao5: records::Average,
    ao12: records::Average,
    ao100: records::Average,
    stopwatch: Stopwatch,
    about_page: About,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleContextPage(ContextPage),
    Rescramble,
    TimerTick,
    KeyPressed(String),
    KeyReleased(String),
    SpaceHeld,
    OpenUrl(String),
    CubeUpdate(usize),
    DialogCancel,
    DialogRemoveAllSolves,
    DialogRemoveSolve(i64),
    RemoveSolve(i64),
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
        let state = cosmic::cosmic_config::Config::new_state(Self::APP_ID, 1).unwrap();

        // keybinds — action -> key map, loaded from config (or defaults if not present)
        // TODO: merge with defaults instead of replacing — new actions won't show up for existing users otherwise
        let keybinds = config
            .get::<HashMap<String, String>>("keybinds")
            .unwrap_or_else(|_| {
                let mut defaults = HashMap::new();
                defaults.insert("start_stop".to_string(), " ".to_string());
                defaults.insert("cancel".to_string(), "Escape".to_string());
                defaults.insert("dnf".to_string(), "3".to_string());
                defaults.insert("normal".to_string(), "1".to_string());
                defaults
            });

        // cube values
        let current_cube = state
            .get::<String>("current_cube")
            .ok()
            .and_then(|id| Cube::from_id(&id))
            .unwrap_or_default();
        let cube_options = vec![
            Cube::Two,
            Cube::Three,
            Cube::Four,
            Cube::Five,
            Cube::Six,
            Cube::Seven,
        ];
        let cube_options_labels: Vec<String> = cube_options.iter().map(|t| t.as_string()).collect();

        // open the solves database
        let data_dir = directories::ProjectDirs::from("uk.co", "cappsy", "Tesseract-timer")
            .expect("could not determine data directory");
        std::fs::create_dir_all(data_dir.data_dir()).expect("failed to create data directory");
        let records = Records::open(&data_dir.data_dir().join("records.db"))
            .expect("failed to open records database");

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            config,
            state,
            dialog_pages: VecDeque::new(),
            current_cube: current_cube.clone(),
            cube_options,
            cube_options_labels,
            current_scramble: generate_scramble(None, Some(current_cube.as_string()))
                .unwrap_or_default(),
            timer: Timer::default(),
            space_pressed: false,
            keybinds,
            records,
            solves: Vec::new(),
            pb: None,
            ao5: records::Average::Incomplete,
            ao12: records::Average::Incomplete,
            ao100: records::Average::Incomplete,
            stopwatch: Stopwatch::new(),
            about_page: build_about(),
        };

        app.refresh_stats(); // populates solves/pb/ao5/12/100 for the first time
        let command = app.update_title();

        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
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
        let mut page_content = widget::column([])
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
                widget::row([])
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
        if !self.solves.is_empty() {
            let mut solve_list = settings::section();
            let ao5_label: String = String::from("AO5: ");
            let ao12_label: String = String::from("AO12: ");
            let ao100_label: String = String::from("AO100: ");
            let ao5_time = self.ao5.to_display();
            let ao12_time = self.ao12.to_display();
            let ao100_time = self.ao100.to_display();
            // Averages
            solve_list = solve_list.add(
                widget::row([])
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
            for solve in &self.solves {
                solve_list = solve_list.add(
                    widget::row([])
                        .push(
                            container(
                                widget::text::body(solve.scramble.clone())
                                    .size(18)
                                    .width(Length::Fill),
                            )
                            .padding(active_theme.cosmic().space_s())
                            .align_y(Alignment::Center),
                        )
                        .push(
                            container(
                                widget::text::body(solve.time())
                                    .size(22)
                                    .align_x(Alignment::Center)
                                    .class(cosmic::theme::style::Text::Color(
                                        if solve.time() == "DNF" {
                                            active_theme.cosmic().control_6().into()
                                        } else {
                                            active_theme.cosmic().on_bg_color().into()
                                        },
                                    )),
                            )
                            .padding(active_theme.cosmic().space_s()),
                        )
                        .push(
                            container(
                                widget::button::icon(
                                    widget::icon::from_name("edit-delete-symbolic").size(100),
                                )
                                .on_press(Message::DialogRemoveSolve(solve.id))
                            )
                            .padding([
                                ((active_theme.cosmic().space_s() / 2) + 2),
                                0,
                                0,
                                0,
                            ]),
                        ),
                );
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
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    key_to_string(key.as_ref()).map(Message::KeyPressed)
                }
                Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                    key_to_string(key.as_ref()).map(Message::KeyReleased)
                }
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
            DialogPage::RemoveSolve(id) => widget::dialog()
                .title(fl!("remove-solve"))
                .primary_action(
                    widget::button::destructive(fl!("remove")).on_press(Message::RemoveSolve(*id)),
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

            Message::DialogRemoveSolve(id) => {
                self.dialog_pages.push_front(DialogPage::RemoveSolve(id));
            }

            Message::DialogRemoveAllSolves => {
                self.dialog_pages.push_front(DialogPage::RemoveAllSolves);
            }

            // TODO: refactor all this
            Message::TimerTick => {
                self.timer.time = self.stopwatch.elapsed().as_millis() as u32;
            }
            Message::KeyPressed(key) => {
                if self.is_bound_key("cancel", &key) {
                    self.cancel_timer();
                } else if self.is_bound_key("dnf", &key) {
                    self.set_last_solve_penalty(-1);
                } else if self.is_bound_key("normal", &key) {
                    self.set_last_solve_penalty(0);
                } else if self.is_bound_key("penalty_cycle", &key) {
                    self.cycle_last_solve_penalty();
                } else if self.is_bound_key("start_stop", &key) {
                    self.space_pressed = true;
                    if self.timer.status == Status::Running {
                        self.timer.time = self.stopwatch.elapsed().as_millis() as u32;
                        let _ = self.records.log(
                            self.current_cube.config_key(),
                            self.timer.time as i64,
                            &self.current_scramble.join(" "),
                        );
                        self.timer.status = Status::Stopped;
                        self.refresh_stats();
                        self.rescramble();
                    } else if self.timer.status == Status::Stopped {
                        self.timer.status = Status::Hold;
                    }
                }
            }
            Message::KeyReleased(key) => {
                if self.is_bound_key("start_stop", &key) {
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
            }
            Message::SpaceHeld => {
                if self.timer.status == Status::Hold {
                    self.timer.status = Status::Ready;
                }
            }
            Message::CubeUpdate(uid) => {
                self.current_cube = self.cube_options[uid].clone();
                let _ = self.state.set("current_cube", self.current_cube.config_key());
                self.refresh_stats();
                self.rescramble();
            }
            Message::Rescramble => {
                self.rescramble();
            }
            Message::RemoveSolve(id) => {
                let _ = self.records.delete(id);
                self.refresh_stats();
                self.dialog_pages.pop_front();
            }

            Message::RemoveAllSolves => {
                let _ = self.records.delete_all(self.current_cube.config_key());
                self.refresh_stats();
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

const PENALTY_STEP_MS: i64 = 2000;

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
        self.current_scramble =
            generate_scramble(None, Some(self.current_cube.as_string())).unwrap_or_default();
    }

    fn is_bound_key(&self, action: &str, key: &str) -> bool {
    self.keybinds.get(action).map(|k| k.as_str()) == Some(key)
    }

    fn cancel_timer(&mut self) {
        match self.timer.status {
            Status::Hold | Status::Ready => {
                self.timer.status = Status::Stopped;
            }
            Status::Running => {
                self.stopwatch.stop();
                self.timer.time = 0;
                self.timer.status = Status::Stopped;
            }
            Status::Stopped => {}
        }
    }

    fn refresh_stats(&mut self) {
        let event_id = self.current_cube.config_key();
        self.solves = self.records.latest(event_id, 100).unwrap_or_default();
        self.pb = self.records.personal_best(event_id).unwrap_or(None);
        self.ao5 = self.records.average(event_id, 5).unwrap_or(records::Average::Incomplete);
        self.ao12 = self.records.average(event_id, 12).unwrap_or(records::Average::Incomplete);
        self.ao100 = self.records.average(event_id, 100).unwrap_or(records::Average::Incomplete);
    }

    fn set_last_solve_penalty(&mut self, penalty_ms: i64) {
        if let Some(last) = self.solves.first() {
            let _ = self.records.set_penalty(last.id, penalty_ms);
            self.refresh_stats();
        }
    }

    fn cycle_last_solve_penalty(&mut self) {
        if let Some(last) = self.solves.first() {
            let next = if last.penalty_ms >= 0 { last.penalty_ms + PENALTY_STEP_MS } else { PENALTY_STEP_MS };
            let _ = self.records.set_penalty(last.id, next);
            self.refresh_stats();
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

fn key_to_string(key: cosmic::iced::keyboard::Key<&str>) -> Option<String> {
    match key {
        cosmic::iced::keyboard::Key::Character(s) => Some(s.to_string()),
        cosmic::iced::keyboard::Key::Named(named) => named_key_to_string(named),
        _ => None,
    }
}

fn named_key_to_string(named: keyboard::key::Named) -> Option<String> {
    match named {
        keyboard::key::Named::Escape => Some("Escape".to_string()),
        _ => None,
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
