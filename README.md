# hackernews-TUI

`hackernews_tui` is a Terminal UI to browse Hacker News with fully customizable and vim-like key bindings.

`hackernews_tui` is written in Rust with the help of [Cursive TUI library](https://github.com/gyscos/cursive/). It uses [HN Algolia search APIs](https://hn.algolia.com/api/) to get Hacker News data.

The application mainly consists of the following views:

- `Story View` displaying a list of HN stories. There are different kinds of `Story View` depending on the `tag` used to filter stories:
  - `Front Page`: stories on the front-page
  - `All Stories`: all stories
  - `Ask HN`: ask HN stories only
  - `Show HN`: show HN stories only
  - `Jobs`: jobs stories only
- `Article View` displaying the content of a web article in reader mode.
- `Comment View` displaying a list of comments in a story.
- `Search View` displaying a search bar and a list of stories matching the search query.

### Why hackernews-TUI?

If you are either

- a Hacker News reader
- a computer nerd who likes doing things on terminal
- a vim key-bindings fanboy
- a person who prefers navigating using the keyboard over a mouse

This application is the right tool for you :muscle:

### Table of Contents

- [Install](#install)
  - [Dependencies](#dependencies)
  - [Using Cargo](#using-cargo)
  - [Arch Linux](#arch-linux)
  - [NetBSD](#netbsd)
- [Examples](#examples)
  - [Demo](#demo)
- [Default Shortcuts](#default-shortcuts)
  - [Global key shortcuts](#global-key-shortcuts)
  - [Key shortcuts for each View](#key-shortcuts-for-each-view)
    - [Story View](#story-view-shortcuts)
    - [Article View](#article-view-shortcuts)
    - [Comment View](#comment-view-shortcuts)
    - [Search View](#search-view-shortcuts)
- [Configuration](#configuration)
  - [User-defined shortcuts](#user-defined-shortcuts)
- [Debug](#debug)
- [Roadmap](#roadmap)

## Install

### Dependencies

#### Mercury Parser

To enable viewing a web page in reader mode with `Article View`, install [`mercury-parser`](https://github.com/postlight/mercury-parser) globally by running

```shell
# using yarn
yarn global add @postlight/mercury-parser

# or using npm
npm install -g @postlight/mercury-parser
```

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

to build the application, then run

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

List of demo videos:

- `hackernews_tui v0.5.0`: demo general usage of `Story View`, `Search View` and `Comment View` can be found [here](https://www.youtube.com/watch?v=AArtVod0b6A)
- `hackernews_tui v0.6.0-beta`: demo the usage of `Article View` to read a web page in reader mode can be found [here](https://www.youtube.com/watch?v=jIsKZwPi2T8)

### Story View

![Example of a Story View - Front Page](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_view.png)

### Article View

![Example of a Story View - Front Page](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/article_view.png)

### Search View

![Example of a Search View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_search_view.png)

### Comment View

![Example of a Comment View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/comment_view.png)

## Default Shortcuts

In each `View`, press `?` to see a list of supported keyboard shortcuts and their functionalities:

![Example of a Help View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/help_view.png)

### Global key shortcuts

- `?`: Open the help dialog
- `esc`: Close a dialog
- `C-q`: Quit the application
- `C-p`: Go to the previous view
- `C-f`: Go to front page view
- `C-s`: Go to search view
- `C-z`: Go to all stories view
- `C-x`: Go to ask HN view
- `C-c`: Go to show HN view
- `C-v`: Go to jobs view

### Key shortcuts for each `View`

#### Story View shortcuts

- `j`: Focus the next story
- `k`: Focus the previous story
- `{story_id} g`: Focus the {story_id}-th story
- `enter`: Go the comment view associated with the focused story
- `o`: Open in browser the article associated with the focused story
- `O`: Open in article view the article associated with the focused story
- `s`: Open in browser the focused story
- `n`: Go to the next page
- `p`: Go the previous page
- `d`: Toggle sort by date/popularity

#### Article View shortcuts

- `k`: Scroll up
- `j`: Scroll down
- `u`: Scroll up half a page
- `d`: Scroll down half a page
- `t`: Scroll to top
- `b`: Scroll to bottom
- `o`: Open article in browser
- `{link_id} f`: Open in browser {link_id}-th link
- `{link_id} F`: Open in article view {link_id}-th link

##### Link dialog shortcuts

- `l`: Open link dialog
- `j`: Focus next link
- `k`: Focus previous link
- `f`: Open in browser the focused link
- `F`: Open in article view the focused link

#### Comment View shortcuts

- `j`: Focus the next comment
- `k`: Focus the previous comment
- `n`: Focus the next top level comment
- `p`: Focus the previous top level comment
- `l`: Focus the next comment with smaller or equal level
- `h`: Focus the previous comment with smaller or equal level
- `r`: Reload the comment view.
- `up`: Scroll up
- `down`: Scroll down
- `page_up`: Scroll up half a page
- `page_down`: Scroll down half a page
- `o`: Open in browser the article associated with the discussed story
- `O`: Open in article view the article associated with the discussed story
- `s`: Open in browser the discussed story
- `c`: Open in browser the focused comment
- `{link_id} f`: Open in browser the {link_id}-th link in the focused comment
- `{link_id} F`: Open in article view the {link_id}-th link in the focused comment

#### Search View shortcuts

In `SearchView`, there are two modes: `Navigation` and `Search`. The default mode is `Search`.

`Search` mode is similar to Vim's Insert mode, in which users can input the query string.

`Navigation` mode allows the `SearchView` to behave like a `StoryView` with all `StoryView` shortcuts enabled.

Key shortcuts:

- `i`: Enter `Search` mode from `Navigation` mode
- `<esc>`: Enter `Navigation` mode from `Search` mode

`Navigation` mode also supports a subset of `StoryView`'s key shortcuts.

## Configuration

By default, the application will look for `~/.config/hn-tui.toml` as its configuration file.

You can specify the path by specifying the `-c` or `--config` argument when running the application:

```shell
hackernews_tui -c ~/.config/hn-tui.toml
```

For further information about the config options, please refer to the example config file by running `hackernews_tui --example-config`.

**Note**: all config options (as included in the example config file) are **required**. You can run

```shell
hackernews_tui --example-config > ~/.config/hn-tui.toml
```

then modify the config options in `~/.config/hn-tui.toml` based on your preferences.

### User-defined shortcuts

Shortcuts in each `View` are full customizable, for futher information about the supported keys and the corresponding functionalities, please refer to the **user-defined key bindings** sections in the example config file by running `hackernews_tui --example-config`.

## Debug

Run

```shell
RUST_LOG=debug hackernews_tui 2> log.txt
```

to view the application's log in `log.txt` file.

## Roadmap

- [x] make all commands customizable
- [x] add a `View` to read the linked story in reader mode on the terminal. A list of possible suggestion can be found [here](https://news.ycombinator.com/item?id=26930466)
- [ ] add commands to navigate parent comments and collapse a comment
- [ ] integrate [HackerNews Official APIs](https://github.com/HackerNews/API) for real-time updating, lazy-loading comments, and sorting stories
- [ ] snipe-like navigation, inspired by [vim-snipe](https://github.com/yangmillstheory/vim-snipe)
- [ ] support more themes
- [ ] add some extra transition effects
