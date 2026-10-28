//! Native Shell's backward compatibility, kept apart from NSH's code: what
//! its releases froze, and the checks that hold a later checkout of NSH to
//! it. An app NSH delivered as a plugin may never be built again, so every
//! later release runs it.
//!
//! - `releases/v<version>/`: what a compatible release offered, written
//!   once when it shipped ([`record`]) and never changed: `runtime.txt`
//!   (every module's exports and the globals), `types/` (the declarations
//!   apps build against), `tailwind.txt` (every class and what it sets) and
//!   `apps/<app>/`, the apps it tested, as its compiler built them, with
//!   the script of what a user does and sees in each.
//! - `closed.txt`: what was closed for security, the only removals.
//!
//! The checkout under test is `../nsh` (`NSH_DIR` names another), its
//! crates this crate's dependencies. Its apps of the API being written
//! (`tests/compat/` in NSH: a case app per domain and some examples, each
//! with its script) are built with its `nsh` (`NSH_BIN`, by default the
//! checkout's `target/debug/nsh`) and run too.
//!
//! Until a release is recorded nothing is frozen: the tests of releases
//! have nothing to check.

mod apps;
mod compare;
mod coverage;
mod drawing;
mod logs;
mod surface;
mod tailwind;
mod values;

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

/// This repository.
fn compat_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The NSH checkout under test.
pub fn nsh_dir() -> PathBuf {
    std::env::var_os("NSH_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| compat_dir().join("../nsh"))
}

/// The `nsh` command line of the checkout, which builds its apps.
fn nsh_bin() -> PathBuf {
    if let Some(bin) = std::env::var_os("NSH_BIN") {
        return bin.into();
    }
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| nsh_dir().join("target"));
    target
        .join("debug")
        .join(format!("nsh{}", std::env::consts::EXE_SUFFIX))
}

/// Where builds and runs keep their files.
fn work_dir() -> PathBuf {
    let dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| compat_dir().join("target"))
        .join("nsh-compat");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A directory for temporary files: the system's (never `/tmp` itself,
/// which some sandboxes refuse).
fn temporary_root() -> PathBuf {
    let root = std::env::var_os("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    assert!(root.is_absolute());
    root
}

/// The recorded releases, oldest first, as (version, directory).
fn releases() -> Vec<(Vec<u32>, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(compat_dir().join("releases")) else {
        return Vec::new();
    };
    let mut out: Vec<_> = entries
        .filter_map(|e| {
            let path = e.ok()?.path();
            let name = path.file_name()?.to_str()?.strip_prefix('v')?.to_owned();
            let version = name
                .split('.')
                .map(str::parse)
                .collect::<Result<Vec<u32>, _>>()
                .ok()?;
            path.is_dir().then_some((version, path))
        })
        .collect();
    out.sort();
    out
}

/// The name a release is shown by.
fn release_name(dir: &Path) -> String {
    dir.file_name().unwrap().to_string_lossy().into_owned()
}

/// The files under `dir`, as (path relative to it, bytes), sorted.
fn read_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_string_lossy();
                out.push((rel.replace('\\', "/"), std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// A text file as the comparison sees it: a checkout on Windows may have
/// turned its line ends into CRLF.
fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap().replace("\r\n", "\n")
}

/// The entries of `closed.txt`: one per line, `<entry> -- <reason>`, where
/// an entry is what a break reported by `compare` starts with: `[<module>]
/// <path>` (a name of `runtime.txt`), `<file>: <Name>` or `<file>:
/// <Name>.<member>` (a declaration of `types/`), `tailwind <class>`, or
/// `<file>` (all of it). It closes the name and everything under it.
fn closed(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (entry, reason) = l
                .split_once(" -- ")
                .unwrap_or_else(|| panic!("closed.txt: `{l}` has no ` -- <reason>`"));
            assert!(!reason.trim().is_empty(), "closed.txt: `{l}` has no reason");
            entry.trim().to_owned()
        })
        .collect()
}

/// The entries of this repository's `closed.txt`.
fn closed_entries() -> Vec<String> {
    closed(
        &std::fs::read(compat_dir().join("closed.txt"))
            .map(text)
            .unwrap_or_default(),
    )
}

