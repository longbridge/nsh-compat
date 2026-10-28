//! The apps a release records are the test suite of what it freezes: while
//! they pass, a later checkout runs what was built for it. So a release
//! records only an API its apps' scripts check: every export, global, host
//! prop, event and style key, usually in a small case of NSH's
//! `tests/compat/cases/<domain>/` ([`check`]). Between releases the claims
//! the checkout's scripts make must hold ([`claims_hold`]).
//!
//! The parts of the API, as scripts name them:
//!
//! - `[<module>] <name>`: an export of a module of `runtime.txt`, the
//!   functions of `nsh:rt` included, and `[globals] <name>`, a global NSH
//!   declares (`nsh/globals.d.ts`): the others are the language's
//!   (ECMAScript's library as QuickJS implements it, and `Intl`, which
//!   TypeScript's library declares), not NSH's API, and stay in
//!   `runtime.txt` only;
//! - `[props] <name>`, `[events] <name>`: the members of the host elements'
//!   props interfaces (`nsh/types.d.ts`, `*Props`), handlers (`on…`) being
//!   events; a platform's props are `ios.<name>`, `android.<name>`,
//!   `desktop.<name>` and `web.<name>`;
//! - `[props] <Component>.<name>`, `[events] <Component>.<name>`: a
//!   component's own props, the members of `<Component>Props` in the module
//!   that exports `<Component>` (`[props] List.header`, `[events]
//!   Dialog.onOpenChange`), less the host props' names, which it inherits
//!   or passes on as they are (`testID`, `style`, `animation`, ...) and
//!   which are items of their own; an event is an `on…` function;
//! - `[styles] <name>`: the keys of `LayoutStyle`, `ViewStyle`, `TextStyle`,
//!   `InputStyle` and `ImageStyle`.
//!
//! A script claims the parts it checks with `covers <item>` steps, and the
//! claims are verified ([`verify`]): the bundle uses the part (a
//! component's prop: the component and the name), and a prop, event or
//! style key renamed in the bundle makes a check of the script fail (any
//! step that checks: `expect`, `absent`, `tap`, `frame`, `origin`,
//! `prop`, `noprop`, `style`, `nostyle`, `drawing`, `nodrawing`,
//! `listens`, `log`), so the script sees what it does (the paint styles
//! `fill`, `stroke`, `strokeWidth` and `strokeDasharray` show only in what
//! a canvas draws: a `drawing` step); a prop a component requires (`NavStack`'s `state`)
//! may make the app fail instead. Exports and globals cannot be taken out of an app that still
//! runs, so only their use is verified; their claim is the script
//! author's word that a step checks their result.
//!
//! Use is read from the bundle, statically: the runtime has no place that
//! sees, by name, every `nsh:rt` call, prop and style key an app makes
//! (props and styles reach the core decoded). A bundle uses an export it
//! imports (bundles are tree-shaken, so what they import they call), a
//! global it names as a free identifier, and a prop, event or style key it
//! writes as a string literal (`prop(el, "testID", …)`) or an object key
//! (`{ flexDirection: "row" }`, spreads, component props, the getters the
//! compiler makes for reactive ones: `{ get header() { … } }`). The compiler
//! makes some exports in place: `View`, `Text`, ... are `element("view")`,
//! ..., `List` is `nsh:rt`'s `list`, `If` and `Each` its `cond` and `each`,
//! a host element's `children` its `append` and `insert`. A style reached
//! only through Tailwind classes is not counted.

use std::{
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
};

use super::{apps, compare, text};

/// The interfaces of the style keys.
const STYLES: &[&str] = &[
    "LayoutStyle",
    "ViewStyle",
    "TextStyle",
    "InputStyle",
    "ImageStyle",
];

/// The primitives the JSX compiler makes in place rather than import
/// (`jsx::PRIMITIVES`): their host element, or `None` for `List`, which is
/// `nsh:rt`'s `list`. The others it compiles to components and imports.
const PRIMITIVES: &[(&str, &str, Option<&str>)] = &[
    ("nsh", "View", Some("view")),
    ("nsh", "Text", Some("text")),
    ("nsh", "Scroll", Some("scroll")),
    ("nsh", "Input", Some("input")),
    ("nsh", "Overlay", Some("overlay")),
    ("nsh", "Image", Some("image")),
    ("nsh", "WebView", Some("webview")),
    ("nsh", "Video", Some("video")),
    ("nsh", "List", None),
    ("nsh/image", "Image", Some("image")),
    ("nsh/webview", "WebView", Some("webview")),
    ("nsh/video", "Video", Some("video")),
    ("nsh/list", "List", None),
];

