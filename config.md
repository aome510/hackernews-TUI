# Configuration Documentation

User can change the application's configurations by modifying the user-defined config file (default to be `$HOME/.config/hn-tui.toml`).

**Note**: user doesn't need to specify all the options in the config file as a default value will be used for non-specified options.

An example of user-defined configuration file can be found in [example `hn-tui.toml`](https://github.com/aome510/hackernews-TUI/blob/main/examples/hn-tui.toml).

## Table of Contents

- [General](#general)
  - [Article Parse Command](#article-parse-command)
- [Theme](#theme)
- [Keymap](#keymap)
  - [Custom Keymap](#custom-keymap)

## General

| Option                  | Description                                                                                                                | Default                                                    |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `use_page_scrolling`    | whether to enable page-like scrolling behavior, which automatically adjusts the view based on the scrolling direction | `true`                                                     |
| `use_pacman_loading`    | whether to use a pacman loading screen or a plain loading screen                                                           | `true`                                                     |
| `url_open_command`      | the command the application uses to open an url in browser                                                                 | `{ command: 'open', options: [] }`                         |
| `article_parse_command` | the command the application uses to parse an article into a readable text                                                    | `{ command: 'article_md', options: ['--format', 'html'] }` |
| `client_timeout`        | the timeout (in seconds) when the application's client makes an API request                                                       | `32`                                                       |

### Article Parse Command

TBA

## Theme

TBA

## Keymap

### Custom Keymap

`custom_keymaps` is a config option used to define custom shortcuts to navigate between different `StoryView` with stories filtered by certain conditions.

`custom_keymaps` has the following schema:

```typescript
let custom_keymaps: [
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
