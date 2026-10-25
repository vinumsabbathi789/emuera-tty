//! Save file management for the options overlay and the command line:
//! list a game's saves, copy or move a slot, export / import single
//! files, and back up / restore the whole save folder.
//!
//! Saves are Emuera's own files (`saveNN.sav`, `global.sav`) in the game's
//! save folder, so everything here is plain file copying. Nothing is ever
//! deleted outright: a removed or overwritten save first goes to a backup
//! under `$XDG_DATA_HOME/emuera-tty/backups/<game>/`.
//!
//! A game kept in git may have worktrees (other branches checked out
//! beside it). The launcher can make them share the main checkout's saves
//! (`save_home`), so the same save loads on every branch.

use std::path::{Path, PathBuf};

/// One save file.
pub struct SaveFile {
    /// slot number; None for global.sav
    pub slot: Option<u32>,
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    /// the comment Emuera shows in its save menu
    pub comment: String,
}

impl SaveFile {
    /// "slot 03" / "global".
    pub fn label(&self) -> String {
        match self.slot {
            Some(n) => format!("slot {:02}", n),
            None => "global".into(),
        }
    }
}

/// The comment of an Emuera binary save, if it is one.
pub fn comment_of(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let Some(mut r) = crate::vm::erasav::Reader::open(&bytes) else {
        // Emuera's text format
        return crate::vm::textsav::looks_like(&bytes).then(|| crate::vm::textsav::header(&bytes).ok().map(|h| h.2)).flatten();
    };
    r.file_type().ok()?;
    r.raw_i64().ok()?;
    r.raw_i64().ok()?;
    r.string().ok()
}

/// The slot of a `saveNN.sav` file name.
fn slot_of(name: &str) -> Option<u32> {
    let n = name.to_ascii_lowercase();
    let d = n.strip_prefix("save")?.strip_suffix(".sav")?;
    (!d.is_empty() && d.chars().all(|c| c.is_ascii_digit())).then(|| d.parse().ok())?
}

/// The file name of slot `n`.
pub fn slot_name(n: u32) -> String {
    format!("save{:02}.sav", n)
}

/// The save files in `dir`: slots in order, then global.sav.
pub fn list(dir: &Path) -> Vec<SaveFile> {
    let mut v: Vec<SaveFile> = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return v };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let slot = slot_of(&name);
        if slot.is_none() && !name.eq_ignore_ascii_case("global.sav") {
            continue;
        }
        let meta = e.metadata().ok();
        let path = e.path();
        let comment = comment_of(&path).unwrap_or_else(|| "(not an Emuera save)".into());
        v.push(SaveFile { slot, size: meta.as_ref().map_or(0, |m| m.len()), modified: meta.and_then(|m| m.modified().ok()), comment, path });
    }
    v.sort_by_key(|s| s.slot.unwrap_or(u32::MAX));
    v
}

/// "2026-10-17 21:04" in local time, from a file time.
pub fn when(t: Option<std::time::SystemTime>) -> String {
    let Some(t) = t else { return String::new() };
    let unix = |t: std::time::SystemTime| t.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    // the local offset: local time of day now minus UTC time of day now
    let d = crate::sys::now();
    let utc_now = unix(std::time::SystemTime::now());
    let mut off = (d.h * 3600 + d.mi * 60 + d.s) - utc_now.rem_euclid(86400);
    if off > 14 * 3600 {
        off -= 86400;
    } else if off < -12 * 3600 {
        off += 86400;
    }
    let secs = unix(t) + off;
    let (y, m, day) = civil(secs.div_euclid(86400));
    let tod = secs.rem_euclid(86400);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, day, tod / 3600, tod / 60 % 60)
}

/// Days since 1970-01-01 to (year, month, day) (Howard Hinnant's algorithm).
pub(crate) fn civil(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}

/// A timestamp for file and folder names: 20261017-210455.
pub fn stamp() -> String {
    let d = crate::sys::now();
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}", d.y, d.mo, d.d, d.h, d.mi, d.s)
}

