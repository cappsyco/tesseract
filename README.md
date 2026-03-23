# Tesseract

A speed cubing timer for the COSMIC™ desktop, built with libcosmic and designed for simple yet effective solve timing.

[![Get it on Flathub](https://flathub.org/api/badge?locale=en)](https://flathub.org/apps/uk.co.cappsy.Tesseract)

### Current state

We're still early in the life of this app, so features are a bit thin on the ground. Currently we have:

- Generate random-move scrambles for cubes from 2x2 to 7x7
- Time your solves
- View your solving record and AO5, 12 and 100, which persists across multiple sessions
- Delete individual solves or the whole record for that puzzle

### Future features

In order of priority (balance between value and effort):

- Comments (e.g., "PLL skip") and manual +2/DNF/DNS in your recorded solves
- Mo3\*
- Special indication when you achieve a new PB and highlight it in your record
- UI enhancements (e.g., scramble image visualization and 'hide timer while solving' option)
- Inspection time for current puzzles (e.g., 15sec for 3x3x3)+Automatic +2/DNF\*
- Keybinds (e.g., 'esc' to cancel the timer start, 'e/E' to cycle between events and 'n' to generate the next scramble)
- WCA compliant scrambles
- Support for missing WCA puzzles (i.e., Megaminx, Pyraminx, Skewb, Square-1, Clock)
- Proper support for all WCA events (e.g., dedicated entries with predefined configurations for BLD).
- Allow organising your solving into separate sessions
- Show pther additional stats in your solving record
- Multi-phase timing
- Other settings (e.g., changing the “hold to start” time and customizing the keybinds)

> '\*' means optional/disabled by default

## Arch User Repository installation

The app can be installed directly from [the AUR](https://aur.archlinux.org/packages/tesseract-timer-git), and this will get you very latest code and not be tied to tagged releases. You will need `base-devel` and `git` if you don't have them already.

```sh
sudo pacman -S base-devel git
git clone https://aur.archlinux.org/tesseract-timer-git.git
cd tesseract-timer-git && makepkg -si
```

## Manual installation

You're going to need to make sure you have the ability to compile Rust binaries, along with `git` and `just`

```sh
git clone https://github.com/cappsyco/tesseract && cd tesseract
just build-release
sudo just install
```

## Translators

[Fluent][fluent] is used for localization of the software. Fluent's translation files are found in the [i18n directory](./i18n). New translations may copy the [English (en) localization](./i18n/en) of the project, rename `en` to the desired [ISO 639-1 language code][iso-codes], and then translations can be provided for each [message identifier][fluent-guide]. If no translation is necessary, the message may be omitted.

## Credit & thanks
* [System76 and their COSMIC desktop environment](https://system76.com/cosmic/)
* [COSMIC Utilities](https://github.com/cosmic-utils/) - Organization containing third party utilities for COSMIC™
* [fluent]: https://projectfluent.org/
* [fluent-guide]: https://projectfluent.org/fluent/guide/hello.html
* [iso-codes]: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
