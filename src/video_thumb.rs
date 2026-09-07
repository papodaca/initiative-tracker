//! XDG video thumbnails, same path Nautilus uses.
//!
//! Look up `$XDG_CACHE_HOME/thumbnails/{large,normal}/<md5(file URI)>.png`,
//! then run a `.thumbnailer` from the data dirs (`gst-video-thumbnailer` or
//! `totem-video-thumbnailer` on a typical GNOME install).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use directories::BaseDirs;
use gtk::gio;
use gtk::glib;

const THUMB_SIZE: i32 = 256;
const GENERATE_TIMEOUT: Duration = Duration::from_secs(15);

pub fn cached_thumbnail(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if !path.is_file() {
        return None;
    }
    let uri = file_uri(path)?;
    let mtime = fs::metadata(path).ok()?.modified().ok()?;
    for dir in cache_dirs()? {
        let png = dir.join(format!("{}.png", uri_hash(&uri)));
        let Ok(meta) = fs::metadata(&png) else {
            continue;
        };
        let Ok(thumb_mtime) = meta.modified() else {
            continue;
        };
        if thumb_mtime >= mtime {
            return Some(png);
        }
    }
    None
}

pub fn generate_thumbnail(path: &str) -> Option<PathBuf> {
    if let Some(existing) = cached_thumbnail(path) {
        return Some(existing);
    }
    let path = Path::new(path);
    if !path.is_file() {
        return None;
    }
    let uri = file_uri(path)?;
    let dest_dir = cache_dirs()?.into_iter().next()?;
    fs::create_dir_all(&dest_dir).ok()?;
    let dest = dest_dir.join(format!("{}.png", uri_hash(&uri)));
    let tmp = dest_dir.join(format!(
        "{}.{}.png.partial",
        uri_hash(&uri),
        std::process::id()
    ));
    let _ = fs::remove_file(&tmp);

    let mime = gio::content_type_guess(Some(path), None::<&[u8]>).0;
    let argv = thumbnailer_argv(&mime, &uri, path, &tmp, THUMB_SIZE)
        .or_else(|| fallback_argv(&uri, path, &tmp, THUMB_SIZE))?;
    if !run_thumbnailer(&argv) {
        let _ = fs::remove_file(&tmp);
        return None;
    }
    if !tmp.is_file() {
        return None;
    }
    fs::rename(&tmp, &dest).ok()?;
    Some(dest)
}

fn file_uri(path: &Path) -> Option<String> {
    let abs = path.canonicalize().ok()?;
    glib::filename_to_uri(abs, None).ok().map(|s| s.to_string())
}

fn uri_hash(uri: &str) -> String {
    glib::compute_checksum_for_string(glib::ChecksumType::Md5, uri)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn cache_dirs() -> Option<Vec<PathBuf>> {
    let cache = BaseDirs::new()?.cache_dir().join("thumbnails");
    Some(vec![cache.join("large"), cache.join("normal")])
}

fn thumbnailer_argv(
    mime: &str,
    uri: &str,
    path: &Path,
    output: &Path,
    size: i32,
) -> Option<Vec<String>> {
    let mut best: Option<(i32, Vec<String>)> = None;
    for dir in thumbnailer_dirs() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path_ent = entry.path();
            if path_ent.extension().and_then(|e| e.to_str()) != Some("thumbnailer") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path_ent) else {
                continue;
            };
            let Some(spec) = parse_thumbnailer(&text) else {
                continue;
            };
            if !spec.mimes.iter().any(|m| m == mime) {
                continue;
            }
            if let Some(try_exec) = &spec.try_exec {
                if !command_exists(try_exec) {
                    continue;
                }
            }
            let argv = expand_exec(&spec.exec, uri, path, output, size);
            if argv.is_empty() || !command_exists(&argv[0]) {
                continue;
            }
            let score = thumbnailer_score(&argv[0]);
            if best.as_ref().is_none_or(|(s, _)| score > *s) {
                best = Some((score, argv));
            }
        }
    }
    best.map(|(_, argv)| argv)
}

fn fallback_argv(uri: &str, path: &Path, output: &Path, size: i32) -> Option<Vec<String>> {
    if let Some(bin) = glib::find_program_in_path("gst-video-thumbnailer") {
        return Some(vec![
            bin.to_string_lossy().into_owned(),
            "--input-uri".into(),
            uri.to_string(),
            "--output".into(),
            output.to_string_lossy().into_owned(),
            "--size".into(),
            size.to_string(),
        ]);
    }
    if let Some(bin) = glib::find_program_in_path("totem-video-thumbnailer") {
        return Some(vec![
            bin.to_string_lossy().into_owned(),
            "-s".into(),
            size.to_string(),
            uri.to_string(),
            output.to_string_lossy().into_owned(),
        ]);
    }
    let _ = path;
    None
}

fn thumbnailer_score(program: &str) -> i32 {
    let name = Path::new(program)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(program);
    if name.contains("gst-video-thumbnailer") {
        2
    } else if name.contains("totem-video-thumbnailer") {
        1
    } else {
        0
    }
}

