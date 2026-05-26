<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./assets/preview.png" />
  <img alt="kitlog" src="./assets/preview.png" width="100%" />
</picture>

<div align="center">
  <h1>kitlog</h1>
  <h3>A CLI log viewer that renders that scales the output based on severity.</h3>
  Exact appearance depends on your default font size and Kitty configuration.
</div>

---

## Overview

We all like headers when rendering markdown. This project explores whether visual hierarchy belongs and to viewing logs.

By using [Kitty’s OSC 66 text sizing protocol](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/), kitlog turns plain terminal output into something closer to a semantic canvas. It's all still text, but much harder to miss or ignore when things ultimately go wrong.

This makes logs much easier to scan…and usually much more annoying, because once errors or warnings start ranting and raving, they dominate your screen and your attention.

> [!NOTE]
> This only works with any terminal emulators that support the text sizing protocol, which at the moment (I think) is only kitty.

---

## Supported Log Levels

The parser detects log levels case-insensitively anywhere in a line:

| Level | Keyword | Scale |
| ----- | ------- | ----- |
| Error | `error` | 5     |
| Warn  | `warn`  | 4     |
| Info  | `info`  | 3     |
| Debug | `debug` | 2     |
| Trace | `trace` | 1     |

The higher the scaling the larger the text renders.

---

## How it Works

1. Reads logs line-by-line (streaming, no buffering the whole file)
2. Uses a compiled regex to detect log level keywords
3. Splits each matching line into:
   - **Header** (timestamp / prefix)
   - **Message body**

4. Emits Kitty `OSC 66` escape sequences to scale the output

Lines without a recognized log level pass through unchanged.

---

## Configuration

kitlog can be configured via a TOML config file. By default it looks in the standard config directory for your OS (e.g. `~/.config/kitlog/config.toml` on Linux). Use `-c, --config <PATH>` to specify a custom path.

### Default Config

```toml
[levels.error]
scaling = 5
color = "1;31"
keyword = "error"

[levels.warn]
scaling = 4
color = "1;33"
keyword = "warn"

[levels.info]
scaling = 3
color = "1;34"
keyword = "info"

[levels.debug]
scaling = 2
color = "1;32"
keyword = "debug"

[levels.trace]
scaling = 1
color = "1;30"
keyword = "trace"
```

Each level has:
- **`scaling`** — text size multiplier (1–5)
- **`color`** — ANSI color code (e.g. `"1;31"` for bold red)
- **`keyword`** — the case-insensitive keyword detected in log lines

---

## Usage

```bash
kitlog --help
A utility to parse and visually format logs

Usage: kitlog [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to a log file [default: -]

Options:
  -c, --config <CONFIG>  Optional Path to config file
  -h, --help             Print help
  -V, --version          Print version
```
