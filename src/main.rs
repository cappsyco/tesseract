// SPDX-License-Identifier: GPL-3.0

mod app;
mod i18n;
mod record;
mod timer;

const SEGMENT7_FONT: &[u8] = include_bytes!("../resources/fonts/Segment7Standard.otf");

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    cosmic::iced::advanced::graphics::text::font_system()
        .write()
        .unwrap()
        .load_font(std::borrow::Cow::Borrowed(SEGMENT7_FONT));

    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    );

    cosmic::app::run::<app::AppModel>(settings, ())
}
