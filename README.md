# hackernews-TUI
`hackernews_tui` is a Terminal UI to browse Hacker News written in Rust.

The application mainly consists of two views:
- `Story View` displaying a list of top stories.
- `Comment View` displaying a list of comments in a story.

## Installation
### Using cargo
Run `cargo install hackernews_tui` to install the application as a binary.

## Examples

Story View:
![Example of a Story View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/story_view.png)

Comment View:
![Example of a Comment View](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/comment_view.png)

## Documentation
### Keyboard shortcuts
#### Story View
- `k`, `j` move the focus up and down between stories.
- `return` to enter the `Comment View` of the focused story.
- `O` to open the link associated with the focused story in the default browser.
- `q` to exit the application.
#### Comment View
- `O` to open the link associated with the discussed story in the default browser.
- `q` to move back to the `Story View`.
- `k`, `j` to move the focus up and down between comments.
- `l` to move the focus to the next comment with smaller or equal level as the currently focused comment.
- `h` to move the focus to the previous comment with smaller or equal level as the currently focused comment.
- `t` to move the the top of the view, `b` to move the bottom of the view.
- `{link_id} f` to open the the `{link_id}`th link in the focused comment.
For example, press `0` followed by `f` will open the first link in the currently focused comment.

## Roadmap/TODO List
TBA...