/// The prefixes of a platform's props interfaces and the prop that holds
/// them.
const PLATFORMS: &[(&str, &str)] = &[
    ("IOS", "ios"),
    ("Android", "android"),
    ("Desktop", "desktop"),
    ("Web", "web"),
];

/// The parts of the API: `runtime.txt`'s exports and the globals NSH
/// declares, the host elements' and the components' props and events and
/// the style keys of `types`; and of them, the props a component requires.
fn items(runtime: &str, types: &[(String, String)]) -> (BTreeSet<String>, HashSet<String>) {
    let mut out = BTreeSet::new();
    let mut required = HashSet::new();
    // Every global the runtime adds to QuickJS' is declared there (nsh-js'
    // `declarations` tests hold it), so what is not declared is the
    // language's.
    let declared: HashSet<String> = types
        .iter()
        .find(|(p, _)| p == "nsh/globals.d.ts")
        .map(|(_, d)| compare::declarations(d))
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, d)| d.kind != "interface" && d.kind != "type")
        .map(|(name, _)| name.split(' ').next().unwrap().to_owned())
        .collect();
    let mut exports = HashSet::new();
    let mut section = "";
    for line in runtime.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name;
            continue;
        }
        let path = line.split_once(' ').map_or(line, |(p, _)| p);
        if !path.contains(['.', '#']) && (section != "globals" || declared.contains(path)) {
            out.insert(format!("[{section}] {path}"));
            exports.insert(format!("[{section}] {path}"));
        }
    }
    let host = &types
        .iter()
        .find(|(p, _)| p == "nsh/types.d.ts")
        .expect("nsh/types.d.ts")
        .1;
    // The names of the host props (no platform's), which components
    // declaring them pass on as they are.
    let mut host_props = HashSet::new();
    for (name, d) in compare::declarations(host) {
        if d.kind != "interface" {
            continue;
        }
        let members = d.members.keys().filter(|m| is_name(m));
        if STYLES.contains(&name.as_str()) {
            out.extend(members.map(|m| format!("[styles] {m}")));
        } else if name.ends_with("Props") {
            // An element's own props are not a platform's, though a name
            // may start like one (`WebViewHostProps` is the WebView's).
            let element = name
                .strip_suffix("HostProps")
                .or_else(|| name.strip_suffix("Props"))
                .is_some_and(|e| PRIMITIVES.iter().any(|(_, p, _)| *p == e));
            let prefix = PLATFORMS
                .iter()
                .find(|(p, _)| !element && name.starts_with(p))
                .map_or(String::new(), |(_, prop)| format!("{prop}."));
            for m in members {
                let section = if is_handler(m) { "events" } else { "props" };
                out.insert(format!("[{section}] {prefix}{m}"));
                if prefix.is_empty() && !OWN_NAMES.contains(&name.as_str()) {
                    host_props.insert(m.clone());
                }
            }
        }
    }
    // A component's own props: `<Component>Props` of the module that
    // exports `<Component>`, less the host props it inherits or declares
    // again (`testID`, `style`, `children`, ...).
    for (path, text) in types {
        let Some(module) = path.strip_suffix(".d.ts") else {
            continue;
        };
        let module = module.strip_suffix("/index").unwrap_or(module);
        for (name, d) in compare::declarations(text) {
            let Some(component) = name.strip_suffix("Props") else {
                continue;
            };
            if d.kind != "interface" || !exports.contains(&format!("[{module}] {component}")) {
                continue;
            }
            for (m, texts) in &d.members {
                if !is_name(m) || host_props.contains(m) {
                    continue;
                }
                // A handler is a function (`onEndReachedThreshold` is a
                // number).
                let handler = is_handler(m) && texts.iter().any(|t| t.contains("=>"));
                let section = if handler { "events" } else { "props" };
                let item = format!("[{section}] {component}.{m}");
                if texts.iter().all(|t| !t.starts_with(&format!("{m}?"))) {
                    required.insert(item.clone());
                }
                out.insert(item);
            }
        }
    }
    (out, required)
}

