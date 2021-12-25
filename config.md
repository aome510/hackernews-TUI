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

| Option                  | Description                                                                                                                | Default |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- | ------- |
| `use_page_scrolling`    | whether to enable page-like behavior, which automatically adjust the view based on the scrolling direction, when scrolling | `true`        |
| `use_pacman_loading`    | whether to use a pacman loading screen or a plain loading screen                                                                                                                           | `true`         |
| `url_open_command`      | the command the application uses to open an url in browser                                                                                                                             | `{ command: 'open', options: [] }`        |
| `article_parse_command` | the command the application uses to parse an article into readable text                                                                                                                             |   `{ command: 'article_md', options: ['--format', 'html'] }`      |
| `client_timeout`        | the timeout (in seconds) when the application's makes an API request                                                                                                                            |       `32`  |

### Article Parse Command

TBA

## Theme

TBA

## Keymap

### Custom Keymap

TBA