/// Where backups of a game go: `<data>/emuera-tty/backups/<game folder name>`.
pub fn backup_root(game: &Path) -> Option<PathBuf> {
    let name = game.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "game".into());
    Some(crate::sys::data_base()?.join("emuera-tty").join("backups").join(name))
}

/// Copy every save file of `dir` into a new folder under `root`; returns
/// the folder and how many files were copied.
pub fn backup_all(dir: &Path, root: &Path, tag: &str) -> Result<(PathBuf, usize), String> {
    let saves = list(dir);
    if saves.is_empty() {
        return Err(format!("no saves in {}", dir.display()));
    }
    let dest = root.join(if tag.is_empty() { stamp() } else { format!("{}-{}", stamp(), tag) });
    std::fs::create_dir_all(&dest).map_err(|e| format!("{}: {}", dest.display(), e))?;
    for s in &saves {
        let to = dest.join(s.path.file_name().unwrap());
        std::fs::copy(&s.path, &to).map_err(|e| format!("{}: {}", to.display(), e))?;
    }
    Ok((dest, saves.len()))
}

/// The backups under `root`, newest first: (folder, number of saves).
pub fn backups(root: &Path) -> Vec<(PathBuf, usize)> {
    let mut v: Vec<(PathBuf, usize)> = std::fs::read_dir(root)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .map(|p| {
                    let n = list(&p).len();
                    (p, n)
                })
                .filter(|(_, n)| *n > 0)
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v
}

/// Put the saves of backup folder `from` back into `dir`, after backing up
/// what is there now.
pub fn restore(from: &Path, dir: &Path, root: &Path) -> Result<usize, String> {
    if !list(dir).is_empty() {
        backup_all(dir, root, "before-restore")?;
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    let saves = list(from);
    for s in &saves {
        let to = dir.join(s.path.file_name().unwrap());
        std::fs::copy(&s.path, &to).map_err(|e| format!("{}: {}", to.display(), e))?;
    }
    Ok(saves.len())
}

/// Keep a copy of `path` (about to be overwritten or removed) under `root`.
fn keep(path: &Path, root: &Path, why: &str) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let dest = root.join(format!("{}-{}", stamp(), why));
    std::fs::create_dir_all(&dest).map_err(|e| format!("{}: {}", dest.display(), e))?;
    std::fs::copy(path, dest.join(path.file_name().unwrap())).map_err(|e| format!("{}: {}", dest.display(), e))?;
    Ok(())
}

/// Copy a save file into slot `to` of `dir` (the old one there is kept as
/// a backup); `remove` makes it a move.
pub fn copy_to_slot(src: &Path, dir: &Path, to: u32, root: &Path, remove: bool) -> Result<PathBuf, String> {
    let dest = dir.join(slot_name(to));
    if dest.canonicalize().ok() == src.canonicalize().ok() {
        return Err("that is the same slot".into());
    }
    keep(&dest, root, "overwritten")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    std::fs::copy(src, &dest).map_err(|e| format!("{}: {}", dest.display(), e))?;
    if remove {
        std::fs::remove_file(src).map_err(|e| format!("{}: {}", src.display(), e))?;
    }
    Ok(dest)
}

/// Remove a save (a copy goes to the backups first).
pub fn delete(path: &Path, root: &Path) -> Result<(), String> {
    keep(path, root, "deleted")?;
    std::fs::remove_file(path).map_err(|e| format!("{}: {}", path.display(), e))
}

/// Copy a save out to `dest` (a file, or a folder to put it in). Returns
/// the file written.
pub fn export(src: &Path, dest: &Path) -> Result<PathBuf, String> {
    let to = if dest.is_dir() { dest.join(src.file_name().unwrap()) } else { dest.to_path_buf() };
    if let Some(p) = to.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("{}: {}", p.display(), e))?;
    }
    std::fs::copy(src, &to).map_err(|e| format!("{}: {}", to.display(), e))?;
    Ok(to)
}

/// Where exports go by default: `~/emuera-saves/<game>-saveNN-<time>.sav`.
pub fn default_export(game: &Path, s: &SaveFile) -> PathBuf {
    let g = game.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "game".into());
    let base = crate::sys::home_dir().unwrap_or_else(|| PathBuf::from(".")).join("emuera-saves");
    let what = match s.slot {
        Some(n) => format!("save{:02}", n),
        None => "global".into(),
    };
    base.join(format!("{}-{}-{}.sav", g, what, stamp()))
}

