# Configuration Documentation

User can change the application's configuration options by modifying the user-defined configuration file (default to be `$HOME/.config/hn-tui.toml`).

User doesn't need to specify all the configuration options in the configuration file as a default value will be used for non-specified options.

## Table of Contents

- [General](#general)
  - [Article Parse Command](#article-parse-command)
- [Theme](#theme)
- [Keymap](#keymap)
  - [Custom Keymap](#custom-keymap)

## General

| Option                  | Description                                                                                                                | Default                                                    |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `use_page_scrolling`    | whether to enable page-like behavior, which automatically adjust the view based on the scrolling direction, when scrolling | `true`                                                     |
| `use_pacman_loading`    | whether to use a pacman loading screen or a plain loading screen                                                           | `true`                                                     |
| `url_open_command`      | the command the application uses to open an url in browser                                                                 | `{ command: 'open', options: [] }`                         |
| `article_parse_command` | the command the application uses to parse an article into readable text                                                    | `{ command: 'article_md', options: ['--format', 'html'] }` |
| `client_timeout`        | the timeout (in seconds) when the application's makes an API request                                                       | `32`                                                       |

### Article Parse Command

TBA

## Theme

TBA

## Keymap

### Custom Keymap

`custom_keymaps` is a config option used to define custom shortcuts to navigate between different `StoryView` with stories filtered by certain conditions.

`custom_keymaps` consists of multiple custom keymaps, each has the following schema:

```
{
  key: string;
  tag: string;
  by_date: string;
  numeric_filters: {
    elapsed_days_interval: { start: number; end: number };
    points_interval: { start: number; end: number };
    num_comments_interval: { start: number; end: number };
  };
}
```

Possible values for `tag`

- `story`: All Stories
- `ask_hn`: Ask HN
- `show_hn`: Show HN
- `job`: Jobs

### Supported keys

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
