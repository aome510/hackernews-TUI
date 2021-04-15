# hackernews-TUI

`hackernews_tui` is a Terminal UI to browse Hacker News.

`hackernews_tui` is written in Rust with the help of [Cursive TUI library](https://github.com/gyscos/cursive/). It uses [HN Algolia search APIs](https://hn.algolia.com/api/) to get Hacker News data.

The application mainly consists of the following views:

- `Story View - Front Page` displaying a list of stories in front page of Hacker News.
- `Comment View` displaying a list of comments in a story.
- `Story Search View` displaying a search bar and a list of stories matching the search query.

### Table of Contents

- [Install](#install)
- [Examples](#examples)
- [Documentation](#documentation)
- [Configuration](#configuration)

## Install

### Using cargo

Run `cargo install hackernews_tui` to install the application as a binary.

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

Story View - Front Page:
![Example of a Story View - Front Page](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_view.png)

Comment View:
![Example of a Comment View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/comment_view.png)

Story Search View
![Example of a Story Search View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_search_view.png)

## Documentation

### Keyboard shortcuts

In each `View`, press `<ctrl-h>` to see a list of supported keyboard shortcuts and their functionalities.

**Global key shortcuts:**

- `<ctrl-h>\<alt-h>`: Open the help dialog
- `<ctrl-s>\<alt-s>`: Go to the story search
- `<ctrl-f>\<alt-f>`: Go to the front page
- `<ctrl-q>\<alt-q>`: Quit the application

In case the above shortcuts don't work, you can always use the corresponding buttons at the bottom of the `View`:
![Footer buttons](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/footer_buttons.png)

**Key shortcuts for each `View`:**

#### StoryView

- `j`: Focus the next story
- `k`: Focus the previous story
- `t`: Focus the story at the top
- `b`: Focus the story at the bottom
- `{story_id} g`: Focus the {story_id}-th story
- `<enter>`: Go the comment view associated with the focused story
- `O`: Open in browser the link associated with the focused story
- `S`: Open in browser the focused story

#### CommentView

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

#### SearchView

In `SearchView`, there are two modes: `Navigation` and `Search`. The default mode is `Search`.

`Search` mode is similar to Vim's Insert mode, in which users can input the query string.

`Navigation` mode allows the `SearchView` to behave like a `StoryView` with all `StoryView` shortcuts enabled.

Switch mode key shortcuts:

- `i`: Enter `Search` mode from `Navigation` mode
- `<esc>`: Enter `Navigation` mode from `Search` mode

`Navigation` mode supports all `StoryView`'s key shortcuts.

## Configuration

By default, the application will look for `~/.config/hn-tui.toml` as its configuration file.

You can specify the path by specifying the `-c` or `--config` argument when running the application:

```shell
hackernews_tui -c ~/.config/hn-tui.toml
```

For the further information about the config options, please look into the [example config file](./examples/hn-tui.toml) for references.

**Note**: all config options (as included in the example config file) are **required**. You can download the example file then modify it based on your needs.