fn thumbnailer_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(base) = BaseDirs::new() {
        dirs.push(base.data_dir().join("thumbnailers"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg.split(':').filter(|s| !s.is_empty()) {
            dirs.push(PathBuf::from(dir).join("thumbnailers"));
        }
    }
    dirs.push(PathBuf::from("/usr/local/share/thumbnailers"));
    dirs.push(PathBuf::from("/usr/share/thumbnailers"));
    dirs
}

struct ThumbnailerSpec {
    mimes: Vec<String>,
    exec: String,
    try_exec: Option<String>,
}

fn parse_thumbnailer(text: &str) -> Option<ThumbnailerSpec> {
    let mut in_entry = false;
    let mut mimes = Vec::new();
    let mut exec = None;
    let mut try_exec = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_entry = line.eq_ignore_ascii_case("[Thumbnailer Entry]");
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "MimeType" => {
                mimes = value
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
            }
            "Exec" => exec = Some(value.trim().to_string()),
            "TryExec" => try_exec = Some(value.trim().to_string()),
            _ => {}
        }
    }
    Some(ThumbnailerSpec {
        mimes,
        exec: exec?,
        try_exec,
    })
}

fn expand_exec(exec: &str, uri: &str, input: &Path, output: &Path, size: i32) -> Vec<String> {
    exec.split_whitespace()
        .map(|token| substitute_token(token, uri, input, output, size))
        .collect()
}

fn substitute_token(token: &str, uri: &str, input: &Path, output: &Path, size: i32) -> String {
    let mut out = String::new();
    let mut chars = token.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('%') => out.push('%'),
            Some('s') => out.push_str(&size.to_string()),
            Some('u') => out.push_str(uri),
            Some('i') => out.push_str(&input.to_string_lossy()),
            Some('o') => out.push_str(&output.to_string_lossy()),
            Some(other) => {
                out.push('%');
                out.push(other);
            }
            None => out.push('%'),
        }
    }
    out
}

fn command_exists(program: &str) -> bool {
    let path = Path::new(program);
    if path.is_absolute() {
        path.is_file()
    } else {
        glib::find_program_in_path(program).is_some()
    }
}

fn run_thumbnailer(argv: &[String]) -> bool {
    let Some((prog, args)) = argv.split_first() else {
        return false;
    };
    let Ok(mut child) = Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if start.elapsed() >= GENERATE_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_hash_is_stable_md5_hex() {
        let hash = uri_hash("file:///tmp/clip.webm");
        assert_eq!(hash.len(), 32);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash, uri_hash("file:///tmp/clip.webm"));
        assert_ne!(hash, uri_hash("file:///tmp/other.webm"));
    }

    #[test]
    fn parse_gst_thumbnailer_exec() {
        let spec = parse_thumbnailer(
            "[Thumbnailer Entry]\n\
             TryExec=/usr/bin/gst-video-thumbnailer\n\
             Exec=/usr/bin/gst-video-thumbnailer --input-uri %u --output %o --size %s\n\
             MimeType=video/mp4;video/webm;\n",
        )
        .unwrap();
        assert!(spec.mimes.contains(&"video/mp4".into()));
        assert!(spec.mimes.contains(&"video/webm".into()));
        let argv = expand_exec(
            &spec.exec,
            "file:///tmp/a.mp4",
            Path::new("/tmp/a.mp4"),
            Path::new("/tmp/out.png"),
            256,
        );
        assert_eq!(
            argv,
            vec![
                "/usr/bin/gst-video-thumbnailer",
                "--input-uri",
                "file:///tmp/a.mp4",
                "--output",
                "/tmp/out.png",
                "--size",
                "256",
            ]
        );
    }

    #[test]
    fn totem_exec_substitutes_s_u_o() {
        let argv = expand_exec(
            "/usr/bin/totem-video-thumbnailer -s %s %u %o",
            "file:///tmp/a.webm",
            Path::new("/tmp/a.webm"),
            Path::new("/tmp/out.png"),
            128,
        );
        assert_eq!(
            argv,
            vec![
                "/usr/bin/totem-video-thumbnailer",
                "-s",
                "128",
                "file:///tmp/a.webm",
                "/tmp/out.png",
            ]
        );
    }

    #[test]
    fn thumbnailer_score_prefers_gst() {
        assert!(
            thumbnailer_score("/usr/bin/gst-video-thumbnailer")
                > thumbnailer_score("/usr/bin/totem-video-thumbnailer")
        );
    }

    #[test]
    fn generate_thumbnail_writes_xdg_png_when_tools_exist() {
        let has_thumb = glib::find_program_in_path("gst-video-thumbnailer").is_some()
            || glib::find_program_in_path("totem-video-thumbnailer").is_some();
        if !has_thumb || glib::find_program_in_path("gst-launch-1.0").is_none() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("it-vthumb-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let video = dir.join("clip.webm");
        let encoded = Command::new("gst-launch-1.0")
            .args([
                "-q",
                "-e",
                "videotestsrc",
                "num-buffers=8",
                "!",
                "video/x-raw,width=160,height=120,framerate=10/1",
                "!",
                "videoconvert",
                "!",
                "vp8enc",
                "deadline=1",
                "!",
                "webmmux",
                "!",
                "filesink",
                &format!("location={}", video.display()),
            ])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(encoded, "gst-launch-1.0 failed to write a test WebM");
        let png = generate_thumbnail(&video.to_string_lossy())
            .expect("gst/totem thumbnailer should write ~/.cache/thumbnails/large/<md5>.png");
        assert!(png.is_file(), "{}", png.display());
        let bytes = fs::read(&png).unwrap();
        assert!(
            bytes.starts_with(b"\x89PNG"),
            "thumbnailer output is not a PNG"
        );
        let _ = fs::remove_file(&png);
        let _ = fs::remove_dir_all(&dir);
    }
}
