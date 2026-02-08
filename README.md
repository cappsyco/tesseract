# Tesseract

A speed cubing timer for the COSMIC™ desktop.

[![Get it on Flathub](https://flathub.org/api/badge?locale=en)](https://flathub.org/apps/uk.co.cappsy.Tesseract)

### Current state

We're still early in the life of this app, so features are a bit thin on the ground. Currently we have:

- Generate scrambles for any of the standard WCA cube puzzles (2x2, 3x3, 4x4, 5x5, 6x6 and7x7)
- Time your solves
- View your solving record and AO5, 12 and 100, which persists across multiple sessions
- Delete individual solves or the whole record for that puzzle

### Future features

In order of priority:

- Additional stats in your solving record, such as all time best single and average times
- Allow organising your solving into separate sessions
- Support for WCA puzzles other than the standard cubes
- UI enhancements (e.g. scramble visualisation)

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

## Credit & thanks
* [System76 and their COSMIC desktop environment](https://system76.com/cosmic/)
* [COSMIC Utilities](https://github.com/cosmic-utils/) - Organization containing third party utilities for COSMIC™
