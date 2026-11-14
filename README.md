# drift-plus

A YouTube Music player for your terminal, with Vim keybinds. This is a fork of
[Drift](https://github.com/socatlolmeow/drift) by socatlolmeow (GPL-3.0-or-later) with
tighter YouTube Music integration.

## What drift-plus adds

- **Autoplay.** When the queue or list runs out, playback continues with songs similar to the last one, like YouTube Music's radio. On by default; `:autoplay off` or `autoplay = false` turns it off.
- **Playlists that update themselves.** `:link <playlist URL>` saves a YouTube / YouTube Music playlist and keeps it in step with YouTube: at startup, every 30 minutes, and on `:sync`. With `cookies_from_browser` set, every playlist in your YouTube Music library and Liked Music are linked for you. See [Synced playlists](#synced-playlists).
- **A cover art view.** `F` now cycles lyrics → cover art → cover art and lyrics → player.
- **Faster starts.** The stream of the next song is looked up while the current one plays, and the song under the cursor is looked up after the cursor rests on it, so the wait for yt-dlp mostly disappears. See [Faster playback](#faster-playback).

The program is called `drift-plus` and keeps its own settings (`~/.config/drift-plus/`), cache and session, so it installs next to the original `drift` without touching it. On its first run it starts from a copy of drift's `config.toml` and playlists if you have them.

---

## Table of contents

- [Features](#features)
- [Installation](#installation)
  - [Arch Linux (AUR)](#arch-linux-aur)
  - [Install script](#install-script)
  - [From source](#from-source)
  - [Nix](#nix)
  - [Installing mpv and yt-dlp manually](#installing-mpv-and-yt-dlp-manually)
- [Usage](#usage)
- [Keybinds](#keybinds)
- [Commands](#commands)
- [Lyrics and cover art](#lyrics-and-cover-art)
- [Synced playlists](#synced-playlists)
- [Faster playback](#faster-playback)
- [Configuration](#configuration)
  - [Streaming or downloading](#streaming-or-downloading)
  - [Discord rich presence](#discord-rich-presence)
  - [Custom keys](#custom-keys)
  - [Environment variables](#environment-variables)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- Vim-style keybinds: `j`/`k`, `gg`/`ge`, counts like `5j`, and `:` for the command line
- Synced lyrics in a side panel or fullscreen, with the current line filled in as it's sung, plus a cover art view with track details and progress
- Queue and playlists. Build a queue, save and reorder playlists, or import any YouTube / YouTube Music playlist by URL
- Listening history: Recently Played and Most Played tabs, with persistent play counts
- Discovery: `ga` for an artist's songs, `gs` for similar songs, `gb` for the album, and `:album` / `:playlist` to search for albums and playlists
- A song menu (`G`) to play a song, queue it, add it to a playlist, jump to the artist or see its info
- A command palette (`:search`, `:seek`, `:vol`, `:import` and more) with completion and history
- Your queue and position are restored the next time you start Drift
- Media keys and `playerctl` on Linux (MPRIS), plus optional Discord rich presence
- One TOML file for volume, seek step, shuffle/repeat defaults, accent color and custom keybinds

## Installation

Drift needs [mpv](https://mpv.io/) for playback and [yt-dlp](https://github.com/yt-dlp/yt-dlp) to get the audio streams. Run `drift-plus --check` at any time to see if both are installed.

### Arch Linux (AUR)

The AUR package is the original Drift, without the drift-plus additions.

```bash
yay -S driftmusic-git
```

Any AUR helper works. You can also build it by hand:

```bash
git clone https://aur.archlinux.org/driftmusic-git.git
cd driftmusic-git
makepkg -si
```

`mpv` and `yt-dlp` are pulled in automatically.

### Install script

Works on Arch, Debian/Ubuntu, Fedora, openSUSE, Void, Alpine and Gentoo. It installs the dependencies and builds Drift with cargo.

```bash
git clone https://github.com/koushikkkkkkkkkk/drift-plus.git
cd drift-plus
./install.sh
```

### From source

Install the dependencies and [Rust](https://rustup.rs/), then:

```bash
git clone https://github.com/koushikkkkkkkkkk/drift-plus.git
cd drift-plus
cargo install --path .
```

This puts `drift-plus` in `~/.cargo/bin/`, which is on your `$PATH` if you installed Rust with rustup.

### Nix

The repo has a flake that puts `mpv` and `yt-dlp` on Drift's `PATH` for you:

```bash
nix run github:koushikkkkkkkkkk/drift-plus
```

### Installing mpv and yt-dlp manually

| Platform | Command |
| --- | --- |
| Debian / Ubuntu | `sudo apt install mpv` and `pipx install yt-dlp` |
| macOS | `brew install mpv yt-dlp` |
| Arch | `sudo pacman -S mpv yt-dlp` |
| Windows | `winget install mpv.net yt-dlp.yt-dlp` or `scoop install mpv yt-dlp` |

## Usage

```bash
drift-plus                    # start the player
drift-plus "daft punk"        # start with a search
drift-plus --check            # check that mpv and yt-dlp are installed
drift-plus --help
drift-plus --version
```

Press `/` to search, `Enter` to play, `a` to add a song to the queue and `?` for the full list of keybinds. Inside the player, `:help` lists the commands.

## Keybinds

These are the defaults. You can change them, see [Custom keys](#custom-keys).

| Key | Action |
| --- | --- |
| `j` `k` / `↓` `↑` | move (counts work: `5j`) |
| `gg` `ge` | jump to top / bottom (`5gg` goes to row 5) |
| `Tab` `Shift-Tab` | cycle Results / Queue / Playlists |
| `gr` `gq` `gp` | jump to Results / Queue / Playlists |
| `Enter` | play the selected song |
| `Space` | pause / resume |
| `n` `N` | next / previous track |
| `h` `l` / `←` `→` | seek back / forward (counts: `6l`) |
| `+` `-` | volume up / down |
| `s` `r` | toggle shuffle / cycle repeat |
| `a` `A` `P` | queue the song / queue all results / play all results |
| `dd` `x` | remove from queue or playlist |
| `J` `K` | move a queue or playlist item down / up |
| `G` | song menu |
| `ga` | go to the artist |
| `gs` | see similar songs |
| `gb` | go to the album |
| `gB` | save the album as a playlist |
| `D` | download the current song |
| `gd` | download the highlighted song |
| `gD` | download the whole album, playlist, queue or loaded results |
| `L` | toggle the lyrics panel |
| `F` | cycle fullscreen lyrics → cover art → cover art and lyrics → player |
| `/` `:` | search / command line |
| `?` | help |
| `q` | quit |

## Commands

Press `:` to open the command line. Most commands have short aliases (`:vol`, `:sa`, `:ar` and so on).

| Command | Action |
| --- | --- |
| `:search "query"` | search YouTube Music |
| `:play [N]` | play or resume, or play row `N` |
| `:pause` `:resume` `:toggle` `:stop` | control playback |
| `:next` `:prev` | next / previous track |
| `:seek 90` `1:30` `+10` | absolute or relative seek |
| `:vol 50` `+5` | set or change the volume |
| `:shuffle` `:repeat [off\|all\|one]` | playback modes |
| `:add [N]` `:addall` `:playall` | queue results |
| `:download [current\|selected\|all]` | download audio files (defaults to the current song; alias `:dl`) |
| `:remove [N]` `:clear` | remove from the queue / clear it |
| `:save name` `:load name` `:append name` | work with playlists |
| `:save! name` | save and overwrite an existing playlist |
| `:savealbum [name]` `:saveresults [name]` | save an album or the results directly as a playlist |
| `:addto name` `:delpl name` | add a song to / delete a playlist |
| `:import URL` | import a YouTube / YouTube Music playlist |
| `:link URL [name]` | save a YouTube / YouTube Music playlist and keep it in sync |
| `:unlink name` | stop syncing a playlist (its songs stay) |
| `:sync` | refresh linked playlists and your library now |
| `:autoplay [on\|off]` | continue with similar songs when everything has played |
| `:album [query]` | open the selected song's album, or search albums |
| `:playlist query` | search YouTube Music playlists |
| `:similar` | songs similar to the selected one (also `:radio`) |
| `:artist` | songs by the selected song's artist |
| `:results` `:queue` `:playlists` | switch view |
| `:lyrics` | toggle the lyrics panel |
| `:quit` | quit |

`:save` refuses to overwrite an existing playlist. Use `:save!` to replace it or `:append` to add to the end of it.

## Lyrics and cover art

Press `L` for a lyrics side panel (needs a terminal at least 70 columns wide). Press `F` to cycle through the fullscreen views:

1. **Lyrics**: the lyrics on their own.
2. **Cover art**: the artwork as large as the window allows, with the title, artist, album and a progress bar underneath.
3. **Cover art and lyrics**: the artwork and track details on the left, lyrics on the right.
4. Back to the player.

`Esc`, `q` or `L` closes any of them. The two cover views hide the bottom player bar to leave more room.

In the lyrics views the current line gets bold, white text on a dark neutral band, while nearby lines fade with distance. It fills in as it's sung. Songs without synced lyrics show plain text that you can scroll with `j`/`k`.

All fullscreen views keep playback controls available: `Space` to pause or resume, `h`/`l` or the arrow keys to seek, `n`/`N` to skip tracks, `+`/`-` for volume, `s` for shuffle and `r` to cycle repeat.

Lyrics come from [LRCLIB](https://lrclib.net) and are only fetched while a lyrics or cover view is open.

Artwork comes from YouTube thumbnails and is only fetched when a cover view is open. Drift trims black bars and crops and resizes the image to 1000 × 1000 pixels. Terminals detected as supporting the Kitty or iTerm2 graphics protocol show the image directly; other terminals use colored half blocks for a rough preview. The cover is hidden in very short windows to leave room for track details.

## Synced playlists

A linked playlist is a normal playlist in the Playlists tab (marked `↻`) whose songs are re-read from YouTube, so anything you add or remove in YouTube Music shows up here.

```
:link https://music.youtube.com/playlist?list=PL...            # named after the playlist
:link https://music.youtube.com/playlist?list=PL... Road trip   # or pick a name
:sync                                                           # refresh everything now
:unlink Road trip                                               # keep the songs, stop syncing
```

Linked playlists are refreshed when drift starts, every `sync_minutes` (30 by default) while it runs, and on `:sync`.

**Your own library.** Public and unlisted playlists need no account. To follow your private playlists and Liked Music, set `cookies_from_browser` under `[audio]` to a browser where you are signed in to YouTube Music (for example `"firefox"`). On the next sync drift links every playlist in your library by itself, and new playlists you create later appear after the following sync. Set `library = false` under `[youtube]` to link playlists only by hand.

Things to know:

- Syncing is one-way, from YouTube to drift. Changes made here to a linked playlist are replaced on the next sync; to change it for good, edit it in YouTube Music, or `:unlink` it first.
- A linked playlist never replaces a playlist you made in drift. On a name clash the linked one is called `Name (YouTube)`.
- Deleting or unlinking a library playlist here keeps the library sync from bringing it back. `:link` its URL to follow it again.
- If a refresh fails (offline, playlist made private), the songs already saved are kept.
- The queue is a copy: a sync does not change what is already queued.

## YouTube's bot check

YouTube sometimes refuses a song with "Sign in to confirm you're not a bot". drift-plus works around it the way [ytm-player](https://github.com/peternaame-boop/ytm-player) does, by trying several ways in, one after the other:

1. Without an account, with yt-dlp's default player clients. Songs are looked up this way even when cookies are configured, because signed-in lookups are the ones YouTube challenges most.
2. Without an account, with the `android` player client added (`player_client=default,android`).
3. With your cookies, if `cookies_from_browser` or `cookies_file` is set under `[audio]`.

The way that worked is tried first for the following songs, so a block costs time only once. If every way is refused, the error says what to do next.

Things you can change under `[audio]`:

- `cookies_from_browser = "firefox"` or `cookies_file = "~/cookies.txt"` adds the third step and lets drift-plus read your library.
- `stream_cookies = true` puts your account first (Premium quality, private or age-restricted songs). yt-dlp then needs its JavaScript challenge solver: install `deno`, and either `yt-dlp-ejs` or add `ytdlp_args = ["--remote-components", "ejs:github"]`.
- `player_client` sets the player clients of the first step.
- `ytdlp_args` adds any other yt-dlp arguments, for example a proxy.

An old yt-dlp is the most common cause of both this error and "HTTP error 403", so update it first (`yt-dlp -U`, or your package manager).

## Faster playback

Most of the wait before a song starts is yt-dlp working out the stream address. drift-plus does that ahead of time:

- While a song plays, the streams of the next two in the queue or list are looked up, so moving on is immediate. This is skipped with shuffle on, since the next song isn't known.
- When the cursor rests on a song for about a second, its stream is looked up too, and mpv is started in the background, so `Enter` plays it right away.
- Plain https audio is asked for before HLS, which mpv opens sooner, and the way past [YouTube's bot check](#youtubes-bot-check) that worked is used first, so no time goes into attempts that are known to fail.
- Looked-up streams are remembered for five hours. If one has expired by the time it's played, it's looked up again without counting as a failed song.

drift-plus plays through mpv with a stream address from yt-dlp, while [cliamp](https://github.com/bjarneo/cliamp) pipes yt-dlp straight into its own decoder; in both, yt-dlp's lookup is the slow part, which is why it is moved out of the way here. The first song after starting drift-plus, or a song you play before the lookup finishes, still takes as long as yt-dlp needs. Both lookups can be turned off with `prewarm` and `prewarm_selected` under `[youtube]`; the second one runs yt-dlp more often than plain playback would. For no waiting at all on a slow connection, `download = true` under `[audio]` caches songs on disk and fetches the next one in the background.

## Configuration

Settings live in `config.toml` inside Drift's config directory (`~/.config/drift-plus/` on Linux). Drift only reads the file and never rewrites it, so your comments are safe. Restart Drift after editing.

```toml
[playback]
volume = 80
shuffle = false
repeat = "off"        # off, all or one
autoplay = true       # keep playing similar songs when the queue or list runs out
restore_session = true
seek_step = 5.0
volume_step = 5.0

[artist]
popular_first = false

[ui]
accent = "cyan"       # a color name or a hex value like "#FF8800"

[discord]
client_id = ""        # set this to enable Discord rich presence

[audio]
download = false              # false: stream songs; true: download to the cache first
download_max_minutes = 15     # with download = true, longer songs are still streamed (0 = no limit)
download_dir = "~/Music/drift-plus"
cookies_from_browser = ""     # e.g. "firefox" or "chrome"; also what signs you in for library sync
cookies_file = ""             # or a cookies.txt (Netscape format)
stream_cookies = false        # true: look songs up with your account first
player_client = ""            # yt-dlp player clients for the first attempt
ytdlp_args = []               # extra yt-dlp arguments

[youtube]
sync = true                   # keep linked playlists up to date
sync_minutes = 30             # 0 = only at startup and on :sync
library = true                # with cookies_from_browser set, link your whole library
prewarm = true                # look up the next song's stream ahead of time
prewarm_selected = true       # also look up the song under the cursor

[keys]
quit = "Q"
move_down = ["j", "down", "ctrl+n"]
move_up = ["k", "up", "ctrl+p"]
go_top = ["gg", "home"]
shuffle = []                  # an empty list unbinds the action
```

If you already have a `config.toml` from an older version, add the `[audio]` and `[youtube]` sections by hand, since Drift never rewrites the file. Settings you leave out use the defaults shown above.

### Streaming or downloading

By default Drift streams songs, so playback starts right away, even for hour-long tracks. With `download = true` songs are cached on disk first, and the next song in the queue is fetched in the background while the current one plays.

### Saving songs permanently

Set `download_dir` under `[audio]` in `config.toml` (default: `~/Music/drift-plus`). The directory is created when needed. `~` expands to your home directory; relative paths resolve from the directory where you start Drift. Restart Drift after changing it.

- Press `D` to download the current song, including from fullscreen lyrics or cover art.
- Press `gd` to download the highlighted song without changing playback.
- Press `gD` to download all songs in the open album, playlist or queue. On the playlist list, it downloads the highlighted playlist. For search results and history, it downloads all currently loaded songs.
- In an album/playlist search picker, press `d` to download the entire collection. To download one song instead, press `Enter`, highlight the song, then press `gd`.
- The `G` context menu also offers `d` for one song and `D` for the whole list.
- Commands: `:download` (current song), `:download selected`, and `:download all`. `:dl` is an alias.

Downloads run in the background, one batch at a time, with progress and a completion summary. Duplicate songs within a batch are downloaded once, existing files are kept, and a failed song does not stop the rest. Keep Drift open until the batch finishes.

Files use the best available audio-only format in its original container, such as `.webm` or `.m4a`; no MP3 conversion or ffmpeg is required. Permanent downloads are separate from the playback cache: they work regardless of `audio.download` or `download_max_minutes`, use `cookies_from_browser` when configured, and are never automatically pruned.

### Discord rich presence

This is off by default.

1. Create an application at https://discord.com/developers/applications.
2. Copy the Application ID into `client_id` under `[discord]` in `config.toml`.
3. Restart Drift.

Your status then shows the current song with cover art, album, a progress bar and a "Listen Along" button. If Discord isn't running, Drift keeps retrying quietly in the background.

### Custom keys

To change keys, add a `[keys]` section to `config.toml`. Each line sets an action to one key or a list of keys and replaces that action's default keys. Everything you leave out keeps its default.

```toml
[keys]
quit = "Q"
move_down = ["j", "down", "ctrl+n"]
move_up = ["k", "up", "ctrl+p"]
go_top = ["gg", "home"]
shuffle = []                  # an empty list unbinds the action
```

A key is a character (`"x"`, `"G"`, `"/"`), a name (`space`, `enter`, `esc`, `tab`, `backtab`, `backspace`, `delete`, `insert`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`, `menu`, `f1` to `f12`), `ctrl+` plus a key, or a sequence like `"gg"` or `"ctrl+x g"`. Counts (`5j`) keep working on any binding. If you bind a key that is a default of another action, the other action loses it. A mistake in `[keys]` shows a message at startup and falls back to the default keys.

| Action | Default keys |
| --- | --- |
| `move_down` `move_up` | `j` `down` / `k` `up` |
| `half_page_down` `half_page_up` | `ctrl+d` / `ctrl+u` |
| `page_down` `page_up` | `ctrl+f` `pagedown` / `ctrl+b` `pageup` |
| `go_top` `go_bottom` | `gg` `home` / `ge` `end` |
| `next_view` `prev_view` | `tab` / `backtab` |
| `view_results` `view_queue` `view_playlists` | `gr` / `gq` / `gp` |
| `toggle_queue_sidebar` | `Q` |
| `activate` | `enter` |
| `pause` | `space` |
| `next` `prev` | `n` `>` / `N` `<` |
| `seek_back` `seek_forward` | `h` `left` / `l` `right` |
| `volume_up` `volume_down` | `+` `=` / `-` `_` |
| `add` `add_all` `play_all` | `a` / `A` / `P` |
| `delete` | `x` `delete` `dd` |
| `move_item_down` `move_item_up` | `J` / `K` |
| `shuffle` `repeat` | `s` / `r` |
| `menu` | `G` `menu` |
| `go_artist` `go_similar` `go_album` `save_album` | `ga` / `gs` / `gb` / `gB` |
| `download` `download_selected` `download_all` | `D` / `gd` / `gD` |
| `lyrics_panel` `lyrics` | `L` / `F` |
| `search` `command` | `/` / `:` |
| `help` | `?` |
| `quit` | `q` |
| `back` | `backspace` `esc` |

In the Playlists tab `seek_forward` opens the selected playlist, and `seek_back` or `back` leaves it. Only the main view is remappable. Popups, the command line and fullscreen lyrics keep their own keys, and the in-app help (`?`) still lists the defaults.

### Environment variables

| Variable | What it does |
| --- | --- |
| `DRIFT_PLUS_CONFIG_DIR` | where Drift keeps `config.toml` and your playlists |
| `DRIFT_PLUS_STATE_DIR` | where Drift keeps the saved session |
| `DRIFT_PLUS_MPV_ARGS` | extra arguments passed to mpv |

## Troubleshooting

**Drift says mpv or yt-dlp is missing.** Run `drift-plus --check` to see which one, and install it with the table in [Installing mpv and yt-dlp manually](#installing-mpv-and-yt-dlp-manually).

**Playback fails with "Sign in to confirm you're not a bot".** See [YouTube's bot check](#youtubes-bot-check). In short: update yt-dlp first, then set `cookies_from_browser` under `[audio]`.

**The lyrics panel doesn't show up.** It needs a terminal at least 70 columns wide. Make the window wider or use `F` for fullscreen lyrics.

**Cover art looks blocky or is missing.** Press `F` twice from the player to open the cover view, and a third time for the cover next to the lyrics. A blocky preview means Drift couldn't detect Kitty or iTerm2 graphics support in your terminal. If you see "Cover art unavailable", the YouTube thumbnail couldn't be fetched or decoded. In very short windows, make the terminal taller to show the cover.

**`drift-plus: command not found`.** Make sure `~/.cargo/bin` is in your `$PATH`.

**`gb` opens the wrong album.** Drift looks albums up by album and artist name, so it can guess wrong, and the song needs album info to begin with. `:album <query>` lets you search for it yourself.

You can run several Drift windows at once. Only the first one restores and saves the session, and playlists saved from different windows are merged instead of overwritten.

## Contributing

Bug reports, ideas and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before you start. To build and run the tests:

```bash
cargo build
cargo test
```

Recent changes are listed in [CHANGELOG.md](CHANGELOG.md).

## License

drift-plus is a modified version of [Drift](https://github.com/socatlolmeow/drift) by socatlolmeow. Like Drift, it is free software, released under the [GNU General Public License version 3 or later](https://www.gnu.org/licenses/gpl-3.0.html).
