# hackernews-TUI
`hackernews_tui` is a Terminal UI to browse Hacker News written in Rust.

The application mainly consists of the following views:
- `Story View - Front Page` displaying a list of stories in front page of Hacker News.
- `Comment View` displaying a list of comments in a story.
- `Story Search View` displaying a search bar and a list of stories matching the search query.

## Installation
### Using cargo
Run `cargo install hackernews_tui` to install the application as a binary.
### Using archlinux AUR
Run `yay -S hackernews_tui` to install the application as an AUR package.

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

** Universal key shortcuts: **
- `<ctrl-h>`: Open the help dialog
- `<ctrl-s>`: Go to the story search
- `<ctrl-f>`: Go to the front page
- `<ctrl-q>`: Quit the application

In case the above shortcuts don't work, you can always use the corresponding buttons at the bottom of the `View`:
![Footer buttons](https://raw.githubusercontent.com/aome510/hackernews-TUI/main/examples/assets/footer_buttons.png)

** Key shortcuts for each `View`: **
#### StoryView
- `j`: Focus the next story
- `k`: Focus the previous story
- `t`: Focus the story at the top
- `b`: Focus the story at the bottom
- `{story_id} g`: Focus the {story_id}-th story
- `<enter>`: Go the comment view associated with the focused story
- `O`: Open in browser the link associated with the focused story

#### CommentView
- `j`: Focus the next comment
- `k`: Focus the previous comment
- `t`: Focus the comment at the top
- `b`: Focus the comment at the bottom
- `l`: Focus the next comment with smaller or equal level
- `h`: Focus the previous comment with smaller or equal level
- `O`: Open in browser the link associated with the discussed story
- `{link_id} f`: Open in browser the {link_id}-th link in the focused comment

#### SearchView
In `SearchView`, there are two modes: `Navigation` and `Search`. The default mode is `Search`.

`Search` mode is similar to Vim's Insert mode, in which users can input the query string.

`Navigation` mode allows the `SearchView` to behave like a `StoryView` with all `StoryView` shortcuts enabled.

Switch mode key shortcuts:
- `i`: Enter `Search` mode from `Navigation` mode
- `<esc>`: Enter `Navigation` mode from `Search` mode

`Navigation` mode supports all `StoryView`'s key shortcuts.


## Roadmap/TODO List
TBA...