/// Whether `entry` closes the break `b`: the break is about the entry or a
/// member of it (`a` closes `a.b` and `a#.m`, not `ab`).
fn is_closed(entry: &str, b: &str) -> bool {
    b.strip_prefix(entry)
        .is_some_and(|rest| rest.starts_with([':', '.', '#', '/', ' ']))
}

/// What the release record `base` (its files) offered that `runtime` and
/// `types` (this checkout's surface) no longer do, less what `closed`
/// closes.
fn record_breaks(
    base: &BTreeMap<String, Vec<u8>>,
    runtime: &str,
    types: &[(String, String)],
    closed: &[String],
) -> Vec<String> {
    let open = |b: &String| !closed.iter().any(|e| is_closed(e, b));
    let mut breaks = Vec::new();
    if let Some(base_runtime) = base.get("runtime.txt") {
        let base_runtime = text(base_runtime.clone());
        for b in compare::runtime(&base_runtime, runtime)
            .iter()
            .filter(|b| open(b))
        {
            breaks.push(format!("runtime {b}"));
        }
    }
    let base_types: Vec<_> = base
        .iter()
        .filter_map(|(p, bytes)| Some((p.strip_prefix("types/")?.to_owned(), text(bytes.clone()))))
        .collect();
    for b in compare::types(&base_types, types)
        .iter()
        .filter(|b| open(b))
    {
        breaks.push(format!("types {b}"));
    }
    breaks
}

/// An app of the checkout the scripts check: a case app of a domain
/// (`tests/compat/cases/<domain>/`, its script beside its `App.tsx`) or an
/// example (`examples/<app>/`, its script `tests/compat/examples/<app>.txt`).
struct CheckoutApp {
    name: String,
    source: PathBuf,
    script: PathBuf,
}

/// The checkout's apps (`NSH_COMPAT_APP=<name>`: those whose name starts
/// so).
fn checkout_apps() -> Vec<CheckoutApp> {
    let nsh = nsh_dir();
    let tests = nsh.join("tests/compat");
    let mut out = Vec::new();
    let dirs = |dir: PathBuf| -> Vec<PathBuf> {
        let mut out: Vec<_> = std::fs::read_dir(dir)
            .map(|d| d.filter_map(|e| Some(e.ok()?.path())).collect())
            .unwrap_or_default();
        out.sort();
        out
    };
    for dir in dirs(tests.join("cases")) {
        if dir.join("script.txt").is_file() {
            out.push(CheckoutApp {
                name: release_name(&dir),
                script: dir.join("script.txt"),
                source: dir,
            });
        }
    }
    for script in dirs(tests.join("examples")) {
        let Some(name) = script
            .file_name()
            .and_then(|n| n.to_str()?.strip_suffix(".txt"))
        else {
            continue;
        };
        out.push(CheckoutApp {
            name: name.to_owned(),
            source: nsh.join("examples").join(name),
            script,
        });
    }
    assert!(
        !out.is_empty(),
        "no apps under {}: is NSH_DIR an NSH checkout?",
        tests.display()
    );
    out.retain(|a| std::env::var("NSH_COMPAT_APP").map_or(true, |only| a.name.starts_with(&only)));
    out
}

/// The files of a build an app runs with; the rest (`bundle.nshc`, source
/// maps) is this runtime's or the debugger's.
const BUILD_FILES: [&str; 3] = ["bundle.js", "capabilities.json", "app.json"];

