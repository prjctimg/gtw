# Themes, wordmark, and effects

## Wordmark

- Crate: **`tui-big-text` 0.8.10** — the Ratatui big-text widget. The name
  `ratatui-big-text` is a reserved `0.0.0` placeholder, not an installable
  widget.
- Text: **`gtw`** (lowercase; the `gtm` source used lowercase `gtm`).
- Port the size-fitting logic from `gtm`’s `ui/bigtext.rs`:
  `Full → HalfHeight → HalfWidth → Quadrant → ThirdHeight → QuarterHeight →
  Octant`, choosing the largest size that fits the available width and height.
  Never clip the wordmark.
- Colors: a horizontal gradient through `accent`, `secondary_accent`, and
  `tertiary_accent`, with a slow phase offset. Keep the logo readable in
  monochrome by using brightness variation instead of hue.
- Placement:
  - first-run/home: centered hero with subtitle “Wikipedia in your terminal”;
  - normal home: compact at the top of the sidebar;
  - small terminals (< 60 columns): plain text `gtw`, no big-text.
- Startup effect: 500 ms `coalesce`/`sweep_in`; skip when
  `effects = "never"` or reduced motion is set.

## Tachyonfx integration

- Crate: **`tachyonfx` 0.25.2**, compatible with ratatui 0.30.
- One `EffectManager` owned by `App`.
- The event loop calls `process_effects(Duration::from_millis(16), &mut buf,
  area)` only when effects are active; the tick rate is 60 fps while an effect
  runs and idle otherwise.
- Effects are keyed by `(surface_id, generation)` so a stale effect cannot
  animate a new page or a replaced image.
- `effects = "auto" | "always" | "never"`; `auto` skips effects for operations
  that complete in < 100 ms (cache hits) to avoid flicker.
- Respect `NO_COLOR`, `--no-animations`, and terminal reduced-motion env vars.

### Effect inventory

| Trigger | Effect | Duration | Notes |
|---|---|---:|---|
| Startup logo | `sweep_in` + `coalesce` | 500 ms | Once per process |
| Page ready (network) | `sweep_in(LeftToRight)` + `fade_from` | 250 ms | Content area only; no effect on sidebar/footer |
| Page ready (cache) | short `fade_in` | 100 ms | `auto` may skip |
| Feed cards | staggered `fade_in` | 180 ms | 20 ms stagger, max 8 cards |
| Image ready | placeholder shimmer → frame/caption `coalesce` | 180 ms | Protocol pixels swap atomically; see below |
| Image ready (halfblocks) | true crossfade from blurred preview | 200 ms | Optional |
| Link peek | `fade_in`/slide | 100 ms | No effect if reduced motion |
| Media first frame | `fade_from` | 150 ms | Not per frame |
| Theme switch | palette crossfade | 180 ms | Optional |
| Modal/palette open | slide/fade | 90 ms | Keyboard response stays immediate |
| Error | no shake; static error panel | — | Motion must not delay reading |

### Why images are different

Tachyonfx applies to the ratatui cell buffer. Kitty/sixel/iTerm2 images are
transmitted as a separate protocol layer and are not cells. A tachyonfx
`dissolve` will not fade the image pixels. Therefore:

- default: animate the shimmering placeholder, border, caption, and the cells
  around the image, then swap the protocol image at the end;
- halfblock mode: render a low-res preview as cells and crossfade to the full
  image;
- never animate every video frame; use only the entry transition.

## Built-in themes

All 16 themes are in `themes.toml`, copied from
`prjctimg/gtm/gtm/src/theme.rs` (GPL-3.0):

1. Classic
2. Chadrula
3. One Dark
4. Tokyo Night
5. Tokyo Night Storm
6. Catppuccin Mocha
7. Catppuccin Latte (light)
8. Gruvbox Dark
9. Nord
10. Rose Pine
11. Everforest
12. Kanagawa
13. Kanagawa Lotus (light)
14. Solarized Light (light)
15. Solarized Dark
16. Monochrome

`themes.toml` is also the canonical schema for user themes.

### Theme model

The gtm fields are retained because they are already contrast-tuned. gtw adds
semantic Wikipedia fields derived at load time:

```text
link             = accent
link_visited     = blend(accent, fg_dim, 0.45)
link_red         = error
link_external    = secondary_accent
code_bg          = picker_bg
code_border      = muted_border
quote_border     = muted_border
quote_fg         = fg_dim
table_header_bg  = picker_bg
table_border     = border
infobox_bg       = picker_bg
infobox_border   = border
mark_bg          = accent
mark_fg          = bg
diff_add         = success
diff_del         = error
diff_add_bg      = blend(success, bg, 0.82)
diff_del_bg      = blend(error, bg, 0.82)
selection_fg/bg  = provided
border_active    = provided
```

Syntax highlighting palette (derived, optionally overridden per theme):

```text
keyword      = accent
function     = secondary_accent
type         = tertiary_accent
string       = success
number       = warning
constant     = warning
comment      = fg_dim (italic)
variable     = fg_bright
operator     = fg
punctuation  = fg_dim
attribute    = tertiary_accent
error        = error
```

- Use `syntect` with a custom `Theme` built from these colors; do not ship a
  fixed syntax theme that clashes with the UI.
- If a derived pair has contrast < 4.5:1 against `code_bg`, brighten it using
  `readable_fg` (port from gtm) before use.
- Monochrome theme forces `monochromatic = true`: all accents derive from the
  single accent hue; syntax uses brightness tiers.

### Theme rules

- Every widget reads colors from `Theme`; no hard-coded `Color::*` except
  `Color::Reset` for terminal default and `Color::Indexed` fallbacks.
- Theme selection: `:theme`, Settings → Appearance, or `--theme`.
- Theme picker shows a live preview pane and marks light/dark.
- Auto mode follows the OS light/dark preference when detectable; default is
  the last explicit choice.
- User themes: `$XDG_CONFIG_HOME/gtw/themes/*.toml`, same schema, validated on
  load. Invalid color or missing required field shows a config error panel.
- Light themes must not use pure white backgrounds; use the warm paper tones
  from the source.
- Contrast tests assert all semantic foreground/background pairs are >= 4.5:1
  (large text >= 3:1), with documented exceptions for decorative borders.

### Licensing and attribution

- `gtm` is GPL-3.0. `gtw` is GPL-3.0.
- Keep the source attribution in the theme module header and in
  `themes.toml`.
- The wordmark sizing/gradient logic is also ported from gtm under the same
  license.

## Config example

```toml
[theme]
name = "Catppuccin Mocha"
auto = false

[effects]
mode = "auto"          # auto | always | never
page = true
images = true
logo = true
video_first_frame = true

[brand]
wordmark = "gtw"
```