/// Host props interfaces no component inherits, whose names components
/// declare as their own (`Overlay`'s `offset`, `margin` and `onCancel`
/// are `KeyboardAvoidingView.offset`'s and `Dialog.onCancel`'s too).
const OWN_NAMES: &[&str] = &["OverlayHostProps"];

/// A member named by an identifier (not a call or index signature).
fn is_name(member: &str) -> bool {
    member
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `onPress`, not `one`.
fn is_handler(name: &str) -> bool {
    name.strip_prefix("on")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_uppercase()))
}

/// What a bundle uses, as its text shows it.
#[derive(Default, Debug)]
struct Usage {
    /// `[<module>] <export>` of what it imports.
    imports: HashSet<String>,
    /// Its string literals and object keys.
    keys: HashSet<String>,
    /// Its free identifiers (not members, not object keys).
    idents: HashSet<String>,
    /// The host elements it makes (`element("view")`).
    elements: HashSet<String>,
}

impl Usage {
    fn add(&mut self, code: &str) {
        let toks = lex(code);
        let at = |i: usize| toks.get(i).copied().unwrap_or(Tok::Other);
        let mut imported = Vec::new();
        for (i, tok) in toks.iter().enumerate() {
            let prev = if i == 0 { Tok::Other } else { at(i - 1) };
            // `...x` spreads `x`; `a.x` and `a?.x` are members.
            let member = prev == Tok::Punct(b'.') && (i < 2 || at(i - 2) != Tok::Punct(b'.'));
            let key = is_key(&toks, i);
            match *tok {
                Tok::Str(s) => {
                    self.keys.insert(s.to_owned());
                }
                Tok::Ident(s) if key => {
                    self.keys.insert(s.to_owned());
                }
                Tok::Ident("import") if !member => {
                    self.import(&toks[i + 1..], &mut imported);
                }
                Tok::Ident(s) if !member => {
                    self.idents.insert(s.to_owned());
                }
                _ => {}
            }
        }
        for (module, name, local) in imported {
            for w in toks.windows(3) {
                let [Tok::Ident(a), Tok::Punct(p), Tok::Ident(s) | Tok::Str(s)] = *w else {
                    continue;
                };
                if a != local {
                    continue;
                }
                match (module, name, p) {
                    // `import * as m from "nsh/x"` uses what `m.<name>` names.
                    (module, "*", b'.') => {
                        self.imports.insert(format!("[{module}] {s}"));
                    }
                    // The primitives compile to `element("view")`, ...
                    ("nsh:rt", "element", b'(') => {
                        self.elements.insert(s.to_owned());
                    }
                    _ => {}
                }
            }
        }
    }