/// Bring a save file from elsewhere into slot `to` (or global.sav when the
/// file is one). It must be an Emuera binary save.
pub fn import(src: &Path, dir: &Path, to: Option<u32>, root: &Path) -> Result<PathBuf, String> {
    if comment_of(src).is_none() {
        return Err(format!("{} is not an Emuera save", src.display()));
    }
    match to {
        Some(n) => copy_to_slot(src, dir, n, root, false),
        None => {
            let dest = dir.join("global.sav");
            keep(&dest, root, "overwritten")?;
            std::fs::copy(src, &dest).map_err(|e| format!("{}: {}", dest.display(), e))?;
            Ok(dest)
        }
    }
}

/// The save folder of `game`, as emuera.config's セーブデータをsavフォルダ内に作成する
/// says (default: `sav/`).
pub fn save_dir_of(game: &Path) -> PathBuf {
    let sav = crate::loader::find_dir_ci(game, "sav");
    match sav {
        Some(d) => d,
        // no sav/ yet: saves next to the game if there are some, else sav/
        None if game.read_dir().map(|mut r| r.any(|e| e.ok().is_some_and(|e| slot_of(&e.file_name().to_string_lossy()).is_some()))).unwrap_or(false) => {
            game.to_path_buf()
        }
        None => game.join("sav"),
    }
}

/// The main checkout of a git worktree: `dir/.git` is then a file
/// (`gitdir: <main>/.git/worktrees/<name>`). None for a main checkout or a
/// folder outside git. Read from the files, without running git.
pub fn worktree_main(dir: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(dir.join(".git")).ok()?;
    let gitdir = PathBuf::from(text.trim().strip_prefix("gitdir:")?.trim());
    let gitdir = if gitdir.is_absolute() { gitdir } else { dir.join(gitdir) };
    // `commondir` points at the repository's .git (usually "../..")
    let common = match std::fs::read_to_string(gitdir.join("commondir")) {
        Ok(c) => gitdir.join(c.trim()),
        Err(_) => gitdir.parent()?.parent()?.to_path_buf(),
    };
    let common = common.canonicalize().ok()?;
    (common.file_name()? == ".git").then(|| common.parent().map(Path::to_path_buf)).flatten()
}

/// A worktree of a checkout: its folder and what it has checked out (a
/// branch, or `detached @ <commit>`).
pub struct Worktree {
    pub dir: PathBuf,
    pub head: String,
}

/// The worktrees of the main checkout `main` that still exist (git keeps
/// entries for removed ones until `git worktree prune`), by folder name.
pub fn worktrees(main: &Path) -> Vec<Worktree> {
    let Ok(rd) = std::fs::read_dir(main.join(".git").join("worktrees")) else { return Vec::new() };
    let mut out: Vec<Worktree> = rd
        .flatten()
        .filter_map(|e| {
            let meta = e.path();
            // `gitdir`: the worktree's own .git file
            let dot_git = PathBuf::from(std::fs::read_to_string(meta.join("gitdir")).ok()?.trim());
            let dir = dot_git.parent()?.to_path_buf();
            if !dir.join(".git").is_file() {
                return None;
            }
            let head = std::fs::read_to_string(meta.join("HEAD")).unwrap_or_default();
            let head = match head.trim().strip_prefix("ref: refs/heads/") {
                Some(b) => b.to_string(),
                None => format!("detached @ {}", head.trim().chars().take(9).collect::<String>()),
            };
            Some(Worktree { dir, head })
        })
        .collect();
    out.sort_by(|a, b| a.dir.cmp(&b.dir));
    out
}

/// The git checkout `game` is in (its folder or one of the two above:
/// some games keep theirs in a subfolder such as `Data/`).
pub fn repo_root(game: &Path) -> Option<PathBuf> {
    game.ancestors().take(3).find(|d| d.join(".git").exists()).map(Path::to_path_buf)
}

