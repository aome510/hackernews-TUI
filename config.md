# Configuration Documentation

User can change the application's configurations by modifying the user's config file (default to be `$HOME/.config/hn-tui.toml`).

**Note**: user doesn't need to specify all the options in the config file as a **default** value will be used for non-specified options.

An example config file (with some default config values) can be found in [example `hn-tui.toml`](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml).

## Table of Contents

- [General](#general)
  - [Article Parse Command](#article-parse-command)
- [Theme](#theme)
  - [Default Theme](#default-theme)
  - [Palette](#palette)
  - [Component Style](#component-style)
- [Keymap](#keymap)
  - [Custom Keymap](#custom-keymap)
  - [Supported Keys](#supported-keys)

## General

| Option                  | Description                                                                                                           | Default                                                    |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `use_page_scrolling`    | whether to enable page-like scrolling behavior, which automatically adjusts the view based on the scrolling direction | `true`                                                     |
| `use_pacman_loading`    | whether to use a pacman loading screen or a plain loading screen                                                      | `true`                                                     |
| `url_open_command`      | the command the application uses to open an url in browser                                                            | `{ command: 'open', options: [] }`                         |
| `article_parse_command` | the command the application uses to parse an article into a readable text                                             | `{ command: 'article_md', options: ['--format', 'html'] }` |
| `client_timeout`        | the timeout (in seconds) when the application's client makes an API request                                           | `32`                                                       |

### Article Parse Command

To enable viewing an article's content in reader mode with `ArticleView`, user will need to install **additional** tools and specify the `article_parse_command` config option.

An `article_parse_command` must be a command that returns result with the following schema:

```typescript
type result_schema = {
  content: string;
  url: string;
  title: string;
  author: string | null;
  date_published: string | null;
};
```

The returned `content` **must** be a **HTML string** respresenting the article's content in reader mode.

By default, `hackernews-TUI` uses [`article_md`](https://github.com/aome510/article-md-cli) as the default command for parsing articles.

One alternative is [`mercury-parser`](https://github.com/postlight/mercury-parser#installation):

```toml
article_parse_command = { command: 'mercury-parser', options: [] }
```

## Theme

An application's theme has two components:

1. `palette`: the theme's color palette
2. `component_style`: styles for application's components

### Default theme

The default theme configurations:

```toml
[theme.palette]
background = "#f6f6ef"
foreground = "#242424"
selection_background = "#d8dad6"
selection_foreground = "#4a4c4c"
black = "#000000"
blue = "#0000aa"
cyan = "#00aaaa"
green = "#00aa00"
magenta = "#aa00aa"
red = "#aa0000"
white = "#aaaaaa"
yellow = "#aaaa00"
light_black = "#555555"
light_white = "#ffffff"
light_red = "#ff5555"
light_magenta = "#5555ff"
light_green = "#55ff55"
light_cyan = "#55ffff"
light_blue = "#5555ff"
light_yellow = "#ffff55"

[theme.component_style]
# styles for application's specific components
title_bar = { back = "#ff6600", effect = "bold" }
matched_highlight = { front = "black", back = "#ffff55"}
metadata = { front = "#828282" }
username = { effect = "bold" }
loading_bar = { front = "light yellow", back = "blue"}

# general component styles
header = { front = "black", effect = "bold" }
quote = { front = "#677280" }
italic = { effect = "italic" }
bold = { effect = "bold" }
single_code_block = { front = "black", back = "#c8c8c8"}
multiline_code_block = { front = "light black", effect = "bold" }
link = { front = "#4fbbfd" }
link_id = { front = "black", back = "#ffff55"}

# story tag styles
current_story_tag = { front = "light white" }
ask_hn = { front = "red", effect = "bold" }
tell_hn = { front = "yellow", effect = "bold" }
show_hn = { front = "blue", effect = "bold" }
launch_hn = { front = "green", effect = "bold" }
```

### Palette

A theme's color palette is based on the 4-bit ANSI terminal colors.

User can change the default color palette by changing the values of any of the following fields

- `background`
- `foreground`
- `selection_background`
- `selection_foreground`
- `black`
- `blue`
- `cyan`
- `green`
- `magenta`
- `red`
- `white`
- `yellow`
- `light_black`
- `light_blue`
- `light_cyan`
- `light_green`
- `light_magenta`
- `light_red`
- `light_white`
- `light_yellow`

Each field's value can be either a **raw hex string** representing the color (`0xf6f6ef`, `#f6f6ef`, `f6f6ef`) or a **16-bit color's name** (`black`, `dark black`, `light black`).

Specifying the 16-bit color's name will use **the terminal's default color**. For example, `background = "black"` will make the application's background be in the terminal's default black color.

### Component Style

The application defines styles for some components. For example, any links in `hackernews-TUI` has the following style by default: `link = { front = "#4fbbfd" }`.

A style has 3 **optional** fields: `front` (foreground color), `back` (background color), `effect` (additional terminal effect).

- `front` and `back` can be either a **raw hex string** representing the color (`0xf6f6ef`, `#f6f6ef`, `f6f6ef`) or a **16-bit color's name** (`black`, `dark black`, `light black`).

Specifying the 16-bit color's name will use **the theme palette's color** (as opposed to **the terminal's default color** when configuring the theme's palette color). For example, `link = { back = "blue" }` will make any links in the application use the theme palette's blue color as the background color.

- `effect` can be only **one** of the following values
  - `simple`
  - `reverse`
  - `bold`
  - `italic`
  - `strikethrough`
  - `underline`
  - `blink`

## Keymap

To modify the [default key mapping](https://github.com/aome510/hackernews-TUI#default-shortcuts), simply add new mapping entries to the corresponding keymap section. For example, to change the key shortcuts for the command `next_comment` to `J` and the command `prev_comment` to `K` in the comment view, add these 3 lines to the config file:

```toml
[keymap.comment_view_keymap]
next_comment = "J"
prev_comment = "K"
```

### Custom Keymap

`custom_keymaps` is a config option used to define custom shortcuts to navigate between different story views with stories filtered by certain conditions.

`custom_keymaps` has the following schema:

```typescript
type custom_keymaps_schema = [
  {
    key: string;
    tag: "story" | "ask_hn" | "show_hn" | "job";
    by_date: bool;
    numeric_filters: {
      elapsed_days_interval: { start: number; end: number };
      points_interval: { start: number; end: number };
      num_comments_interval: { start: number; end: number };
    };
  }
];
```

An example of defining such custom keymaps can be found in the [example configuration file](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml).

### Supported keys

List of supported keys for mapping:

- `<char>` (any single character)
- `C-<char>` (ctrl + character)
- `M-<char>` (alt + character)
- `enter`
- `tab`
- `backspace`
- `esc`
- `left`
- `right`
- `up`
- `down`
- `ins`
- `del`
- `home`
- `end`
- `page_up`
- `page_down`
- `f1`
- `f2`
- `f3`
- `f4`
- `f5`
- `f6`
- `f7`
- `f8`
- `f9`
- `f10`
- `f11`
- `F12`
