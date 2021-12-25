# hackernews-TUI

`hackernews_tui` is a fast and customizable Terminal UI application for browsing Hacker News on the terminal.

`hackernews_tui` is written in Rust with the help of [Cursive TUI library](https://github.com/gyscos/cursive/). It uses [HN Algolia APIs](https://hn.algolia.com/api/) and [HN Official APIs](https://github.com/HackerNews/API) to get Hacker News data.

### Why hackernews-TUI?

If you are either

- a Hacker News reader
- a computer nerd who likes doing things on the terminal
- a person who prefers navigating using the keyboard over a mouse

This application is the right tool for you :muscle:

### Table of Contents

- [Install](#install)
  - [Using Cargo](#using-cargo)
  - [Docker Image](#docker-image)
  - [Building from source](#building-from-source)
  - [macOS](#macos)
  - [Arch Linux](#arch-linux)
  - [NetBSD](#netbsd)
- [Examples](#examples)
  - [Demo](#demo)
- [Default Shortcuts](#default-shortcuts)
  - [Global key shortcuts](#global-key-shortcuts)
  - [Edit key shortcuts](#edit-key-shortcuts)
  - [Key shortcuts for each View](#key-shortcuts-for-each-view)
    - [Story View](#story-view-shortcuts)
    - [Article View](#article-view-shortcuts)
    - [Comment View](#comment-view-shortcuts)
    - [Search View](#search-view-shortcuts)
- [Configuration](#configuration)
  - [Article Parse Command](#article-parse-command)
  - [User-defined shortcuts](#user-defined-shortcuts)
  - [Custom keymap](#custom-keymap)
- [Logging](#logging)
- [Roadmap](#roadmap)

## Install

### Using cargo

Install the latest version from [crates.io](https://crates.io/crates/hackernews_tui) by running `cargo install hackernews_tui`

### Docker image

You can download the binary image of the latest build from the `master` branch by running

```
# docker pull aome510/hackernews_tui:latest
```

then run

```
# docker run -it aome510/hackernews_tui:latest
```

to run the application. You can also use your local [configuration file](#configuration) when running the image by running

```
# docker run --rm -v ${CONFIG_FILE_PATH}:/app/hn-tui.toml -it aome510/hackernews_tui:latest
```

with `${CONFIG_FILE_PATH}` is your local configuration file path.

#### Building from source

Run

```
git clone https://github.com/aome510/hackernews-TUI.git
cd hackernews-TUI
cargo build --release
```

to build the application, then run

```
./target/release/hackernews_tui
```

to run the application, or

```
ln -sf $PWD/target/release/hackernews_tui /usr/local/bin
```

to link the executable binary to `/usr/local/bin` folder.

### macOS

#### Via MacPorts

Run `sudo port install hackernews-tui` to install the application.

### Arch Linux

Run `yay -S hackernews_tui` to install the application as an AUR package.

### NetBSD

#### Using the package manager

```
pkgin install hackernews-tui
```

#### Building from source

```
$ cd /usr/pkgsrc/www/hackernews-tui
# make install
```

## Examples

### Story View

![Example of a Story View](https://user-images.githubusercontent.com/40011582/147393397-71991e48-cba6-4f89-9d28-cafbc0143c42.png)

### Article View

![Example of an Article View](https://user-images.githubusercontent.com/40011582/147393483-06b57c07-3fa3-49ec-b238-a2d67905610d.png)

### Search View

![Example of a Search View](https://user-images.githubusercontent.com/40011582/147393493-41d52d9f-65cd-4f63-bf76-c11d9bea1f49.png)

### Comment View

![Example of a Comment View](https://user-images.githubusercontent.com/40011582/147393459-641dd6c3-3564-472c-83cd-e1865339c861.png)

## Default Shortcuts

In each `View`, press `?` to see a list of supported keyboard shortcuts and their functionalities. Note that the shortcuts are fully [customizable](#user-defined-shortcuts).

![Example of a Help View](https://user-images.githubusercontent.com/40011582/147393555-5ca346ca-b59a-4a7f-ab53-b1ec7025eca4.png)

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

### Edit key shortcuts

**Shortcuts** only available in an editable text.

- `left`: Move cursor to left
- `right`: Move cursor to right
- `home`: Move cursor to the begin of line
- `end`: Move cursor to the end of line
- `backspace`: Delete backward character

### Key shortcuts for each `View`

#### Story View shortcuts

- `j`: Focus the next story
- `k`: Focus the previous story
- `{story_id} g`: Focus the {story_id}-th story
- `enter`: Go the comment view associated with the focused story
- `l`: Go to the next story tag
- `h`: Go to the previous story tag
- `o`: Open in browser the article associated with the focused story
- `O`: Open in article view the article associated with the focused story
- `s`: Open in browser the focused story
- `n`: Go to the next page
- `p`: Go the previous page
- `d`: Toggle sort by date

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
- `u`: Focus the parent comment (if exists)
- `tab`: Toggle collapsing the focused comment
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

`Search` mode is similar to Vim's insert mode, in which users can input the query string.

`Navigation` mode allows the `SearchView` to behave like a `StoryView` with a subset of `StoryView` shortcuts enabled.

Key shortcuts:

- `i`: Enter `Search` mode from `Navigation` mode
- `<esc>`: Enter `Navigation` mode from `Search` mode

## Configuration

By default, `hackernews-tui` will look for `~/.config/hn-tui.toml` as the configuration file, which can be configured by specifying the path with the `-c` or `--config` option when running the application:

```
hackernews_tui -c ~/.config/hn-tui.toml
```

For further information about the configuration options, please refer to the [example configuration file](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml)

### Article Parse Command

TBA

### User-defined shortcuts

Shortcuts in each `View` are fully customizable. For further information about the supported keys and the commands, please refer to the **key bindings** section in the [example configuration file](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml).

### Custom Keymap

It's possible to define a custom shortcut to switch between different `StoryView` (`front_page`, `show_hn`, `ask_hn`, etc) with stories filtered by HN Algolia's [`numericFilters` filter option](https://hn.algolia.com/api/). An example of defining such custom shortcuts can be found under the **custom keymap** section of the [example configuration file](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml).

## Logging

`hackernews-tui` uses `RUST_LOG` environment variable to define the application's [logging level](https://docs.rs/log/0.4.14/log/enum.Level.html) (default to be `INFO`). The application stores logs inside the `$HOME/.cache/hn-tui.log` file, which can be configured by specifying the `-l` or `--log` option.

## Roadmap

- [x] make all commands customizable
- [x] add a `View` to read the linked story in reader mode on the terminal. A list of possible suggestion can be found [here](https://news.ycombinator.com/item?id=26930466)
- [x] add commands to navigate parent comments and collapse a comment
- [x] make all the configuration options optional
- integrate [HackerNews Official APIs](https://github.com/HackerNews/API) for real-time updating, lazy-loading comments, and sorting stories
  - [x] lazy-loading comments
  - [x] front-page stories like the official site
  - [ ] real-time updating
- [x] implement smarter lazy-loading comment functionality
- add crediential support to allow
  - [ ] upvote/downvote
  - [ ] add comment
  - [ ] post
- improve application's theme
  - [x] improve the application's overall look
  - [x] include useful font-highliting
  - [x] rewrite the theme parser to support more themes and allow to parse themes from known colorschemes
  - [ ] add some extra transition effects
- improve the keybinding handler
  - [ ] allow to bind multiple keys to a single command
  - [ ] add prefix key support (emacs-like key chaining - `C-x C-c ...`)
- [ ] improve the loading progress bar
- [ ] snipe-like navigation, inspired by [vim-snipe](https://github.com/yangmillstheory/vim-snipe)