/// Where `game` keeps its saves: its own folder, or the same folder in the
/// main checkout when `game` is in a worktree of a checkout listed in
/// `shared` (the launcher's "share saves"). `sav/`, `dat/` and the backups
/// all follow.
pub fn save_home(game: &Path, shared: &[PathBuf]) -> PathBuf {
    let Some(root) = repo_root(game) else { return game.to_path_buf() };
    match worktree_main(&root) {
        Some(main) if shared.iter().any(|s| s.canonicalize().ok().as_deref() == Some(main.as_path())) => {
            let sub = game.strip_prefix(&root).unwrap_or(Path::new(""));
            if sub.as_os_str().is_empty() {
                main
            } else {
                main.join(sub)
            }
        }
        _ => game.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(20743), (2026, 10, 17));
    }

    #[test]
    fn slots_and_copies() {
        assert_eq!(slot_of("save07.sav"), Some(7));
        assert_eq!(slot_of("SAVE123.SAV"), Some(123));
        assert_eq!(slot_of("save.sav"), None);
        assert_eq!(slot_of("global.sav"), None);
        let t = std::env::temp_dir().join(format!("emuera-savemgr-{}", std::process::id()));
        let (dir, root) = (t.join("sav"), t.join("bk"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("save01.sav"), b"one").unwrap();
        std::fs::write(dir.join("save02.sav"), b"two").unwrap();
        // overwriting slot 2 keeps the old one in a backup
        copy_to_slot(&dir.join("save01.sav"), &dir, 2, &root, false).unwrap();
        assert_eq!(std::fs::read(dir.join("save02.sav")).unwrap(), b"one");
        let kept: Vec<_> = std::fs::read_dir(&root).unwrap().flatten().collect();
        assert_eq!(std::fs::read(kept[0].path().join("save02.sav")).unwrap(), b"two");
        // backup and restore everything
        let (bk, n) = backup_all(&dir, &root, "").unwrap();
        assert_eq!(n, 2);
        delete(&dir.join("save01.sav"), &root).unwrap();
        assert!(!dir.join("save01.sav").exists());
        assert_eq!(restore(&bk, &dir, &root).unwrap(), 2);
        assert_eq!(std::fs::read(dir.join("save01.sav")).unwrap(), b"one");
        let _ = std::fs::remove_dir_all(&t);
    }

    /// A worktree is found from its .git file, listed from the main
    /// checkout, and shares the main checkout's saves only when asked
    #[test]
    fn worktrees_and_shared_saves() {
        let root = std::env::temp_dir().join(format!("emuera-tty-worktrees-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let main = root.join("game");
        let wt = root.join("game.worktrees").join("parsee");
        let gone = root.join("game.worktrees").join("removed");
        let meta = main.join(".git").join("worktrees").join("parsee");
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(meta.join("gitdir"), format!("{}\n", wt.join(".git").display())).unwrap();
        std::fs::write(meta.join("HEAD"), "ref: refs/heads/parsee-banquets\n").unwrap();
        std::fs::write(meta.join("commondir"), "../..\n").unwrap();
        std::fs::write(wt.join(".git"), format!("gitdir: {}\n", meta.display())).unwrap();
        // an entry git hasn't pruned yet, its folder deleted
        let meta2 = main.join(".git").join("worktrees").join("removed");
        std::fs::create_dir_all(&meta2).unwrap();
        std::fs::write(meta2.join("gitdir"), format!("{}\n", gone.join(".git").display())).unwrap();
        let main_c = main.canonicalize().unwrap();
        assert_eq!(worktree_main(&wt), Some(main_c.clone()));
        assert_eq!(worktree_main(&main), None);
        let list = worktrees(&main);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].dir, wt);
        assert_eq!(list[0].head, "parsee-banquets");
        assert_eq!(save_home(&wt, &[]), wt);
        assert_eq!(save_home(&wt, &[main.clone()]), main_c);
        assert_eq!(save_home(&main, &[main.clone()]), main);
        // a game in a subfolder of the checkout: the same subfolder of the main one
        std::fs::create_dir_all(wt.join("Data")).unwrap();
        assert_eq!(save_home(&wt.join("Data"), &[main.clone()]), main_c.join("Data"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
