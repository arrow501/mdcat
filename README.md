# mdcat — arrow501 fork

> Fancy `cat` for Markdown. Forked from [swsnr/mdcat] (archived) — see [NOTICE](./NOTICE).

[swsnr/mdcat]: https://github.com/swsnr/mdcat

```
$ mdcat sample.md
```

## What this fork adds

- **Sixel image protocol** — inline images in any sixel-capable terminal (BlackBox, foot, mlterm, xterm, …)
- **Runtime capability detection** — probes kitty graphics protocol first, then sixel via DA1; works automatically with no `$TERM` or `$TERM_PROGRAM` setup
- **Nix flake** — `nix run github:arrow501/mdcat`

## Install

```bash
# Cargo
cargo install --git https://github.com/arrow501/mdcat

# Nix
nix run github:arrow501/mdcat

# Build from source
git clone https://github.com/arrow501/mdcat && cd mdcat && cargo build --release
```

---

## Features

`mdcat` works best with [iTerm2], [WezTerm], [kitty], and any sixel-capable terminal, with a good terminal font with italic characters.
Then it

* nicely renders all basic CommonMark syntax,
* highlights code blocks with [syntect],
* shows [links][osc8], and images inline — with the right protocol chosen automatically at runtime,
* adds jump marks for headings in [iTerm2] (jump forwards and backwards with <key>⇧⌘↓</key> and <key>⇧⌘↑</key>).

[CommonMark]: http://commonmark.org
[wezterm]: https://wezfurlong.org/wezterm/

| Terminal                   |  Basic syntax | Syntax highlighting | Images | Jump marks |
| :------------------------- | :-----------: | :-----------------: | :----: | :--------: |
| Basic ANSI¹                | ✓             | ✓                   |        |            |
| Windows 10 console         | ✓             | ✓                   |        |            |
| [Terminology]              | ✓             | ✓                   | ✓      |            |
| [iTerm2]                   | ✓             | ✓                   | ✓²     | ✓          |
| [kitty]                    | ✓             | ✓                   | ✓²     |            |
| [WezTerm]                  | ✓             | ✓                   | ✓²     |            |
| [VSCode]                   | ✓             | ✓                   | ✓²     |            |
| [Ghostty]                  | ✓             | ✓                   | ✓²     |            |
| Sixel terminals³           | ✓             | ✓                   | ✓      |            |

1) mdcat requires that the terminal supports strikethrough formatting and [inline links][osc8].
    This includes most modern terminal emulators, such as Windows Terminal, KDE Konsole, or anything based on VTE, GNOME's terminal emulation library.
    But mdcat likely won't work well on old terminals that lack these features (e.g. the Linux text console).
2) SVG images are rendered with [resvg], see [SVG support].
3) Any terminal that reports sixel support in its DA1 device attributes response — detected automatically at runtime.
    Examples: [BlackBox], foot, mlterm, xterm (compiled with sixel support).

Not supported:

* CommonMark extension for footnotes.
* Inline markup and text wrapping in table cells.

[syntect]: https://github.com/trishume/syntect
[osc8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
[Terminology]: http://terminolo.gy
[iterm2]: https://www.iterm2.com
[WezTerm]: https://wezfurlong.org/wezterm/
[kitty]: https://sw.kovidgoyal.net/kitty/
[resvg]: https://github.com/RazrFalcon/resvg
[SVG support]: https://github.com/RazrFalcon/resvg#svg-support
[VSCode]: https://code.visualstudio.com/
[Ghostty]: https://mitchellh.com/ghostty
[BlackBox]: https://gitlab.gnome.org/raggesilver/blackbox

## Sixel in action

This README is its own demo — run `mdcat README.md` and the image below renders inline via sixel:

![mdcat rendering markdown with syntax highlighting and inline images](./screenshots/side-by-side.png)

## Usage

Try `mdcat --help` or read the [mdcat(1)](./mdcat.1.adoc) manpage.

`mdcat` can be linked or copied to `mdless`; if invoked as `mdless` it automatically uses pagination.

## Building

Run `cargo build --release`.

Building requires `libcurl` and `openssl`.

## Packaging

When packaging `mdcat` you may wish to include the following additional artifacts:

- A symlink or hardlink from `mdless` to `mdcat` (see above).
- Shell completions for relevant shells, by invoking `mdcat --completions` after building, e.g.

  ```console
  $ mdcat --completions fish > /usr/share/fish/vendor_completions.d/mdcat.fish
  $ mdcat --completions bash > /usr/share/bash-completion/completions/mdcat
  $ mdcat --completions zsh > /usr/share/zsh/site-functions/_mdcat
  # Same for mdless if you include it
  $ mdless --completions fish > /usr/share/fish/vendor_completions.d/mdless.fish
  $ mdless --completions bash > /usr/share/bash-completion/completions/mdless
  $ mdless --completions zsh > /usr/share/zsh/site-functions/_mdless
  ```

- A build of the man page `mdcat.1.adoc`, using [AsciiDoctor]:

  ```console
  $ asciidoctor -b manpage -a reproducible -o /usr/share/man/man1/mdcat.1 mdcat.1.adoc
  $ gzip /usr/share/man/man1/mdcat.1
  # If you include a mdless as above, you may also want to support man mdless
  $ ln -s mdcat.1.gz /usr/share/man/man1/mdless.1.gz
  ```

[AsciiDoctor]: https://asciidoctor.org/

## Troubleshooting

`mdcat` can output extensive tracing information when asked to.
Run `mdcat` with `$MDCAT_LOG=trace` for complete tracing information, or with `$MDCAT_LOG=mdcat::render=trace` to trace only rendering.

## License

Copyright Sebastian Wiesner <sebastian@swsnr.de> *(original author, not affiliated with this fork or its use of AI)*
Copyright arrow.swiech@gmail.com *(fork modifications)*

All code is subject to the terms of the Mozilla Public License, v. 2.0, see [LICENSE](LICENSE).

Some files are subject to the terms of the Apache 2.0 license,
see <http://www.apache.org/licenses/LICENSE-2.0>

> This Source Code Form is "Incompatible With Secondary Licenses", as defined by the Mozilla Public License, v. 2.0.
