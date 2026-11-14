# Changelog

Changes to drift. Loosely follows [Keep A Changelog](https://keepachangelog.com/en/1.0.0/).

### Added

- drift-plus: tighter YouTube Music integration
  - Autoplay: when the queue or list runs out, playback continues with similar songs (`:autoplay`, `autoplay` under `[playback]`, now on by default)
  - Synced playlists: `:link URL [name]`, `:unlink name` and `:sync` keep playlists up to date with YouTube Music; with `cookies_from_browser` set, the account's library and Liked Music are linked automatically
  - `F` now cycles lyrics → cover art → cover art and lyrics → player
  - The next song's stream, and the stream of the song under the cursor, are looked up ahead of time so playback starts sooner
  - New `[youtube]` config section: `sync`, `sync_minutes`, `library`, `prewarm`, `prewarm_selected`
  - The program, its config, cache and state directories and its environment variables are now named `drift-plus` / `DRIFT_PLUS_*`, so it installs next to drift; drift's config and playlists are copied on the first run
  - "Sign in to confirm you're not a bot": songs are looked up without an account first, then with the `android` player client, then with cookies; the working way is remembered. New `[audio]` settings `cookies_file`, `stream_cookies`, `player_client`, `ytdlp_args`
  - `:sync` now finds a signed-in browser by itself, links the whole library and remembers the browser; `:sync <browser>`, `:sync <playlist URL>`, `:login`, `:logout`
  - Streams are always the best audio YouTube offers; cover art uses the largest thumbnail
  - The next two songs are looked up ahead and stream addresses are kept for 5 hours

- Download support for saving songs permanently to disk
  - Press `D` to download current song, `gd` for highlighted, `gD` for all in collection
  - Commands: `:download`, `:download selected`, `:download all` (alias `:dl`)
  - Configure save location with `download_dir` under `[audio]` in config.toml (default: `~/Music/drift`)
  - Background downloads with progress tracking and completion summary
  - Deduplication within batch, fails don't stop remaining downloads
  - Files saved in original audio format without transcoding
  - Configurable key bindings: `download`, `download_selected`, `download_all`

- Enhanced fullscreen lyrics and cover art display
  - Current line now bold white text on dark neutral band
  - Extra breathing room around current line with surrounding lines fading
  - Consistent styling across both fullscreen views

- Playlist pagination support
  - Browse collections beyond 50-item limit (tested up to 400+ tracks)
  - Proper continuation token handling and repetition detection
 
# New Contributors
- [@pnlt](https://github.com/pnlt) in [#6](https://github.com/socatlolmeow/drift/pull/6).
 