    /// Reads the import whose tokens (after `import`) start `toks`, as
    /// (module, exported name, local name).
    fn import<'a>(&mut self, toks: &[Tok<'a>], out: &mut Vec<(&'a str, &'a str, &'a str)>) {
        let Some(from) = toks.iter().position(|t| *t == Tok::Ident("from")) else {
            return;
        };
        let Some(Tok::Str(module)) = toks.get(from + 1).copied() else {
            return;
        };
        let clause = &toks[..from];
        if let [Tok::Punct(b'*'), Tok::Ident("as"), Tok::Ident(alias)] = clause {
            out.push((module, "*", alias));
            return;
        }
        let mut braces = false;
        let mut i = 0;
        while i < clause.len() {
            match clause[i] {
                Tok::Punct(b'{') => braces = true,
                Tok::Ident(name) | Tok::Str(name) => {
                    let (name, mut local) = if braces {
                        (name, name)
                    } else {
                        ("default", name)
                    };
                    if let [Tok::Ident("as"), Tok::Ident(alias), ..] = clause[i + 1..] {
                        local = alias;
                        i += 2;
                    }
                    self.imports.insert(format!("[{module}] {name}"));
                    out.push((module, name, local));
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Whether the bundle renders the component `name`, of whichever
    /// module: it imports it, or the compiler made it in place.
    fn uses_component(&self, name: &str) -> bool {
        self.imports
            .iter()
            .any(|i| i.rsplit_once(' ').is_some_and(|(_, n)| n == name))
            || PRIMITIVES
                .iter()
                .filter(|p| p.1 == name)
                .any(|p| self.covers(&format!("[{}] {}", p.0, p.1)))
    }

    /// Whether some bundle read into `self` exercises `item`.
    fn covers(&self, item: &str) -> bool {
        let (section, name) = item.split_once(' ').expect("[<section>] <name>");
        let rt = |f: &str| self.imports.contains(&format!("[nsh:rt] {f}"));
        match section {
            "[globals]" => match name {
                "If" => rt("cond"),
                "Each" => rt("each"),
                _ => self.idents.contains(name),
            },
            "[props]" if name == "children" => {
                self.keys.contains(name) || rt("append") || rt("insert")
            }
            "[props]" | "[events]" => match name.split_once('.') {
                Some((component, prop))
                    if component.starts_with(|c: char| c.is_ascii_uppercase()) =>
                {
                    self.keys.contains(prop) && self.uses_component(component)
                }
                Some((platform, prop)) => self.keys.contains(platform) && self.keys.contains(prop),
                None => self.keys.contains(name),
            },
            "[styles]" => self.keys.contains(name),
            _ => {
                self.imports.contains(item)
                    || match PRIMITIVES
                        .iter()
                        .find(|p| item == format!("[{}] {}", p.0, p.1))
                    {
                        Some((_, _, Some(tag))) => self.elements.contains(*tag),
                        Some((_, _, None)) => rt("list"),
                        None => false,
                    }
            }
        }
    }
}

/// A token of a bundle, as much as [`Usage`] needs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tok<'a> {
    Ident(&'a str),
    /// A string literal or a template without substitutions, unescaped
    /// never (the names looked for have no escapes).
    Str(&'a str),
    Punct(u8),
    /// A number, a regular expression or a template with substitutions.
    Other,
}

/// Words after which a `/` starts a regular expression.
const BEFORE_REGEX: &[&str] = &[
    "return",
    "typeof",
    "instanceof",
    "in",
    "of",
    "new",
    "delete",
    "void",
    "throw",
    "case",
    "do",
    "else",
    "yield",
    "await",
];

/// Splits JavaScript into tokens: enough of a lexer to tell code from
/// strings, comments, templates and regular expressions.
fn lex(src: &str) -> Vec<Tok<'_>> {
    let b = src.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'$' || c >= 0x80;
    let mut out: Vec<Tok> = Vec::new();
    // Per open brace: whether it is a template's `${`.
    let mut braces: Vec<bool> = Vec::new();
    let mut i = 0;
    // Scans a template from `i` (inside it) to its end or a `${`; whether
    // it ended.
    let template = |i: &mut usize, braces: &mut Vec<bool>| -> bool {
        while *i < b.len() {
            match b[*i] {
                b'\\' => *i += 2,
                b'`' => {
                    *i += 1;
                    return true;
                }
                b'$' if b.get(*i + 1) == Some(&b'{') => {
                    *i += 2;
                    braces.push(true);
                    return false;
                }
                _ => *i += 1,
            }
        }
        true
    };
    while i < b.len() {
        let c = b[i];
        match c {
            _ if c.is_ascii_whitespace() => i += 1,
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i = src[i + 2..].find("*/").map_or(b.len(), |e| i + 2 + e + 2);
            }
            b'/' if regex_may_follow(out.last()) => {
                i += 1;
                let mut class = false;
                while i < b.len() {
                    match b[i] {
                        b'\\' => i += 1,
                        b'[' => class = true,
                        b']' => class = false,
                        b'/' if !class => break,
                        _ => {}
                    }
                    i += 1;
                }
                i += 1;
                while i < b.len() && ident(b[i]) {
                    i += 1;
                }
                out.push(Tok::Other);
            }
            b'"' | b'\'' => {
                let start = i + 1;
                i = start;
                while i < b.len() && b[i] != c {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
                out.push(Tok::Str(&src[start..i.min(b.len())]));
                i += 1;
            }
            b'`' => {
                let start = i + 1;
                i = start;
                if template(&mut i, &mut braces) {
                    out.push(Tok::Str(&src[start..i - 1]));
                } else {
                    out.push(Tok::Other);
                }
            }
            b'{' => {
                braces.push(false);
                out.push(Tok::Punct(c));
                i += 1;
            }
            b'}' => {
                i += 1;
                if braces.pop() == Some(true) {
                    template(&mut i, &mut braces);
                    out.push(Tok::Other);
                } else {
                    out.push(Tok::Punct(c));
                }
            }
            _ if c.is_ascii_digit()
                || (c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit)) =>
            {
                i += 1;
                while i < b.len()
                    && (ident(b[i])
                        || b[i] == b'.'
                        || (matches!(b[i], b'+' | b'-') && matches!(b[i - 1], b'e' | b'E')))
                {
                    i += 1;
                }
                out.push(Tok::Other);
            }
            _ if ident(c) => {
                let start = i;
                while i < b.len() && ident(b[i]) {
                    i += 1;
                }
                out.push(Tok::Ident(&src[start..i]));
            }
            _ => {
                out.push(Tok::Punct(c));
                i += 1;
            }
        }
    }
    out
}

/// Whether `toks[i]` is a property's name in an object literal:
/// `{ name: … }`, or a getter, `{ get name() { … } }`, as the JSX
/// compiler passes a component's reactive props.
fn is_key(toks: &[Tok], i: usize) -> bool {
    let at = |i: Option<usize>| i.and_then(|i| toks.get(i)).copied().unwrap_or(Tok::Other);
    let opens = |t: Tok| matches!(t, Tok::Punct(b'{' | b','));
    let (prev, next) = (at(i.checked_sub(1)), at(Some(i + 1)));
    (opens(prev) && next == Tok::Punct(b':'))
        || (prev == Tok::Ident("get") && opens(at(i.checked_sub(2))) && next == Tok::Punct(b'('))
}

/// Whether a `/` after `prev` starts a regular expression (not a division).
fn regex_may_follow(prev: Option<&Tok>) -> bool {
    match prev {
        None => true,
        Some(Tok::Punct(c)) => !matches!(c, b')' | b']' | b'}'),
        Some(Tok::Ident(word)) => BEFORE_REGEX.contains(word),
        Some(_) => false,
    }
}

/// A built app with its script.
struct Fixture {
    name: String,
    dir: PathBuf,
    code: String,
    script: String,
    usage: Usage,
}

/// The built apps in `root` (`<app>/bundle.js` and its script).
fn fixtures(root: &Path) -> Vec<Fixture> {
    let mut dirs: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join("bundle.js").is_file())
        .collect();
    dirs.sort();
    dirs.into_iter()
        .map(|dir| {
            let read = |f: &str| text(std::fs::read(dir.join(f)).unwrap_or_default());
            let code = read("bundle.js");
            let mut usage = Usage::default();
            usage.add(&code);
            Fixture {
                name: dir.file_name().unwrap().to_string_lossy().into_owned(),
                script: read("script.txt"),
                code,
                usage,
                dir,
            }
        })
        .collect()
}

/// The parts of the API a script claims to check: its `covers <item>`
/// steps.
fn claims(script: &str) -> Vec<&str> {
    script
        .lines()
        .filter_map(|l| l.trim().strip_prefix("covers "))
        .map(str::trim)
        .collect()
}

/// Whether the claim that `fixture`'s script checks `item` holds, or why
/// not. The bundle must use it; a prop, event or style key must also
/// matter to the script: with it renamed in the bundle (so the runtime no
/// longer knows it), a check of the script fails, with no error reported;
/// a prop a component requires may make the app fail instead, as an app
/// without it would. Exports and globals cannot be taken out without the
/// app failing to link or throwing, so for them the use is what is
/// verified.
fn verify(fixture: &Fixture, item: &str, required: bool) -> Result<(), String> {
    if !fixture.usage.covers(item) {
        return Err("its bundle does not use it".into());
    }
    let (section, name) = item.split_once(' ').unwrap();
    if !matches!(section, "[props]" | "[events]" | "[styles]") || name == "children" {
        return Ok(());
    }
    let name = name.rsplit('.').next().unwrap();
    let code = without(&fixture.code, name, section == "[styles]");
    if code == fixture.code {
        return Err("its bundle names it only where it cannot be taken out".into());
    }
    match apps::run(code, &fixture.script, &fixture.dir) {
        Ok(()) => Err("the script passes without it: no step checks what it does".into()),
        Err(f) if f.check.is_some() && !f.errors => Ok(()),
        // The component cannot work without it: the app's failing is the
        // check.
        Err(f) if required && f.errors => Ok(()),
        Err(f) => Err(format!(
            "without it the app fails, but not at a check of the script: {}",
            f.message
        )),
    }
}

/// `code` with the prop, event or style key `name` renamed where it is
/// passed: object keys, and for props (not `keys_only`) the string
/// arguments of calls (`prop(el, "testID", …)`).
fn without(code: &str, name: &str, keys_only: bool) -> String {
    let toks = lex(code);
    let at = |i: usize| toks.get(i).copied().unwrap_or(Tok::Other);
    let mut out = String::with_capacity(code.len());
    let mut copied = 0;
    for (i, tok) in toks.iter().enumerate() {
        let (Tok::Ident(s) | Tok::Str(s)) = *tok else {
            continue;
        };
        if s != name || i == 0 {
            continue;
        }
        let (prev, next) = (at(i - 1), at(i + 1));
        let key = is_key(&toks, i);
        let argument = matches!(tok, Tok::Str(_))
            && prev == Tok::Punct(b',')
            && matches!(next, Tok::Punct(b',' | b')'));
        if key || (argument && !keys_only) {
            let start = s.as_ptr() as usize - code.as_ptr() as usize;
            out.push_str(&code[copied..start]);
            out.push_str(name);
            out.push_str("__untested");
            copied = start + s.len();
        }
    }
    out.push_str(&code[copied..]);
    out
}

/// The parts of the API, the parts the `covers` claims of the scripts of
/// the apps in `root` check (verified), and the claims that do not hold.
fn tested(
    runtime: &str,
    types: &[(String, String)],
    root: &Path,
) -> (BTreeSet<String>, BTreeSet<String>, String) {
    let (items, required) = items(runtime, types);
    let fixtures = fixtures(root);
    let mut problems = String::new();
    let claimed: Vec<(&Fixture, &str)> = fixtures
        .iter()
        .flat_map(|f| claims(&f.script).into_iter().map(move |c| (f, c)))
        .collect();
    // Each verification runs an app; they run side by side.
    let (all, required) = (&items, &required);
    let verified: Vec<Result<(), String>> = std::thread::scope(|s| {
        let runs: Vec<_> = claimed
            .iter()
            .map(|&(f, item)| {
                s.spawn(move || {
                    if all.contains(item) {
                        verify(f, item, required.contains(item))
                    } else {
                        Err("not a part of the API".into())
                    }
                })
            })
            .collect();
        runs.into_iter().map(|r| r.join().unwrap()).collect()
    });
    let mut tested = BTreeSet::new();
    for ((f, item), result) in claimed.iter().zip(verified) {
        match result {
            Ok(()) => {
                tested.insert(item.to_string());
            }
            Err(e) => problems += &format!("{}: `covers {item}`: {e}\n", f.name),
        }
    }
    (items, tested, problems)
}

/// Every `covers` claim of the scripts of the apps in `root` holds.
pub(crate) fn claims_hold(
    runtime: &str,
    types: &[(String, String)],
    root: &Path,
) -> Result<(), String> {
    let (_, _, problems) = tested(runtime, types, root);
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// What a release records is tested: every part of the API is checked by
/// a script of the apps in `root` (its `covers` claims, verified).
pub(crate) fn check(runtime: &str, types: &[(String, String)], root: &Path) -> Result<(), String> {
    let (items, tested, mut problems) = tested(runtime, types, root);
    let uncovered: Vec<_> = items.difference(&tested).cloned().collect();
    if !uncovered.is_empty() {
        problems += &format!(
            "no script checks:\n  {}\n\
             What a release freezes is tested: use it in a case of NSH's \
             tests/compat/cases/<domain>/ and give the domain's script steps that check what it \
             does and a `covers <item>` step per part.\n",
            uncovered.join("\n  ")
        );
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// What a bundle uses is read from its code, not its strings' contents or
/// comments, and the parts of the API are named as the list names them.
#[test]
fn usage_is_read() {
    let mut usage = Usage::default();
    usage.add(
        "import{prop as a,cond as b,element as el}from\"nsh:rt\";el(`view`);import*as m from'nsh/device';\
         import{\"odd\" as c}from\"nsh/x\";\
         // fetch(\"onPress\")\n/* Blob */ let r=/[\"/]fetch/g,x=1/2/3;\
         a(e,`style`,{flexDirection:`row`,...setTimeout,[k]:1});m.Platform.OS;\
         x=`a${{width:URL}}b${`testID`}`;y={gap:1}.gap;z=q?flex:1;foo.atob(1);\
         import{list as li}from\"nsh:rt\";c(li,{get header(){return 1},get:2});",
    );
    let mut imports: Vec<_> = usage.imports.iter().map(String::as_str).collect();
    imports.sort_unstable();
    assert_eq!(
        imports,
        [
            "[nsh/device] Platform",
            "[nsh/x] odd",
            "[nsh:rt] cond",
            "[nsh:rt] element",
            "[nsh:rt] list",
            "[nsh:rt] prop"
        ]
    );
    for key in [
        "style",
        "flexDirection",
        "row",
        "width",
        "testID",
        "gap",
        "header",
        "get",
    ] {
        assert!(usage.keys.contains(key), "{key}");
    }
    for no in ["onPress", "flex", "OS"] {
        assert!(!usage.keys.contains(no), "{no}");
    }
    for global in ["setTimeout", "URL", "flex"] {
        assert!(usage.idents.contains(global), "{global}");
    }
    for no in ["fetch", "Blob", "gap", "atob", "width"] {
        assert!(!usage.idents.contains(no), "{no}");
    }
    assert!(usage.covers("[globals] If"));
    assert!(usage.covers("[nsh] View"));
    assert!(!usage.covers("[nsh] Text"));
    assert!(!usage.covers("[globals] Each"));
    assert!(!usage.covers("[props] ios.testID"));
    assert!(usage.covers("[props] List.header"));
    assert!(!usage.covers("[props] Video.header"));
    assert_eq!(
        without("c(li,{get header(){},header:1,x:header})", "header", false),
        "c(li,{get header__untested(){},header__untested:1,x:header})"
    );

    let runtime = "# x\n[nsh]\nView function\nView.x number\nA#.m function\n\n[nsh/list]\nList function\n\n[globals]\nArray function\nfetch function\n";
    let types = [
        (
            "nsh/globals.d.ts".to_owned(),
            "declare function fetch(input: string): Promise<unknown>;\ninterface Array<T> {}\n"
                .to_owned(),
        ),
        (
            "nsh/types.d.ts".to_owned(),
            "export interface HostProps {\n  testID?: string;\n  onPress?: () => void;\n}\n\
         export interface IOSInputHostProps {\n  /** d */\n  clearButtonMode?: string;\n}\n\
         export interface TextStyle extends ViewStyle {\n  color?: string;\n}\n\
         export interface Insets {\n  top: number;\n}\n"
                .to_owned(),
        ),
        (
            "nsh/list.d.ts".to_owned(),
            "export interface ListProps<T> extends HostProps {\n  testID?: string;\n  header?: T;\n  onEndReached?: () => void;\n  onEndReachedThreshold?: number;\n}\n\
             export interface RowProps {\n  x?: number;\n}\n"
                .to_owned(),
        ),
    ];
    assert_eq!(
        items(runtime, &types).0.into_iter().collect::<Vec<_>>(),
        [
            "[events] List.onEndReached",
            "[events] onPress",
            "[globals] fetch",
            "[nsh/list] List",
            "[nsh] View",
            "[props] List.header",
            "[props] List.onEndReachedThreshold",
            "[props] ios.clearButtonMode",
            "[props] testID",
            "[styles] color",
        ]
    );
}
