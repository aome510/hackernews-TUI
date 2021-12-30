# hackernews-TUI

`hackernews_tui` is a fast and customizable Terminal UI application for browsing Hacker News on the terminal.

`hackernews_tui` is written in Rust with the help of [Cursive TUI library](https://github.com/gyscos/cursive/). It uses [HN Algolia APIs](https://hn.algolia.com/api/) and [HN Official APIs](https://github.com/HackerNews/API) to get Hacker News data.

## Table of Contents

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

### Demo

Demo videos of `hackernews_tui` `v0.9.0` are available on [youtube](https://www.youtube.com/watch?v=m5O5QIlRFpc) and [asciinema](https://asciinema.org/a/459196)

[![asciicast](https://asciinema.org/a/459196.svg)](https://asciinema.org/a/459196)

### Story View

![Example of a Story View](https://user-images.githubusercontent.com/40011582/147393397-71991e48-cba6-4f89-9d28-cafbc0143c42.png)

### Article View

![Example of an Article View](https://user-images.githubusercontent.com/40011582/147393483-06b57c07-3fa3-49ec-b238-a2d67905610d.png)

### Search View

![Example of a Search View](https://user-images.githubusercontent.com/40011582/147393493-41d52d9f-65cd-4f63-bf76-c11d9bea1f49.png)

### Comment View

![Example of a Comment View](https://user-images.githubusercontent.com/40011582/147393459-641dd6c3-3564-472c-83cd-e1865339c861.png)

## Default Shortcuts

In each `View`, press `?` to see a list of supported keyboard shortcuts and their functionalities.

![Example of a Help View](https://user-images.githubusercontent.com/40011582/147393555-5ca346ca-b59a-4a7f-ab53-b1ec7025eca4.png)

The below sections will list the application's default shortcuts, which can be customized by changing the [key mappings](https://github.com/aome510/hackernews-TUI/blob/main/config.md#keymap) in the user's config file.

For more information about configuring the key mapping or defining custom shortcuts, please refer to the [config documentation](https://github.com/aome510/hackernews-TUI/blob/main/config.md#keymap).

### Global key shortcuts

| Command                 | Description             | Default Shortcut |
| ----------------------- | ----------------------- | ---------------- |
| `open_help_dialog`      | Open the help dialog    | `?`              |
| `close_dialog`          | Close a dialog          | `esc`            |
| `quit`                  | Quit the application    | `C-q`            |
| `goto_previous_view`    | Go to the previous view | `C-p`            |
| `goto_search_view`      | Go to search view       | `C-s`            |
| `goto_front_page_view`  | Go to front page view   | `F1`             |
| `goto_all_stories_view` | Go to all stories view  | `F2`             |
| `goto_ask_hn_view`      | Go to ask HN view       | `F3`             |
| `goto_show_hn_view`     | Go to show HN view      | `F4`             |
| `goto_jobs_view`        | Go to jobs view         | `F5`             |

### Edit key shortcuts

| Command                | Description                      | Default Shortcut |
| ---------------------- | -------------------------------- | ---------------- |
| `move_cursor_left`     | Move cursor to left              | `left`           |
| `move_cursor_right`    | Move cursor to right             | `right`          |
| `move_cursor_to_begin` | Move cursor to the begin of line | `home`           |
| `move_cursor_to_end`   | Move cursor to the end of line   | `end`            |
| `backward_delete_char` | Delete backward a character      | `backspace`      |

### Key shortcuts for each `View`

#### Story View shortcuts

| Command                        | Description                                                        | Default Shortcut |
| ------------------------------ | ------------------------------------------------------------------ | ---------------- |
| `next_story`                   | Focus the next story                                               | `j`              |
| `prev_story`                   | Focus the previous story                                           | `k`              |
| `next_story_tag`               | Go to the next story tag                                           | `l`              |
| `previous_story_tag`           | Go to the previous story tag                                       | `h`              |
| `goto_story`                   | Focus the {story_id}-th story                                      | `{story_id} g`   |
| `goto_story_comment_view`      | Go the comment view associated with the focused story              | `enter`          |
| `open_article_in_browser`      | Open in browser the article associated with the focused story      | `o`              |
| `open_article_in_article_view` | Open in article view the article associated with the focused story | `O`              |
| `open_story_in_browser`        | Open in browser the focused story                                  | `s`              |
| `next_page`                    | Go to the next page                                                | `n`              |
| `prev_page`                    | Go the previous page                                               | `p`              |
| `toggle_sort_by_date`          | Toggle sort stories by date                                        | `d`              |

#### Article View shortcuts

| Command                     | Description                            | Default Shortcut |
| --------------------------- | -------------------------------------- | ---------------- |
| `up`                        | Scroll up                              | `k`              |
| `down`                      | Scroll down                            | `j`              |
| `page_down`                 | Scroll up half a page                  | `u`              |
| `page_up`                   | Scroll down half a page                | `d`              |
| `top`                       | Scroll to top                          | `g`              |
| `bottom`                    | Scroll to bottom                       | `G`              |
| `open_article_in_browser`   | Open article in browser                | `o`              |
| `open_link_in_browser`      | Open in browser {link_id}-th link      | `{link_id} f`    |
| `open_link_in_article_view` | Open in article view {link_id}-th link | `{link_id} F`    |
| `open_link_dialog`          | Open link dialog                       | `l`              |

##### Link dialog shortcuts

| Command                     | Description                           | Default Shortcut |
| --------------------------- | ------------------------------------- | ---------------- |
| `link_dialog_focus_next`    | Focus next link                       | `j`              |
| `link_dialog_focus_prev`    | Focus previous link                   | `k`              |
| `open_link_in_browser`      | Open in browser the focused link      | `f`              |
| `open_link_in_article_view` | Open in article view the focused link | `F`              |

#### Comment View shortcuts

| Command                     | Description                                                          | Default Shortcut |
| --------------------------- | -------------------------------------------------------------------- | ---------------- |
| `next_comment`              | Focus the next comment                                               | `j`              |
| `prev_comment`              | Focus the previous comment                                           | `k`              |
| `next_leq_level_comment`    | Focus the next comment with smaller or equal level                   | `l`              |
| `prev_leq_level_comment`    | Focus the previous comment with smaller or equal level               | `h`              |
| `next_top_level_comment`    | Focus the next top level comment                                     | `n`              |
| `prev_top_level_comment`    | Focus the previous top level comment                                 | `p`              |
| `parent_comment`            | Focus the parent comment (if exists)                                 | `u`              |
| `toggle_collapse_comment`   | Toggle collapsing the focused comment                                | `tab`            |
| `up`                        | Scroll up                                                            | `up`             |
| `down`                      | Scroll down                                                          | `down`           |
| `page_up`                   | Scroll up half a page                                                | `page_up`        |
| `page_down`                 | Scroll down half a page                                              | `page_down`      |
| `open_article_in_browser`   | Open in browser the article associated with the discussed story      | `o`              |
| `open_link_in_article_view` | Open in article view the article associated with the discussed story | `O`              |
| `open_story_in_browser`     | Open in browser the discussed story                                  | `s`              |
| `open_comment_in_browser`   | Open in browser the focused comment                                  | `c`              |
| `open_link_in_browser`      | Open in browser the {link_id}-th link in the focused comment         | `{link_id} f`    |
| `open_link_in_article_view` | Open in article view the {link_id}-th link in the focused comment    | `{link_id} F`    |

#### Search View shortcuts

In `SearchView`, there are two modes: `Navigation` and `Search`. The default mode is `Search`.

`Search` mode is similar to Vim's insert mode, in which users can input a query string.

`Navigation` mode allows the `SearchView` to behave like a `StoryView` of matched stories with a subset of `StoryView` shortcuts enabled.

`SearchView`-specific key shortcuts:

| Command              | Description                                | Default Shortcut |
| -------------------- | ------------------------------------------ | ---------------- |
| `to_search_mode`     | Enter `Search` mode from `Navigation` mode | `i`              |
| `to_navigation_mode` | Enter `Navigation` mode from `Search` mode | `<esc>`          |

## Configuration

By default, `hackernews-tui` will look for `~/.config/hn-tui.toml` as the configuration file, which can be configured by specifying the path with the `-c` or `--config` option when running the application:

```
hackernews_tui -c ~/.config/hn-tui.toml
```

For further information about the configuration options, please refer to the [example configuration file](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml)

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