/// Builds `app` with the checkout's `nsh build --release` into `out`, as an
/// app runs it: its bundle, the capabilities and id it declares, its assets
/// and its script.
fn build(app: &CheckoutApp, out: &Path) -> Result<(), String> {
    let bin = nsh_bin();
    if !bin.is_file() {
        return Err(format!(
            "{} is missing: build the checkout's command line (`cargo build -p nsh-cli` in {}), \
             or name it with NSH_BIN",
            bin.display(),
            nsh_dir().display()
        ));
    }
    let staging = out.with_extension("build");
    let _ = std::fs::remove_dir_all(&staging);
    let _ = std::fs::remove_dir_all(out);
    let built = Command::new(&bin)
        .args(["build", "--release", "-o"])
        .arg(&staging)
        .arg(&app.source)
        .output()
        .map_err(|e| format!("{}: {e}", bin.display()))?;
    if !built.status.success() {
        return Err(format!(
            "`nsh build` failed:\n{}{}",
            String::from_utf8_lossy(&built.stdout),
            String::from_utf8_lossy(&built.stderr)
        ));
    }
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    for file in BUILD_FILES {
        if staging.join(file).exists() {
            std::fs::copy(staging.join(file), out.join(file)).map_err(|e| e.to_string())?;
        }
    }
    if staging.join("assets").exists() {
        copy_dir(&staging.join("assets"), &out.join("assets"))?;
    }
    std::fs::copy(&app.script, out.join("script.txt")).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&staging);
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    for (rel, bytes) in read_files(from) {
        let path = to.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Builds the checkout's apps into `root/<app>/`, side by side; the
/// directories built, or what failed.
fn build_checkout_apps(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let apps = checkout_apps();
    let results: Vec<_> = std::thread::scope(|s| {
        let runs: Vec<_> = apps
            .iter()
            .map(|app| {
                s.spawn(move || {
                    let dir = root.join(&app.name);
                    build(app, &dir)
                        .map(|()| (app.name.clone(), dir))
                        .map_err(|e| format!("{}: {e}", app.name))
                })
            })
            .collect();
        runs.into_iter().map(|r| r.join().unwrap()).collect()
    });
    let (built, failed): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
    if !failed.is_empty() {
        return Err(failed
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>()
            .join("\n\n"));
    }
    Ok(built.into_iter().map(Result::unwrap).collect())
}

/// Runs the apps in `dirs` (each a build with its script); the failures.
fn run_apps(dirs: &[(String, PathBuf)]) -> Vec<String> {
    dirs.iter()
        .filter_map(|(name, dir)| apps::run_app(dir).err().map(|e| format!("{name}: {e}")))
        .collect()
}

/// Records the checkout as release `version` in `releases/v<version>/`:
/// its surface and its apps built by its compiler. It refuses an existing
/// record (records are never rewritten), apps whose scripts fail, `covers`
/// claims that do not hold, and parts of the API no script checks.
pub fn record(version: &str) -> Result<PathBuf, String> {
    let dir = compat_dir().join("releases").join(format!("v{version}"));
    if dir.exists() {
        return Err(format!(
            "{} exists: a record is never rewritten",
            dir.display()
        ));
    }
    let staging = work_dir().join(format!("record-v{version}"));
    let _ = std::fs::remove_dir_all(&staging);
    let built = build_checkout_apps(&staging.join("apps"))?;
    let failed = run_apps(&built);
    if !failed.is_empty() {
        return Err(failed.join("\n\n"));
    }
    let runtime = surface::runtime();
    let types = surface::types();
    let previous = releases()
        .last()
        .and_then(|(_, d)| std::fs::read(d.join("tailwind.txt")).ok())
        .map(text);
    let tailwind = tailwind::record(previous.as_deref());
    coverage::check(&runtime, &types, &staging.join("apps"))?;
    std::fs::write(staging.join("runtime.txt"), &runtime).map_err(|e| e.to_string())?;
    std::fs::write(staging.join("tailwind.txt"), &tailwind).map_err(|e| e.to_string())?;
    for (path, text) in &types {
        let path = staging.join("types").join(path);
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(path, text).map_err(|e| e.to_string())?;
    }
    copy_dir(&staging, &dir)?;
    Ok(dir)
}

/// The checkout's apps, built by its compiler, do what their scripts say,
/// and their `covers` claims hold.
pub fn checkout_apps_run() -> Result<(), String> {
    let root = work_dir().join("checkout");
    let built = build_checkout_apps(&root)?;
    let failed = run_apps(&built);
    if !failed.is_empty() {
        return Err(failed.join("\n\n"));
    }
    coverage::claims_hold(&surface::runtime(), &surface::types(), &root)
}

/// Everything a recorded release offered is still offered, by the rules of
/// `compare` (and the Tailwind record's), less `closed.txt`.
pub fn nothing_released_is_removed() -> Result<(), String> {
    let releases = releases();
    if releases.is_empty() {
        return Ok(());
    }
    let closed = closed_entries();
    let (runtime, types, tailwind) = (surface::runtime(), surface::types(), tailwind::record(None));
    let mut breaks = Vec::new();
    for (_, dir) in &releases {
        let base: BTreeMap<_, _> = read_files(dir)
            .into_iter()
            .filter(|(p, _)| !p.starts_with("apps/"))
            .collect();
        let name = release_name(dir);
        for b in record_breaks(&base, &runtime, &types, &closed) {
            breaks.push(format!("{name}: {b}"));
        }
        if let Some(base_tailwind) = base.get("tailwind.txt") {
            for b in tailwind::breaks(&text(base_tailwind.clone()), &tailwind, &closed) {
                breaks.push(format!("{name}: {b}"));
            }
        }
    }
    if breaks.is_empty() {
        return Ok(());
    }
    Err(format!(
        "what a release offered is no longer offered:\n{}\n\n\
         Keep it: a rename keeps the old name as a @deprecated alias of the new one, a type only \
         widens (NSH's docs/dev/compatibility.md).",
        breaks.join("\n")
    ))
}

/// The apps each recorded release built still start and work.
pub fn released_apps_run() -> Result<(), String> {
    let mut failed = Vec::new();
    for (_, dir) in releases() {
        let mut apps: Vec<_> = std::fs::read_dir(dir.join("apps"))
            .map(|d| d.filter_map(|e| Some(e.ok()?.path())).collect())
            .unwrap_or_default();
        apps.sort();
        let name = release_name(&dir);
        let apps: Vec<_> = apps
            .into_iter()
            .filter(|p| p.is_dir())
            .map(|p| (format!("{name}/{}", release_name(&p)), p))
            .collect();
        failed.extend(run_apps(&apps));
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(failed.join("\n\n"))
    }
}

/// Whether a release is recorded: until one is, nothing is frozen.
pub fn any_release() -> bool {
    !releases().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkout_apps_run() {
        super::checkout_apps_run().unwrap_or_else(|e| panic!("{e}"));
    }

    #[test]
    fn nothing_released_is_removed() {
        if !any_release() {
            eprintln!("note: no release is recorded, nothing is frozen");
        }
        super::nothing_released_is_removed().unwrap_or_else(|e| panic!("{e}"));
    }

    #[test]
    fn released_apps_run() {
        super::released_apps_run().unwrap_or_else(|e| panic!("{e}"));
    }

    /// The comparison of records reports what goes and what changes, and
    /// `closed.txt` exempts what it names.
    #[test]
    fn record_breaks_are_reported() {
        let file = |p: &str, t: &str| (p.to_owned(), t.as_bytes().to_vec());
        let runtime = "[nsh]\nView function\nText function\n";
        let types = "export declare function a(x: number): void;\n";
        let base: BTreeMap<_, _> = [
            file("runtime.txt", runtime),
            file("types/nsh/index.d.ts", types),
        ]
        .into();
        let now_types = [("nsh/index.d.ts".to_owned(), types.to_owned())];
        assert!(record_breaks(&base, runtime, &now_types, &[]).is_empty());

        let fewer = "[nsh]\nView function\n";
        assert_eq!(
            record_breaks(&base, fewer, &[], &[]),
            [
                "runtime [nsh] Text: removed",
                "types nsh/index.d.ts: removed"
            ]
        );
        let closed = closed("# closed\n[nsh] Text -- unsafe\nnsh/index.d.ts -- unsafe\n");
        assert!(record_breaks(&base, fewer, &[], &closed).is_empty());
        assert!(!is_closed("[nsh] Text", "[nsh] TextInput: removed"));
        assert!(is_closed("[nsh] Text", "[nsh] Text#.m: removed"));
    }

    /// The comparison understands every declaration the checkout makes (an
    /// unknown form panics) and names every member.
    #[test]
    fn declarations_are_understood() {
        for (path, text) in surface::types() {
            for (name, d) in compare::declarations(&text) {
                if matches!(d.kind, "interface" | "class") {
                    assert!(
                        d.members.keys().all(|k| !k.is_empty()),
                        "{path}: {name}: a member without a name: {:?}",
                        d.members
                    );
                }
            }
        }
    }

    /// `releases/` holds only records named by a version.
    #[test]
    fn releases_are_named_by_version() {
        let Ok(entries) = std::fs::read_dir(compat_dir().join("releases")) else {
            return;
        };
        let named = releases().len();
        let all = entries
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .count();
        assert_eq!(named, all, "releases/ holds a directory not named v<x.y.z>");
    }
}
