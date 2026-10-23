## A wikipedia client with inline image/video and audio playback support

Using Rust (latest edition) and ratatui and any other widgets we may need to
make the TUI experience awesome design and architect a client for wiipedia with
support for caching articles and as well as logging in to wikipedia and editing
the articles.

### Features

- support for playing embedded media in all articles (query all the types of
  content and figure out deps we may need to preview them)
- support for syntax highlighting of code snippets
- rich rendering callouts, quotes, tables and other markdown formats
- support for link/article peeks on hovering over a link (similar to lsp hover
  in neovim)
- support for browsing bookmarked articles and adding them to bookmarks
- support for the daily articles and the rich feed provided by the wikipedia api
