use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use ratatui::style::Color;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

use crate::keys::{Keymap, Keys};
use crate::model::{RepeatMode, Track};
use crate::session::Session;
use crate::store::{write_atomic, Store};

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    struct ColorVisitor;
    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "a color name (e.g. \"cyan\") or hex value (e.g. \"#FF8800\")"
            )
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Color, E> {
            Color::from_str(v).map_err(de::Error::custom)
        }
    }
    deserializer.deserialize_str(ColorVisitor)
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Playback {
    pub volume: u8,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub autoplay: bool,
    pub restore_session: bool,
    pub seek_step: f64,
    pub volume_step: f64,
}

impl Default for Playback {
    fn default() -> Self {
        Playback {
            volume: 80,
            shuffle: false,
            repeat: RepeatMode::Off,
            autoplay: true,
            restore_session: true,
            seek_step: 5.0,
            volume_step: 5.0,
        }
    }
}

impl Playback {
    fn sanitize(&mut self) {
        self.volume = self.volume.min(100);
        let ok = |v: f64| v.is_finite() && v > 0.0;
        if !ok(self.seek_step) {
            self.seek_step = 5.0;
        }
        if !ok(self.volume_step) {
            self.volume_step = 5.0;
        }
        self.volume_step = self.volume_step.min(100.0);
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct Artist {
    pub popular_first: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Discord {
    pub client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Audio {
    pub download: bool,
    pub download_max_minutes: u32,
    pub download_dir: String,
    pub cookies_from_browser: String,
    pub cookies_file: String,
    pub stream_cookies: bool,
    pub player_client: String,
    pub ytdlp_args: Vec<String>,
}

#[cfg(unix)]
impl Default for Audio {
    fn default() -> Self {
        Audio {
            download: false,
            download_max_minutes: 15,
            download_dir: "~/Music/drift-plus".into(),
            cookies_from_browser: String::new(),
            cookies_file: String::new(),
            stream_cookies: false,
            player_client: String::new(),
            ytdlp_args: Vec::new(),
        }
    }
}

impl Audio {
    pub fn download_directory(&self) -> Result<PathBuf, String> {
        let dir = self.download_dir.as_str();
        if dir.trim().is_empty() {
            return Err("audio.download_dir must not be empty".into());
        }
        let path = if dir == "~" {
            dirs::home_dir().ok_or("cannot determine home directory")?
        } else if let Some(rest) = dir.strip_prefix("~/") {
            dirs::home_dir()
                .ok_or("cannot determine home directory")?
                .join(rest.trim_start_matches('/'))
        } else if dir.starts_with('~') {
            return Err("audio.download_dir supports only ~ or ~/ for home expansion".into());
        } else {
            PathBuf::from(dir)
        };
        if path.is_absolute() {
            Ok(path)
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .map_err(|e| format!("cannot resolve audio.download_dir: {e}"))
        }
    }

    /// Browser name for yt-dlp's --cookies-from-browser, if one is set.
    pub fn cookies(&self) -> Option<&str> {
        let c = self.cookies_from_browser.trim();
        (!c.is_empty()).then_some(c)
    }

    /// A Netscape cookies.txt for yt-dlp's --cookies, if one is set.
    pub fn cookie_file(&self) -> Option<PathBuf> {
        let c = self.cookies_file.trim();
        if c.is_empty() {
            return None;
        }
        Some(match c.strip_prefix("~/").zip(dirs::home_dir()) {
            Some((rest, home)) => home.join(rest),
            None => PathBuf::from(c),
        })
    }

    /// Where account cookies come from: `file:<path>` or a browser name.
    pub fn cookie_source(&self) -> Option<String> {
        match self.cookie_file() {
            Some(path) => Some(format!("file:{}", path.display())),
            None => self.cookies().map(str::to_string),
        }
    }

    /// Whether a track should be downloaded to the cache before playing,
    /// as opposed to streamed. Unknown lengths are streamed when a limit is set.
    pub fn should_download(&self, duration: Option<u32>) -> bool {
        if !self.download {
            return false;
        }
        match (self.download_max_minutes, duration) {
            (0, _) => true,
            (max, Some(d)) => d <= max.saturating_mul(60),
            (_, None) => false,
        }
    }
}

/* Everything that talks to YouTube Music beyond plain playback. */
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct Youtube {
    pub sync: bool,
    pub sync_minutes: u32,
    pub library: bool,
    pub prewarm: bool,
    pub prewarm_selected: bool,
}

impl Default for Youtube {
    fn default() -> Self {
        Youtube {
            sync: true,
            sync_minutes: 30,
            library: true,
            prewarm: true,
            prewarm_selected: true,
        }
    }
}

impl Youtube {
    /* how often linked playlists are refreshed; None = only at startup and on :sync */
    pub fn sync_interval(&self) -> Option<std::time::Duration> {
        (self.sync && self.sync_minutes > 0)
            .then(|| std::time::Duration::from_secs(u64::from(self.sync_minutes.max(5)) * 60))
    }
}

fn default_accent() -> Color {
    Color::Cyan
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Ui {
    #[serde(deserialize_with = "deserialize_color")]
    pub accent: Color,
}

impl Default for Ui {
    fn default() -> Self {
        Ui {
            accent: default_accent(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub playback: Playback,
    pub artist: Artist,
    pub ui: Ui,
    pub discord: Discord,
    pub audio: Audio,
    pub youtube: Youtube,
    pub keys: HashMap<String, Keys>,
    #[serde(skip)]
    pub keymap: Keymap,
}

impl Config {
    fn path() -> Result<PathBuf> {
        Ok(Store::dir()?.join("config.toml"))
    }

    pub fn render(p: &Playback, a: &Artist, u: &Ui, d: &Discord, au: &Audio) -> String {
        let accent = match u.accent {
            Color::Reset => "reset",
            Color::Black => "black",
            Color::Red => "red",
            Color::Green => "green",
            Color::Yellow => "yellow",
            Color::Blue => "blue",
            Color::Magenta => "magenta",
            Color::Cyan => "cyan",
            Color::Gray => "gray",
            Color::DarkGray => "dark-gray",
            Color::LightRed => "light-red",
            Color::LightGreen => "light-green",
            Color::LightYellow => "light-yellow",
            Color::LightBlue => "light-blue",
            Color::LightMagenta => "light-magenta",
            Color::LightCyan => "light-cyan",
            Color::White => "white",
            _ => "cyan",
        };
        format!(
            "\
# drift-plus settings. drift-plus only reads this file and never rewrites it, so your
# comments are safe. Restart drift-plus after editing.

[playback]
volume = {volume}
shuffle = {shuffle}
repeat = \"{repeat}\"
# When the queue or list runs out, keep going with songs similar to the last one.
autoplay = {autoplay}
restore_session = {restore}
seek_step = {seek:?}
volume_step = {vstep:?}

[artist]
# :artist and ga list the artist's most popular songs first, then more of their songs below
popular_first = {popular}

[ui]
# Accent color used throughout the UI.
# Named colors: black, red, green, yellow, blue, magenta, cyan, gray, dark-gray,
#               light-red, light-green, light-yellow, light-blue, light-magenta,
#               light-cyan, white
# Or use a hex value: \"#FF8800\"
accent = \"{accent}\"

[discord]
# Show the current song in your Discord status. Create an application at
# https://discord.com/developers/applications (its name is shown after
# \"Listening to\") and paste its Application ID here. Empty means off.
client_id = \"{client_id}\"

[audio]
# false: stream songs straight from YouTube (starts playing right away).
# true:  download songs to the cache first, then play (and prefetch the next one).
download = {download}
# With download = true, songs longer than this many minutes are still streamed
# instead of downloaded. 0 means download everything, however long.
download_max_minutes = {dl_max}
download_dir = {download_dir}
# Your browser's cookies: used to read your YouTube Music library, and as a
# fallback when YouTube refuses to play something without an account
# (\"Sign in to confirm you're not a bot\", age-restricted songs, ...).
# Examples: \"firefox\", \"chrome\", \"chromium\", \"brave\", \"edge\",
# \"firefox:my-profile\". Empty means off. Close the browser if reading fails.
cookies_from_browser = \"{cookies}\"
# Or a cookies.txt file (Netscape format) exported from your browser. Used
# instead of cookies_from_browser when set.
cookies_file = {cookies_file}
# false: songs are looked up without your account first (this is what trips
#        YouTube's bot check least often) and the cookies are only a fallback.
# true:  look songs up with your account first (Premium quality, private songs).
stream_cookies = {stream_cookies}
# yt-dlp player client(s) for the first attempt, e.g. \"default,android\".
# Empty lets yt-dlp choose. A second attempt with \"default,android\" is made
# on its own when the first one is refused.
player_client = {player_client}
# Extra arguments for every yt-dlp call that looks up or downloads a song,
# e.g. [\"--remote-components\", \"ejs:github\"] or [\"--proxy\", \"socks5://127.0.0.1:9050\"].
ytdlp_args = {ytdlp_args}

[youtube]
# Keep linked playlists (:link <playlist URL>) up to date with YouTube Music.
sync = true
# Minutes between refreshes while drift-plus runs. 0 = only at startup and on :sync.
sync_minutes = 30
# With cookies_from_browser set, also link every playlist in your YouTube Music
# library (and Liked Music) automatically.
library = true
# Look up the stream of the next song while the current one plays, so the
# switch is instant.
prewarm = true
# Also look up the song under the cursor after resting on it for a moment, so
# Enter starts it right away.
prewarm_selected = true

[keys]
# Remap keys here. Each line sets an action to one key or a list of keys and
# replaces that action's default keys. Everything you leave out keeps its vim
# default. See the README for the list of actions.
# Write keys as a character (\"x\", \"G\"), a name (space, enter, esc, tab, backtab,
# backspace, delete, home, end, pageup, pagedown, up, down, left, right, f1-f12),
# ctrl+key (\"ctrl+d\") or a sequence (\"gg\", \"ctrl+x g\"). An empty list unbinds.
# quit = \"Q\"
# toggle_queue_sidebar = \"Q\"
# move_down = [\"j\", \"down\"]
# go_top = [\"gg\", \"home\"]
",
            volume = p.volume,
            shuffle = p.shuffle,
            repeat = p.repeat.label(),
            autoplay = p.autoplay,
            restore = p.restore_session,
            seek = p.seek_step,
            vstep = p.volume_step,
            popular = a.popular_first,
            accent = accent,
            client_id = d.client_id,
            download = au.download,
            dl_max = au.download_max_minutes,
            download_dir = toml::Value::String(au.download_dir.clone()),
            cookies = au.cookies_from_browser.replace(['\\', '"'], ""),
            cookies_file = toml::Value::String(au.cookies_file.clone()),
            stream_cookies = au.stream_cookies,
            player_client = toml::Value::String(au.player_client.clone()),
            ytdlp_args = toml::Value::Array(
                au.ytdlp_args
                    .iter()
                    .cloned()
                    .map(toml::Value::String)
                    .collect()
            ),
        )
    }

    fn write(text: &str) -> Result<()> {
        write_atomic(&Self::path()?, text)
    }

    pub fn load() -> (Config, Option<String>) {
        let Ok(path) = Self::path() else {
            return (Config::default(), None);
        };
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let _ = Self::write(&Self::render(
                    &Playback::default(),
                    &Artist::default(),
                    &Ui::default(),
                    &Discord::default(),
                    &Audio::default(),
                ));
                return (Config::default(), None);
            }
            Err(e) => {
                return (
                    Config::default(),
                    Some(format!("cannot read {}: {e}", path.display())),
                )
            }
        };
        match toml::from_str::<Config>(&text) {
            Ok(mut c) => {
                c.playback.sanitize();
                let warning = match Keymap::build(&c.keys) {
                    Ok(keymap) => {
                        c.keymap = keymap;
                        None
                    }
                    Err(e) => Some(format!("config.toml [keys]: {e}; using default keys")),
                };
                (c, warning)
            }
            Err(e) => {
                let bak = path.with_extension("toml.bak");
                let _ = fs::rename(&path, &bak);
                (
                    Config::default(),
                    Some(format!(
                        "config.toml was invalid ({}); moved to {}",
                        e.message(),
                        bak.display()
                    )),
                )
            }
        }
    }
}

pub fn migrate_legacy() {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Legacy {
        volume: Option<u8>,
        shuffle: Option<bool>,
        repeat: Option<RepeatMode>,
        autoplay: Option<bool>,
        current: Option<usize>,
        position: f64,
        queue: Vec<Track>,
    }

    let Ok(path) = Config::path() else { return };
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    let Ok(table) = text.parse::<toml::Table>() else {
        return;
    };
    if table.contains_key("playback")
        || !(table.contains_key("queue") || table.contains_key("position"))
    {
        return;
    }
    let Ok(old) = toml::from_str::<Legacy>(&text) else {
        return;
    };

    let d = Playback::default();
    let mut p = Playback {
        volume: old.volume.unwrap_or(d.volume),
        shuffle: old.shuffle.unwrap_or(d.shuffle),
        repeat: old.repeat.unwrap_or(d.repeat),
        autoplay: old.autoplay.unwrap_or(d.autoplay),
        ..d
    };
    p.sanitize();
    let session = Session {
        queue: old.queue,
        current: old.current,
        position: old.position,
    };
    if let Ok(s) = session.render() {
        if !Session::exists() {
            let _ = Session::write(&s);
        }
    }
    let _ = Config::write(&Config::render(
        &p,
        &Artist::default(),
        &Ui::default(),
        &Discord::default(),
        &Audio::default(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_file_parses_back() {
        let text = Config::render(
            &Playback::default(),
            &Artist::default(),
            &Ui::default(),
            &Discord::default(),
            &Audio::default(),
        );
        let c: Config = toml::from_str(&text).unwrap();
        let d = Playback::default();
        assert_eq!(c.playback.volume, d.volume);
        assert_eq!(c.playback.repeat, d.repeat);
        assert_eq!(c.playback.seek_step, 5.0);
        assert!(c.playback.restore_session);
        assert!(!c.artist.popular_first);
        assert!(c.playback.autoplay);
        assert_eq!(c.youtube, Youtube::default());
    }

    #[test]
    fn youtube_section() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.youtube, Youtube::default());
        assert_eq!(
            c.youtube.sync_interval(),
            Some(std::time::Duration::from_secs(30 * 60))
        );
        let c: Config = toml::from_str(
            "[youtube]\nsync_minutes = 1\nlibrary = false\nprewarm_selected = false",
        )
        .unwrap();
        assert!(c.youtube.sync && c.youtube.prewarm);
        assert!(!c.youtube.library && !c.youtube.prewarm_selected);
        // very short intervals are raised so YouTube isn't hammered
        assert_eq!(
            c.youtube.sync_interval(),
            Some(std::time::Duration::from_secs(5 * 60))
        );
        let c: Config = toml::from_str("[youtube]\nsync_minutes = 0").unwrap();
        assert_eq!(c.youtube.sync_interval(), None);
        let c: Config = toml::from_str("[youtube]\nsync = false").unwrap();
        assert_eq!(c.youtube.sync_interval(), None);
        let c: Config = toml::from_str("[playback]\nautoplay = false").unwrap();
        assert!(!c.playback.autoplay);
    }

    #[test]
    fn artist_section() {
        let c: Config = toml::from_str("[artist]\npopular_first = true").unwrap();
        assert!(c.artist.popular_first);
        let c: Config = toml::from_str("[playback]\nvolume = 10").unwrap();
        assert!(!c.artist.popular_first);
    }

    #[test]
    fn partial_and_empty_files() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.playback.volume, 80);
        let c: Config =
            toml::from_str("[playback]\nvolume = 10\nrepeat = \"one\"\nseek_step = 10").unwrap();
        assert_eq!(
            (c.playback.volume, c.playback.repeat),
            (10, RepeatMode::One)
        );
        assert_eq!(c.playback.seek_step, 10.0);
        assert!(toml::from_str::<Config>("[playback]\nrepeat = \"sometimes\"").is_err());
    }

    #[test]
    fn sanitize_bad_values() {
        let mut p = Playback {
            volume: 250,
            seek_step: -3.0,
            volume_step: 900.0,
            ..Playback::default()
        };
        p.sanitize();
        assert_eq!((p.volume, p.seek_step, p.volume_step), (100, 5.0, 100.0));
    }

    #[test]
    fn accent_color_parsing() {
        // Default is cyan
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.ui.accent, Color::Cyan);

        // Named colors
        let c: Config = toml::from_str("[ui]\naccent = \"red\"").unwrap();
        assert_eq!(c.ui.accent, Color::Red);
        let c: Config = toml::from_str("[ui]\naccent = \"green\"").unwrap();
        assert_eq!(c.ui.accent, Color::Green);
        let c: Config = toml::from_str("[ui]\naccent = \"magenta\"").unwrap();
        assert_eq!(c.ui.accent, Color::Magenta);

        // Hex value
        let c: Config = toml::from_str("[ui]\naccent = \"#FF8800\"").unwrap();
        assert_eq!(c.ui.accent, Color::Rgb(255, 136, 0));

        // Invalid color is an error
        assert!(toml::from_str::<Config>("[ui]\naccent = \"notacolor\"").is_err());

        // Roundtrip: render then parse
        let u = Ui {
            accent: Color::Blue,
        };
        let text = Config::render(
            &Playback::default(),
            &Artist::default(),
            &u,
            &Discord::default(),
            &Audio::default(),
        );
        let c: Config = toml::from_str(&text).unwrap();
        assert_eq!(c.ui.accent, Color::Blue);
    }

    #[test]
    fn keys_section() {
        let c: Config = toml::from_str("").unwrap();
        assert!(c.keys.is_empty());
        let c: Config =
            toml::from_str("[keys]\nquit = \"Q\"\nmove_down = [\"e\", \"down\"]").unwrap();
        assert!(Keymap::build(&c.keys).is_ok());
        assert_eq!(c.keys.len(), 2);

        let c: Config = toml::from_str("[keys]\ntoggle_queue_sidebar = \"ctrl+q\"").unwrap();
        assert!(Keymap::build(&c.keys).is_ok());
        assert!(c.keys.contains_key("toggle_queue_sidebar"));
    }

    #[test]
    fn audio_section() {
        let c: Config = toml::from_str("").unwrap();
        assert!(!c.audio.download);
        assert_eq!(c.audio.cookies(), None);
        let c: Config = toml::from_str(
            "[audio]\ndownload = true\ndownload_max_minutes = 10\ncookies_from_browser = \" firefox \"",
        )
        .unwrap();
        assert_eq!(c.audio.cookies(), Some("firefox"));
        assert!(c.audio.should_download(Some(600)));
        assert!(!c.audio.should_download(Some(601)));
        assert!(!c.audio.should_download(None));
        let mut a = c.audio.clone();
        a.download_max_minutes = 0;
        assert!(a.should_download(Some(10_000)));
        a.download = false;
        assert!(!a.should_download(Some(1)));
    }

    #[test]
    fn download_directory_paths() {
        let mut audio = Audio::default();
        assert_eq!(audio.download_dir, "~/Music/drift-plus");
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            audio.download_directory().unwrap(),
            home.join("Music/drift-plus")
        );
        audio.download_dir = "~".into();
        assert_eq!(audio.download_directory().unwrap(), home);
        audio.download_dir = "~//Music/drift-plus".into();
        assert_eq!(
            audio.download_directory().unwrap(),
            home.join("Music/drift-plus")
        );
        audio.download_dir = "music/downloads".into();
        assert_eq!(
            audio.download_directory().unwrap(),
            std::env::current_dir().unwrap().join("music/downloads")
        );
        let absolute = std::env::temp_dir().join("drift music");
        audio.download_dir = absolute.to_str().unwrap().into();
        assert_eq!(audio.download_directory().unwrap(), absolute);
        for invalid in ["", " \t\n", "~user", "~user/music"] {
            audio.download_dir = invalid.into();
            assert!(audio.download_directory().is_err(), "{invalid:?}");
        }
        let c: Config = toml::from_str("[audio]\ndownload = true").unwrap();
        assert_eq!(c.audio.download_dir, "~/Music/drift-plus");
    }

    #[test]
    fn download_directory_roundtrip() {
        for dir in [
            "~/Music/drift-plus",
            "relative music/日本語",
            "C:\\Users\\Music\\drift",
            "music/\"quoted\"/back\\slash\twith\nnewline\rand\0null",
            "'''triple single quotes''' and \"\"\"double quotes\"\"\"",
        ] {
            let audio = Audio {
                download_dir: dir.into(),
                ..Audio::default()
            };
            let text = Config::render(
                &Playback::default(),
                &Artist::default(),
                &Ui::default(),
                &Discord::default(),
                &audio,
            );
            let c: Config = toml::from_str(&text).unwrap();
            assert_eq!(c.audio.download_dir, dir);
        }
    }

    #[test]
    fn stream_settings_roundtrip() {
        let a = Audio {
            cookies_file: "/tmp/my \"cookies\".txt".into(),
            stream_cookies: true,
            player_client: "default,android".into(),
            ytdlp_args: vec!["--proxy".into(), "socks5://127.0.0.1:9050".into()],
            ..Audio::default()
        };
        let text = Config::render(
            &Playback::default(),
            &Artist::default(),
            &Ui::default(),
            &Discord::default(),
            &a,
        );
        let c: Config = toml::from_str(&text).unwrap();
        assert_eq!(c.audio.cookies_file, a.cookies_file);
        assert!(c.audio.stream_cookies);
        assert_eq!(c.audio.player_client, "default,android");
        assert_eq!(c.audio.ytdlp_args, a.ytdlp_args);
        assert_eq!(
            c.audio.cookie_source().as_deref(),
            Some("file:/tmp/my \"cookies\".txt")
        );

        let plain: Config = toml::from_str("").unwrap();
        assert!(!plain.audio.stream_cookies && plain.audio.ytdlp_args.is_empty());
        assert_eq!(plain.audio.cookie_source(), None);
    }

    #[test]
    fn audio_roundtrip() {
        let a = Audio {
            download: true,
            download_max_minutes: 30,
            cookies_from_browser: "chrome".into(),
            ..Audio::default()
        };
        let text = Config::render(
            &Playback::default(),
            &Artist::default(),
            &Ui::default(),
            &Discord::default(),
            &a,
        );
        let c: Config = toml::from_str(&text).unwrap();
        assert!(c.audio.download);
        assert_eq!(c.audio.download_max_minutes, 30);
        assert_eq!(c.audio.cookies(), Some("chrome"));
    }
}
