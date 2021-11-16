# hackernews-TUI

`hackernews_tui` is a fast and customizable Terminal UI application for browsing Hacker News on terminal.

`hackernews_tui` is written in Rust with the help of [Cursive TUI library](https://github.com/gyscos/cursive/). It uses [HN Algolia search APIs](https://hn.algolia.com/api/) and [HN Official APIs](https://github.com/HackerNews/API) to get Hacker News data.

**Note**: `hackernews-tui` implements a lazy-loading comment functionality which only loads comments **on demand**. That means to load more comments, you should keep focusing the next comment passing the last visible comment until hitting the end when no additional comment is loaded.

The application consists of the following views:

- `Story View` displays a list of HN stories. There are different kinds of `Story View` depending on the `tag` used to filter stories:
  - `Front Page`: stories on the front page
  - `All Stories`: all stories
  - `Ask HN`: ask HN stories
  - `Show HN`: show HN stories
  - `Jobs`: jobs stories
- `Article View` displays the content of a web page in reader mode.
- `Comment View` displays a list of comments of a story.
- `Search View` displays a search bar and a list of stories matching the search query.

### Why hackernews-TUI?

If you are either

- a Hacker News reader
- a computer nerd who likes doing things on terminal
- a vim key-bindings fanboy
- a person who prefers navigating using the keyboard over a mouse

This application is the right tool for you :muscle:

### Table of Contents

- [Install](#install)
  - [Using Cargo](#using-cargo)
  - [Docker Image](#docker-image)
  - [Building](#building)
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
- [Debug](#debug)
- [Roadmap](#roadmap)

## Install

### Using cargo

#### Install the latest version from [crates.io](https://crates.io/crates/hackernews_tui)

Run `cargo install hackernews_tui` to install the application as a binary.

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

#### Building

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

to run the application. Or

```
ln -sf $PWD/target/release/hackernews_tui /usr/local/bin
```

to link the executable binary to `/usr/local/bin` folder.

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

### Edit key shortcuts

**Shortcuts** available in any editable texts.

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

`Navigation` mode allows the `SearchView` to behave like a `StoryView` with all `StoryView` shortcuts enabled.

Key shortcuts:

- `i`: Enter `Search` mode from `Navigation` mode
- `<esc>`: Enter `Navigation` mode from `Search` mode

`Navigation` mode also supports a subset of `StoryView`'s key shortcuts.

## Configuration

By default, the application will look for `~/.config/hn-tui.toml` as the configuration file.

You can also specify the path with the `-c` or `--config` option when running the application:

```
hackernews_tui -c ~/.config/hn-tui.toml
```

For further information about the configuration options, please refer to the example configuration file by running `hackernews_tui --example-config`.

### Article Parse Command

To enable viewing a web page in reader mode with `Article View`, you must configure the `article_parse_command` field in your configuration file:

````yaml
# `article_parse_command` defines a command to parse a web article's content
# to a markdown format. The parsed data is then used to render the `ArticleView`
# of the corresponding article.
#
# The command must have the following form:
# `<article_parse_command> [options...] <article_url>`
# It should return a JSON string representing the parsed `Article` data:
# ```
# pub struct Article {
#     title: String,
#     url: String,
#     content: String,
#     author: Option<String>,
#     date_published: Option<String>,
#     word_count: usize,
# }
# ```
article_parse_command = {command = "mercury-parser", options = ["--format", "markdown"]} // default value
# article_parse_command = {command = "article_md", options = []}
````

If you don't want to implement an article parser by your own, one way to configure `article_parse_command` is to use [`mercury-parser`](https://github.com/postlight/mercury-parser#installation), a web parser tool that `hackernews_tui` has been using by default since the version `0.6.0`. `mercury-parser` is powerful and stable. However, in some cases, the text content it returns when parsing HTML `code` tags has some weird indentations.

An alternative is to use [`article_md`](https://github.com/aome510/article-md-cli), a CLI tool I wrote for parsing web page's content into a markdown text. Under the hood, it uses [mozilla's readability](https://github.com/mozilla/readability), so the parsed text for HTML `code` tags look nicer.

### User-defined shortcuts

Shortcuts in each `View` are fully customizable. For further information about the supported keys and the commands, please refer to the **key bindings** section in the example configuration file.

### Custom Keymap

It's possible to define a custom shortcut to switch between different `StoryView` (`front_page`, `show_hn`, `ask_hn`, etc) with stories filtered by HN Algolia's [`numericFilters`](https://hn.algolia.com/api/). An example of defining such custom shortcuts can be found under the **custom keymap** section of the example configuration file.

## Debug

Run

```
RUST_LOG=debug RUST_BACKTRACE=1 hackernews_tui 2> log.txt
```

then run

```
tail -f log.txt
```

in another terminal to view the application's log in `log.txt` file.

## Roadmap

- [x] make all commands customizable
- [x] add a `View` to read the linked story in reader mode on the terminal. A list of possible suggestion can be found [here](https://news.ycombinator.com/item?id=26930466)
- [x] add commands to navigate parent comments and collapse a comment
- [x] make all the configuration options optional
- integrate [HackerNews Official APIs](https://github.com/HackerNews/API) for real-time updating, lazy-loading comments, and sorting stories
  - [x] lazy-loading comments
  - [x] front-page stories like the official site
  - [ ] real-time updating
- [ ] implement smarter lazy-loading comment functionality
- [ ] support more themes
- [ ] snipe-like navigation, inspired by [vim-snipe](https://github.com/yangmillstheory/vim-snipe)
- [ ] add some extra transition effects
