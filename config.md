# Configuration Documentation

User can change the application's configurations by modifying the user's config file (default to be `$HOME/.config/hn-tui.toml`).

**Note**: user doesn't need to specify all the options in the config file as a default value will be used for non-specified options.

An example of user-defined configuration file can be found in [example `hn-tui.toml`](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml).

## Table of Contents

- [General](#general)
  - [Article Parse Command](#article-parse-command)
- [Theme](#theme)
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

An `article_parse_command` must be a command that returns result of the following schema:

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

TBA

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
    by_date: string;
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
- `C-<char>` (ctrl character)
- `M-<char>` (alt character)
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
