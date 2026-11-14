/* Playlists that follow a YouTube / YouTube Music playlist.
 *
 * A link ties a local playlist name to a YouTube playlist id. Syncing
 * re-reads every linked playlist and replaces the local copy, so songs added
 * or removed on YouTube Music show up here on their own. With browser cookies
 * configured, the playlists in the account's library are linked
 * automatically. */

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::Track;
use crate::store::{write_atomic, Store};
use crate::ytmusic;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Links {
    /* local playlist name -> YouTube playlist id */
    #[serde(default)]
    pub links: BTreeMap<String, String>,
    /* library playlists the user deleted here; never linked again by themselves */
    #[serde(default)]
    pub ignored: BTreeSet<String>,
    /* the browser :sync found (or was given) to read the account from */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
}

impl Links {
    fn path() -> Result<PathBuf> {
        Ok(Store::dir()?.join("synced.json"))
    }

    pub fn load() -> Links {
        Self::path()
            .ok()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        write_atomic(&Self::path()?, &serde_json::to_string_pretty(self)?)
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    pub fn has_id(&self, id: &str) -> bool {
        self.links.values().any(|linked| linked == id)
    }

    /// A playlist name nothing else uses yet, starting from `wanted`.
    pub fn free_name(&self, wanted: &str, taken: &BTreeSet<String>) -> String {
        let wanted = match wanted.trim() {
            "" => "YouTube playlist",
            name => name,
        };
        let used = |name: &str| self.links.contains_key(name) || taken.contains(name);
        if !used(wanted) {
            return wanted.to_string();
        }
        let first = format!("{wanted} (YouTube)");
        if !used(first.as_str()) {
            return first;
        }
        (2..)
            .map(|n| format!("{wanted} (YouTube {n})"))
            .find(|name| !used(name.as_str()))
            .expect("an unused name always exists")
    }

    /// Link the library playlists that aren't linked yet. `taken` holds the
    /// names of local playlists that must not be overwritten. Returns how many
    /// links were added.
    pub fn adopt(&mut self, found: Vec<(String, String)>, taken: &BTreeSet<String>) -> usize {
        let mut added = 0;
        for (id, title) in found {
            if self.ignored.contains(&id) || self.has_id(&id) {
                continue;
            }
            let name = self.free_name(&title, taken);
            self.links.insert(name, id);
            added += 1;
        }
        added
    }
}

pub struct Report {
    /* the links after library discovery */
    pub links: Links,
    pub added: usize,
    pub lists: Vec<(String, Result<Vec<Track>, String>)>,
    pub library_error: Option<String>,
    /* the browser a search for a signed-in one settled on */
    pub detected: Option<String>,
}

/// Where the account's playlists are read from.
pub enum Account {
    /* no account: only linked playlists are refreshed */
    None,
    /* a browser name or `file:<cookies.txt>` */
    Use(String),
    /* try these browsers until one turns out to be signed in */
    Detect(Vec<String>),
}

/* Browsers yt-dlp can read cookies from, most common first. */
const BROWSERS: [&str; 8] = [
    "firefox", "chrome", "chromium", "brave", "edge", "vivaldi", "opera", "whale",
];

/* Firefox-based browsers keep their profiles elsewhere; yt-dlp reads them
 * as `firefox:<directory>`. Paths are relative to the home directory. */
const FIREFOX_FORKS: [&str; 10] = [
    ".zen",
    ".librewolf",
    ".floorp",
    ".waterfox",
    ".var/app/app.zen_browser.zen/.zen",
    ".var/app/io.gitlab.librewolf-community/.librewolf",
    "Library/Application Support/zen/Profiles",
    "Library/Application Support/librewolf/Profiles",
    "AppData/Roaming/zen/Profiles",
    "AppData/Roaming/librewolf/Profiles",
];

/// Every browser worth asking, given the home directory.
pub fn browser_candidates(home: Option<&std::path::Path>) -> Vec<String> {
    let mut list: Vec<String> = BROWSERS.iter().map(|b| b.to_string()).collect();
    if cfg!(target_os = "macos") {
        list.push("safari".into());
    }
    if let Some(home) = home {
        for dir in FIREFOX_FORKS {
            let path = home.join(dir);
            if path.is_dir() {
                list.push(format!("firefox:{}", path.display()));
            }
        }
    }
    list
}

/// A short name for messages: `firefox:/home/me/.zen` -> `zen`.
pub fn browser_label(source: &str) -> String {
    if source.starts_with("file:") {
        return "cookies file".into();
    }
    match source.split_once(':') {
        Some((_, path)) => FIREFOX_FORKS
            .iter()
            .filter(|dir| path.ends_with(**dir))
            .filter_map(|dir| {
                dir.split('/')
                    .rev()
                    .find(|part| *part != "Profiles")
                    .map(|part| part.trim_start_matches('.').to_string())
            })
            .next()
            .unwrap_or_else(|| source.to_string()),
        None => source.to_string(),
    }
}

/// The first browser whose account shows playlists, with those playlists.
pub fn detect(
    candidates: &[String],
    mut ask: impl FnMut(&str) -> Result<Vec<(String, String)>, String>,
) -> Result<(String, Vec<(String, String)>), String> {
    for browser in candidates {
        match ask(browser) {
            Ok(found) if !found.is_empty() => return Ok((browser.clone(), found)),
            _ => {}
        }
    }
    Err("No browser signed in to YouTube Music was found. Open music.youtube.com in your browser, sign in, then run :sync again (or name it: :sync firefox)".into())
}

/// One sync pass. `library` is what the account's library holds, when it was
/// asked for; `fetch` loads the songs of one playlist id.
pub fn sync_with(
    mut links: Links,
    taken: &BTreeSet<String>,
    library: Option<Result<Vec<(String, String)>, String>>,
    mut fetch: impl FnMut(&str) -> Result<Vec<Track>, String>,
) -> Report {
    let (mut added, mut library_error) = (0, None);
    match library {
        Some(Ok(found)) => added = links.adopt(found, taken),
        Some(Err(error)) => library_error = Some(error),
        None => {}
    }
    let lists = links
        .links
        .iter()
        .map(|(name, id)| {
            let result = fetch(id).and_then(|tracks| match tracks.is_empty() {
                true => Err("YouTube returned no songs".to_string()),
                false => Ok(tracks),
            });
            (name.clone(), result)
        })
        .collect();
    Report {
        links,
        added,
        lists,
        library_error,
        detected: None,
    }
}

pub fn run(links: Links, taken: BTreeSet<String>, account: Account, library: bool) -> Report {
    let (cookies, found, detected) = match account {
        Account::None => (None, None, None),
        Account::Use(source) => {
            let found = library.then(|| ytmusic::library(&source));
            (Some(source), found, None)
        }
        Account::Detect(candidates) => match detect(&candidates, ytmusic::library_listing) {
            Ok((browser, found)) => (
                Some(browser.clone()),
                Some(Ok(ytmusic::with_liked_music(found))),
                Some(browser),
            ),
            Err(e) => (None, Some(Err(e)), None),
        },
    };
    let cookies = cookies.as_deref();
    let mut report = sync_with(links, &taken, found, |id| {
        ytmusic::playlist_tracks(id, cookies)
    });
    report.detected = detected;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(id: &str) -> Track {
        Track {
            id: id.into(),
            title: id.into(),
            artist: "Artist".into(),
            artists: Vec::new(),
            album: None,
            duration: None,
        }
    }

    fn pair(id: &str, title: &str) -> (String, String) {
        (id.to_string(), title.to_string())
    }

    #[test]
    fn adopts_new_library_playlists_without_clobbering() {
        let mut links = Links::default();
        links.links.insert("Gym".into(), "PLgym".into());
        links.ignored.insert("PLdeleted".into());
        let taken: BTreeSet<String> = ["Chill".to_string()].into();
        let added = links.adopt(
            vec![
                pair("PLgym", "Gym renamed online"),
                pair("PLdeleted", "Deleted here"),
                pair("PLchill", "Chill"),
                pair("PLgym2", "Gym"),
                pair("PLblank", "  "),
                pair("LM", "Liked Music"),
            ],
            &taken,
        );
        assert_eq!(added, 4);
        assert_eq!(links.links["Gym"], "PLgym");
        assert_eq!(links.links["Chill (YouTube)"], "PLchill");
        assert_eq!(links.links["Gym (YouTube)"], "PLgym2");
        assert_eq!(links.links["YouTube playlist"], "PLblank");
        assert_eq!(links.links["Liked Music"], "LM");
        assert!(!links.has_id("PLdeleted"));
        assert!(!links.links.contains_key("Chill"));

        let mut again = links.clone();
        assert_eq!(again.adopt(vec![pair("PLgym2", "Gym")], &taken), 0);
        assert_eq!(again, links);
        let mut taken = taken;
        taken.insert("Gym (YouTube 2)".into());
        assert_eq!(links.free_name("Gym", &taken), "Gym (YouTube 3)");
    }

    #[test]
    fn sync_reports_every_list_and_keeps_failures_apart() {
        let mut links = Links::default();
        links.links.insert("Good".into(), "PLgood".into());
        links.links.insert("Empty".into(), "PLempty".into());
        links.links.insert("Broken".into(), "PLbroken".into());
        let mut asked = Vec::new();
        let report = sync_with(
            links,
            &BTreeSet::new(),
            Some(Ok(vec![pair("PLnew", "New")])),
            |id| {
                asked.push(id.to_string());
                match id {
                    "PLempty" => Ok(Vec::new()),
                    "PLbroken" => Err("private".into()),
                    _ => Ok(vec![track("aaaaaaaaaaa"), track("bbbbbbbbbbb")]),
                }
            },
        );
        assert_eq!(report.added, 1);
        assert_eq!(asked.len(), 4);
        assert!(report.library_error.is_none());
        let get = |name: &str| &report.lists.iter().find(|(n, _)| n == name).unwrap().1;
        assert_eq!(get("Good").as_ref().unwrap().len(), 2);
        assert_eq!(get("New").as_ref().unwrap().len(), 2);
        assert!(get("Empty").is_err());
        assert_eq!(get("Broken").as_ref().unwrap_err(), "private");

        let failed = sync_with(
            report.links.clone(),
            &BTreeSet::new(),
            Some(Err("no cookies".into())),
            |_| Ok(vec![track("aaaaaaaaaaa")]),
        );
        assert_eq!(failed.library_error.as_deref(), Some("no cookies"));
        assert_eq!((failed.added, failed.lists.len()), (0, 4));
        assert_eq!(failed.links, report.links);
    }

    #[test]
    fn finds_the_signed_in_browser() {
        let candidates = browser_candidates(None);
        assert_eq!(candidates[0], "firefox");
        assert!(candidates.iter().any(|b| b == "chrome"));

        let mut asked = Vec::new();
        let hit = detect(&candidates, |browser| {
            asked.push(browser.to_string());
            match browser {
                "firefox" => Err("could not find firefox cookies database".into()),
                /* installed, but nobody is signed in */
                "chrome" => Ok(Vec::new()),
                "chromium" => Ok(vec![pair("PLmix", "Mix")]),
                _ => panic!("asked {browser} after a hit"),
            }
        });
        assert_eq!(hit, Ok(("chromium".to_string(), vec![pair("PLmix", "Mix")])));
        assert_eq!(asked, ["firefox", "chrome", "chromium"]);

        let miss = detect(&candidates, |_| Err("nope".into())).unwrap_err();
        assert!(miss.contains(":sync firefox"));

        /* Firefox-based browsers are offered only when they are installed */
        let home = std::env::temp_dir().join(format!("drift-plus-home-{}", std::process::id()));
        std::fs::create_dir_all(home.join(".zen")).unwrap();
        let with_zen = browser_candidates(Some(&home));
        let zen = with_zen.last().unwrap();
        assert_eq!(zen, &format!("firefox:{}", home.join(".zen").display()));
        assert_eq!(with_zen.len(), candidates.len() + 1);
        assert_eq!(browser_label(zen), "zen");
        assert_eq!(browser_label("brave"), "brave");
        assert_eq!(browser_label("firefox:work"), "firefox:work");
        assert_eq!(browser_label("file:/tmp/c.txt"), "cookies file");
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn links_file_roundtrip() {
        let mut links = Links::default();
        links.links.insert("Mix".into(), "PLmix".into());
        links.ignored.insert("PLgone".into());
        let text = serde_json::to_string(&links).unwrap();
        assert!(!text.contains("browser"));
        assert_eq!(serde_json::from_str::<Links>(&text).unwrap(), links);
        links.browser = Some("firefox".into());
        let text = serde_json::to_string(&links).unwrap();
        assert_eq!(serde_json::from_str::<Links>(&text).unwrap(), links);
        assert_eq!(
            serde_json::from_str::<Links>("{}").unwrap(),
            Links::default()
        );
        assert!(Links::default().is_empty() && !links.is_empty());
    }
}
