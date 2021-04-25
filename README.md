# hackernews-TUI

`hackernews_tui` is a Terminal UI to browse Hacker News with vim-like key bindings.

`hackernews_tui` is written in Rust with the help of [Cursive TUI library](https://github.com/gyscos/cursive/). It uses [HN Algolia search APIs](https://hn.algolia.com/api/) to get Hacker News data.

The application mainly consists of the following views:

- `Story View` displaying a list of stories. There are different kinds of `Story View` depending on the `tag` it uses to filter stories:
  - `Front Page`: stories on the front-page
  - `All Stories`: all stories
  - `Ask HN`: ask HN stories only
  - `Show HN`: show HN stories only
  - `Jobs`: jobs stories only
- `Comment View` displaying a list of comments in a story.
- `Serrch View` displaying a search bar and a list of stories matching the search query.

### Why hackernews-TUI?

If you are either

- a Hacker News reader
- a computer nerd who likes doing things on terminal
- a vim key-bindings fan boy
- a person who prefers navigating using keyboard to using mouse

This application is a right tool for you :muscle:

### Table of Contents

- [Install](#install)
- [Examples](#examples)
  - [Demo](#demo)
- [Shortcuts](#shortcuts)
  - [Global key shortcuts](#global-key-shortcuts)
  - [Key shortcuts for each View](#key-shortcuts-for-each-view)
    - [Story View](#story-view-shortcuts)
    - [Comment View](#comment-view-shortcuts)
    - [Search View](#search-view-shortcuts)
- [Configuration](#configuration)

## Install

### Using cargo

#### Install the latest version from [crates.io](https://crates.io/crates/hackernews_tui)

Run `cargo install hackernews_tui` to install the application as a binary.

#### Build from the `master` branch

Run

```shell
# git clone https://github.com/aome510/hackernews-TUI.git
# cd hackernews-TUI
# cargo build --release
```

to build the application then run

```shell
# ./target/release/hackernews_tui
```

to run the application

### Arch Linux

Run `yay -S hackernews_tui` to install the application as an AUR package.

### NetBSD

#### Using the package manager

```shell
# pkgin install hackernews-tui
```

#### Building from source

```shell
$ cd /usr/pkgsrc/www/hackernews-tui
# make install
```

## Examples

### Demo

A quick demo video made from `hackernews_tui` version `v0.5.0`:
![Example demo video 1](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/v0-5-demo-1.gif)

![Example demo video 2](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/v0-5-demo-2.gif)

### Story View

![Example of a Story View - Front Page](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_view.png)

### Search View

![Example of a Search View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_search_view.png)

### Comment View

![Example of a Comment View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/comment_view.png)

## Shortcuts

In each `View`, press `?/<ctrl-h>/<alt-h>` to see a list of supported keyboard shortcuts and their functionalities:

![Example of a Help View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/help_view.png)

### Global key shortcuts

- `?/<ctrl-h>/<alt-h>`: Open the help dialog
- `<ctrl-q>/<alt-q>`: Quit the application
- `<ctrl-p>/<alt-p>`: Go to the previous view
- `<ctrl-f>/<alt-f>`: Go to front page view
- `<ctrl-s>/<alt-s>`: Go to search view
- `<ctrl-z>/<alt-z>`: Go to all stories view
- `<ctrl-x>/<alt-x>`: Go to ask HN view
- `<ctrl-c>/<alt-c>`: Go to show HN view
- `<ctrl-v>/<alt-v>`: Go to jobs view

### Key shortcuts for each `View`

#### Story View shortcuts

- `j`: Focus the next story
- `k`: Focus the previous story
- `t`: Focus the story at the top
- `b`: Focus the story at the bottom
- `{story_id} g`: Focus the {story_id}-th story
- `<enter>`: Go the comment view associated with the focused story
- `O`: Open in browser the link associated with the focused story
- `S`: Open in browser the focused story
- `n`: Go to the next page
- `p`: Go the previous page
- `d`: Toggle sort by date/popularity
- `q`: Filter stories past 24 hours
- `w`: Filter stories past week
- `e`: Filter stories past month
- `r`: Filter stories past year

#### Comment View shortcuts

- `j`: Focus the next comment
- `k`: Focus the previous comment
- `n`: Focus the next top level comment
- `p`: Focus the previous top level comment
- `l`: Focus the next comment with smaller or equal level
- `h`: Focus the previous comment with smaller or equal level
- `t`: Focus the comment at the top
- `b`: Focus the comment at the bottom
- `r`: Reload the comment view.
- `O`: Open in browser the link associated with the discussed story
- `S`: Open in browser the discussed story
- `C`: Open in browser the focused comment
- `{link_id} f`: Open in browser the {link_id}-th link in the focused comment

#### Search View shortcuts

In `SearchView`, there are two modes: `Navigation` and `Search`. The default mode is `Search`.

`Search` mode is similar to Vim's Insert mode, in which users can input the query string.

`Navigation` mode allows the `SearchView` to behave like a `StoryView` with all `StoryView` shortcuts enabled.

Key shortcuts:

- `i`: Enter `Search` mode from `Navigation` mode
- `<esc>`: Enter `Navigation` mode from `Search` mode

`Navigation` mode also supports all `StoryView`'s key shortcuts beside `q`,`w`,`e`,`r`.

## Configuration

By default, the application will look for `~/.config/hn-tui.toml` as its configuration file.

You can specify the path by specifying the `-c` or `--config` argument when running the application:

```shell
hackernews_tui -c ~/.config/hn-tui.toml
```

For the further information about the config options, please refer to the example config file by running `hackernews_tui --example-config`.

**Note**: all config options (as included in the example config file) are **required**. You can run

```shell
hackernews_tui --example-config > ~/.config/hn-tui.toml
```

then modify the config options in `~/.config/hn-tui.toml` based on your preferences.
