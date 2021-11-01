
# Installation

Unless otherwise specified, you will need to install `taskwarrior` first. See <https://taskwarrior.org/download/> for more information.

**Manual** ( _Recommended_ ) [![](https://img.shields.io/github/v/tag/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest) [![](https://img.shields.io/github/downloads/kdheepak/taskwarrior-tui/total)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

1. Download the tar.gz file for your OS from [the latest release](https://github.com/kdheepak/taskwarrior-tui/releases/latest).
2. Unzip the tar.gz file
3. Run with `./taskwarrior-tui`.

**Install from source** [![](https://img.shields.io/badge/branch-main-red)](https://github.com/kdheepak/taskwarrior-tui)

```bash
git clone https://github.com/kdheepak/taskwarrior-tui.git
cd taskwarrior-tui
cargo build --release
```

**Using [`brew`](https://brew.sh/)** [![](https://img.shields.io/homebrew/v/taskwarrior-tui)](https://formulae.brew.sh/formula/taskwarrior-tui) [![](https://img.shields.io/homebrew/installs/dy/taskwarrior-tui)](https://formulae.brew.sh/formula/taskwarrior-tui)


This installs `task` from `homebrew` as well.

```bash
brew install taskwarrior-tui
```

**Installation for Arch Linux** [![](https://img.shields.io/archlinux/v/community/x86_64/taskwarrior-tui)](https://archlinux.org/packages/community/x86_64/taskwarrior-tui/) [![](https://img.shields.io/aur/version/taskwarrior-tui-git)](https://aur.archlinux.org/packages/taskwarrior-tui-git/)

Use [pacman](https://wiki.archlinux.org/index.php/Pacman) to install it from the [community repository](https://archlinux.org/packages/community/x86_64/taskwarrior-tui/):

```bash
pacman -S taskwarrior-tui
```

Or use your favorite [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers) to download the [git](https://aur.archlinux.org/packages/taskwarrior-tui-git/) package maintained by [**@loki7990**](https://github.com/loki7990). For example:

```bash
yay -S taskwarrior-tui-git # build from source
```

**Using [`snap`](https://snapcraft.io/)** [![](https://snapcraft.io/taskwarrior-tui/badge.svg)](https://snapcraft.io/taskwarrior-tui)

```bash
snap install taskwarrior-tui
```

**Using [`zdharma-continuum/zinit`](https://github.com/zdharma-continuum/zinit)** [![](https://img.shields.io/github/v/tag/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

Add the following to your `~/.zshrc`:

```zsh
zinit ice wait:2 lucid extract"" from"gh-r" as"command" mv"taskwarrior-tui* -> tt"
zinit load kdheepak/taskwarrior-tui
```
