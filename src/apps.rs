//! Apps run through a script: an app is a directory with what
//! `nsh build --release` wrote for it
//! (`<app>/bundle.js`; packages always carry their source, which runs when
//! their bytecode is not this runtime's) and a script of what a user does
//! and sees (`<app>/script.txt`), one step per line (with the assets it
//! imports, `<app>/assets/`, and what the build wrote of an app that
//! declares `capabilities` in `nsh.json`: `<app>/capabilities.json` and its
//! id, `<app>/app.json`, which the app then runs with, as a package's do,
//! its data directory a new empty one):
//!
//! - `viewport <width> <height>`: before anything else, the screen's size
//!   (390 × 844 by default);
//! - `platform <os> [desktop]`: before anything else (with `viewport`), the
//!   platform the app runs on: `ios` (the default), `android`, `macos`,
//!   `windows`, `linux` or `web`; `desktop` makes a browser a desktop one.
//!   It decides which of `ios={{…}}`, `android={{…}}`, `desktop={{…}}`
//!   (the desktop OSes and desktop browsers) and `web={{…}}` apply;
//! - `expect <text>`: a text on screen reads `<text>` (waits for it);
//! - `absent <text>`: no text on screen reads `<text>` (after the app is
//!   quiet);
//! - `tap <selector>`: presses the first view on screen `id:<testID>`,
//!   `label:<accessibility label>` or `text:<text>` names, or its nearest
//!   pressable ancestor (as `nsh test`'s `press`); `span:<text>` presses
//!   the innermost pressable span (a nested `<Text>` with `onPress`, a
//!   link) where `<text>` starts in the first text that contains it, else
//!   the text's nearest pressable ancestor. The press carries where it
//!   is (the view's center, in it and on the surface; a span's in its
//!   text's view) and its click count: a `tap` right after a `tap` of the
//!   same view counts up (two in a row are a double click, as `nsh test`'s
//!   `press` twice), any other step between starts again at 1;
//! - `wait <ms>`: lets timers and animations run;
//! - `frame <selector> <x> <y> <width> <height>`: the first view on screen
//!   the selector names (itself, pressable or not) is laid out at that
//!   frame, in points relative to its parent (waits for it);
//! - `prop <selector> <name> <value>`: the first view on screen the
//!   selector names has host prop `<name>` (its JS name; a platform's props
//!   with or without their `ios.` / `android.` / `desktop.` / `web.`
//!   prefix) at `<value>` as the platform receives it, written as
//!   compat/values.rs says (waits for it);
//! - `noprop <selector> <name>`: the view has no such prop, or the value a
//!   view of its kind has with none set (after the app is quiet);
//! - `style <selector> <key> <value>`, `nostyle <selector> <key>`: the
//!   same for a style key, as mounted (compat/values.rs: paint keys, and
//!   the text keys of texts and inputs; layout keys show in `frame`s). A
//!   selector `span:<text>` names the first text that contains `<text>`,
//!   and its text keys are those of the run where `<text>` starts (nested
//!   `<Text>`s);
//! - `drawing <selector> <operation>`: the latest drawing of the canvas
//!   the selector names (what the platform was last told to draw: its
//!   `draw`, or what a component draws, with the view's paint styles
//!   `fill`, `stroke`, `strokeWidth` and `strokeDasharray` applied) has an
//!   operation that reads `<operation>` (the rest of the line), written as
//!   compat/drawing.rs says: `strokeStyle #ff0000ff`, `setLineDash 4 2`,
//!   `fillRect 0 0 20 10`; an `<operation>` ending in `...` is any that
//!   starts with what comes before (`fillText "Q1"...`). Waits for it;
//! - `nodrawing <selector> <operation>`: no operation of that drawing
//!   reads `<operation>` (after the app is quiet; a canvas that drew
//!   nothing has none);
//! - `animated <selector> <name> <value>`: an animation bound to the
//!   view the selector names (or, with none bound to it, to the first
//!   view inside it with one: a component's mark) sets `<name>` on it to
//!   `<value>` now, the animation graph run as a platform runs it beside
//!   its views, on the run's clock: `opacity`, `backgroundColor`,
//!   `borderColor`, `color`, `transform.<i>` (an entry of its transform
//!   list) or `slot.<i>` (an argument of its canvas's drawing), written as
//!   `style` and `drawing` write them (waits for it). What settles soon,
//!   as a hover's fade, is read at its end, and a long animation easing
//!   in at its start;
//! - `listens <selector> <event>`: the platform reports event `<event>`
//!   (`onPress`, ...) of the view, so its handler is set (waits for it);
//! - `log <text>`: the app printed a `console` line (any level) containing
//!   `<text>` (waits for it);
//! - `event <selector> <onEvent> [<key>=<value>...]`: the platform
//!   reports event `<onEvent>` of the first view on screen the selector
//!   names, or of its nearest ancestor that listens to it (as `listens`;
//!   it fails when none does), with a payload of the given keys (the
//!   handler's payload keys; a value is a number, `true`, `false` or a
//!   string, quoted when it has spaces or looks like a number), none
//!   without keys. A key given more than once makes a list (`onFileDrop`'s
//!   `uris`). `onMouseMove`, `onMouseEnter`, `onMouseLeave` and `onContextMenu` get their typed payload:
//!   `pageX` / `pageY` default to the listening view's position on screen
//!   plus `locationX` / `locationY`, the modifier keys to false;
//! - `touch start <selector> <x> <y>`, `touch move <x> <y>`,
//!   `touch end [<x> <y>]`, `touch cancel [<x> <y>]`: one finger goes
//!   down on the first view on screen the selector names at `<x> <y>` in
//!   it (the views around it that listen to touches get it; it fails when
//!   none does), moves or lifts there (positions in the view it went down
//!   on), or the system cancels it. The n-th touch event of a run has
//!   timestamp `16 * n` (ms);
//! - `type <selector> <text>`: the user types `<text>` (the rest of the
//!   line) into the input the selector names, one character at a time:
//!   `onKeyPress` (`key`), `onChange` (the whole text) and
//!   `onSelectionChange` (the caret after it) for each. It types after
//!   the input's text: the value the app last set, or what earlier `type`
//!   steps typed since;
//! - `key <selector> <key>`: `onKeyPress` with `key` (`Backspace`,
//!   `Enter`, ...), the text unchanged;
//! - `key <keys>`: a hardware keyboard's keystroke (`nsh test`'s
//!   `keyboard.press`), written as `nsh/keymap` writes them (`mod+k`,
//!   `shift+tab`, `down`, `escape`; `mod` is the platform's command key),
//!   on every platform: the newest binding of the app's keymap
//!   (`keymap.bind`, and the keys a focused control binds: a tree's or a
//!   carousel's arrows) that fires where the focus is hears it
//!   (`onKeyCommand`); Escape that no binding takes goes back as `back`
//!   does when there is something to go back from and no input has the
//!   focus; another key no binding takes reaches nothing. A control
//!   is focused by `event <selector> onFocus`, as the platform reports it;
//!   while an input has the focus (the app's `focus()`, or an `onFocus`
//!   step), only the bindings that fire while editing do;
//! - `back`: goes back as the user does (a header's back button, the back
//!   key; `nsh test`'s `back`): when the root listens to `onBackPress`
//!   (the core's overlays, `BackHandler`) and the back would not close a
//!   modal, it hears it; else in the newest modal (or the whole screen)
//!   the innermost stack that can go back tells its top screen
//!   (`onBackRequest`, when it listens) and pops it (`onDismiss`), or only
//!   tells it when it prevents going back (`preventBack`); without one, the
//!   newest modal gets `onRequestClose` (`reason` `cancel`) and closes
//!   (`onDismiss`) when `closeOnEscape`. A screen popped or a modal closed
//!   this way is gone until the app removes it. It fails when there is
//!   nothing to go back from. A back the app asks for (a back press no
//!   listener took) goes back so without asking, after any step;
//! - `scroll <selector> <x> <y>`: the scroll view the selector names
//!   scrolled to content offset `<x> <y>` (`onScroll` then reports it,
//!   with the sizes the core knows, and its `contentOffset` reads it) and
//!   at rest there: the animated value bound to its offset follows it, and
//!   the snapping values that follow that settle (what `animated` reads:
//!   a carousel's indicator, a header that hides as content scrolls);
//! - `fontscale <scale>`: before anything else (with `viewport`), the
//!   user's text size (1 by default), which scales the fonts of texts and
//!   inputs;
//! - `state <selector> <state>...`: the platform reports the interaction
//!   states of the first view on screen the selector names (itself): it
//!   is now `pressed`, `hovered` and / or `focused`, and in no other
//!   (`none`: in none). A platform reports only the states the core asked
//!   it to track (`activeStyle`, `hover:`, ...), so the step waits for the
//!   view to track the states it names and fails when it does not;
//! - `origin <selector> <x> <y>`: the first view on screen the selector
//!   names is at `<x> <y>` on the surface: its frames' origins added up,
//!   scroll offsets aside (a list's row, whose frame is its cell's), waits
//!   for it;
//! - `keyboard <height> <duration>`: the software keyboard shows at
//!   `<height>` points over the bottom of the surface, animated over
//!   `<duration>` seconds (`0 0`: it hides);
//! - `pixelratio <ratio>`: the surface moves to a screen of `<ratio>`
//!   device pixels per point (1 at the start), as a window moved to
//!   another screen does;
//! - `statusbar <key>=<value>...`: the status bar the platform was last
//!   told to show reads `<value>` for each `<key>` given (`foreground`:
//!   `auto`, `light` or `dark`; `hidden`, `animated`: `true` or `false`;
//!   compat/values.rs), waits for it;
//! - `covers <item>`: no step, a claim: the script checks what `<item>` (a
//!   part of the API as coverage.rs names it) does (coverage.rs
//!   verifies it).
//!
//! A selector is one word (`id:<testID>`, `label:<label>`, `text:<text>`,
//! `scroll:<selector>`, the nearest scroll view around the view the
//! inner selector names: a list, by one of its rows, and
//! `parent:<selector>`, the view around it: a component's popup, by the
//! app's content in it, and `below:<selector>`, its sibling below it: a
//! popup's backdrop);
//! the steps after `frame` also take the part after `:` quoted, for one
//! with spaces: `text:"Taps: 0"`. The first view a selector names is
//! looked for as `nsh test` resolves selectors: in the newest modal (and
//! what it holds) first, then in the rest in tree order. Of a navigation
//! stack only the top screen is on screen: the screens it covers and what
//! they hold are not (`expect` and `absent` never read their texts, a
//! user's steps never reach them), and the steps that read a view take one
//! in a covered screen only when no view on screen or hidden matches (the
//! options of screens pushed together). A hidden view (`visibility: hidden`, as
//! the cells a list keeps for reuse with their last row) and what it holds
//! are not on screen: `expect` and `absent` never read their texts, and
//! the steps a user takes (`tap`, `touch`, `type`, `key`, `scroll`,
//! `state`) never reach them; the other steps, which read a view, take a
//! hidden one only when no view on screen matches. Any error the app reports fails it. `NSH_COMPAT_APP=<name>` runs the apps whose name starts so; `NSH_COMPAT_DUMP=1` prints the texts
//! on screen after each step, for writing scripts.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, LazyLock, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread::ThreadId,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use nsh_core::{
    ApproxTextEngine, Command, ContextMenuEvent, EventKind, EventPayload, MountBatch, MountItem,
    MouseButton, MouseEvent, NodeId, PayloadValue, Points, PressEvent, Size, TextLink, TouchEvent,
    TouchPhase, TouchPoint, ViewKind,
    geometry::{DevicePixels, point, px},
    keymap::{Key, KeyMatch, Keymap, Keystroke, Modifiers},
    platform::{Environment, Idiom, Os},
    tailwind::state,
};
use nsh_engine::{Assets, Bundle, Engine, EngineConfig, EngineControl, MountSink};

use super::{
    drawing,
    values::{self, Mounted},
};

/// The surface's root view.
const ROOT: NodeId = 1;

/// How long an `expect` waits for its text.
const EXPECT_WAIT: Duration = Duration::from_secs(5);

/// How long a run waits for the app's first views: a large app (the
/// gallery) evaluates its bundle for seconds on a busy machine, with as
/// many apps starting at once as it has cores.
const START_WAIT: Duration = Duration::from_secs(30);

/// How long a `tap` waits at most for the app to be quiet first (a ticking
/// app never is).
const TAP_WAIT: Duration = Duration::from_millis(1000);

/// How long the app must send nothing to count as quiet.
const QUIET: Duration = Duration::from_millis(300);

/// The display's frame interval: a frame the engine asks for begins this
/// long after it asked, as at 60 Hz.
const FRAME: Duration = Duration::from_millis(16);

/// How many apps may be starting at once, as many as the machine has
/// cores. Starting (loading the bundle, the first render) is what costs;
/// the steps after it mostly wait, so the runs themselves stay side by
/// side. With every claim's app starting at once, a large app on a small
/// machine may not render within a step's wait, and fail for that alone.
static STARTS: LazyLock<(Mutex<usize>, Condvar)> = LazyLock::new(|| {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    (Mutex::new(cores), Condvar::new())
});

/// A turn to start an app, given back when dropped.
struct StartTurn;

impl StartTurn {
    fn take() -> StartTurn {
        let (free, freed) = &*STARTS;
        let mut free = freed.wait_while(free.lock().unwrap(), |n| *n == 0).unwrap();
        *free -= 1;
        StartTurn
    }
}

impl Drop for StartTurn {
    fn drop(&mut self) {
        let (free, freed) = &*STARTS;
        *free.lock().unwrap() += 1;
        freed.notify_one();
    }
}

struct Sink {
    batches: Sender<MountBatch>,
    errors: Arc<Mutex<Vec<String>>>,
    /// The engine's thread, which runs the app's JS: what it logs is the
    /// app's.
    thread: Arc<OnceLock<ThreadId>>,
    /// The engine's requests for a display frame (`requestAnimationFrame`).
    frames: Sender<()>,
}

impl MountSink for Sink {
    fn submit(&self, batch: MountBatch) {
        self.thread.get_or_init(|| std::thread::current().id());
        let _ = self.batches.send(batch);
    }

    fn report_error(&self, message: &str) {
        self.errors.lock().unwrap().push(message.to_owned());
    }

    fn request_frame(&self) {
        let _ = self.frames.send(());
    }
}

/// `asset:/<path>` from the app's `assets/`.
struct AppAssets(std::path::PathBuf);

impl Assets for AppAssets {
    fn read(&self, uri: &str) -> Result<Vec<u8>, String> {
        let path = uri
            .strip_prefix("asset:/")
            .ok_or_else(|| format!("not an asset URI: {uri}"))?;
        std::fs::read(self.0.join(path)).map_err(|e| format!("{uri}: {e}"))
    }
}

/// The native view tree as the mount items built it.
#[derive(Default)]
struct Views {
    nodes: HashMap<NodeId, View>,
    children: HashMap<NodeId, Vec<NodeId>>,
    /// The app's keyboard shortcuts (`nsh/keymap`, the keys a focused
    /// control binds): the surface's last `Keymap`.
    keymap: Option<Arc<Keymap>>,
    /// The view with the keyboard focus: an input the app focused, or a
    /// view an `event` step reported focused.
    focus: Option<NodeId>,
    /// Screens a back popped natively, until the app removes them.
    popped: HashSet<NodeId>,
    /// Modals a back closed natively, until the app removes them.
    closed: HashSet<NodeId>,
    /// The app asked for a back (`Command::Back`: a back press no listener
    /// took), which the run carries out.
    back_asked: bool,
}

/// What a back does ([`Views::back_target`]), as `nsh test` decides it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Back {
    /// The stack pops this screen.
    Pop(NodeId),
    /// This screen prevents going back (`preventBack`): it is asked.
    Ask(NodeId),
    /// The modal asks to close, and closes when the user may close it.
    Close {
        modal: NodeId,
        close_on_escape: bool,
    },
    /// Nothing to go back from.
    Nothing,
}

/// Which views a selector looks among.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reach {
    /// The views on screen ([`Views::shown`]).
    Shown,
    /// Those and the hidden ones ([`Views::attached`]).
    Attached,
    /// Those and the ones in the screens stacks cover ([`Views::all`]).
    All,
}

#[derive(Default)]
struct View {
    parent: Option<NodeId>,
    pressable: bool,
    test_id: Option<Arc<str>>,
    label: Option<Arc<str>>,
    text: Option<String>,
    /// Its text's pressable spans.
    links: Vec<TextLink>,
    frame: Option<Rect>,
    /// What the platform got for it, as `prop` and `style` read it.
    mounted: Mounted,
}

impl Views {
    fn apply(&mut self, batch: &MountBatch) {
        for item in &batch.items {
            match item {
                MountItem::Create { id, .. } => {
                    self.nodes.insert(*id, View::default());
                }
                MountItem::Insert {
                    parent,
                    child,
                    before,
                } => {
                    self.detach(*child);
                    let siblings = self.children.entry(*parent).or_default();
                    let at = before
                        .and_then(|b| siblings.iter().position(|s| *s == b))
                        .unwrap_or(siblings.len());
                    siblings.insert(at, *child);
                    self.nodes.entry(*child).or_default().parent = Some(*parent);
                }
                MountItem::Remove { id } => {
                    self.detach(*id);
                    self.popped.remove(id);
                    self.closed.remove(id);
                }
                MountItem::Delete { id } => {
                    self.detach(*id);
                    self.nodes.remove(id);
                    self.children.remove(id);
                    self.popped.remove(id);
                    self.closed.remove(id);
                    if self.focus == Some(*id) {
                        self.focus = None;
                    }
                }
                MountItem::Keymap(keymap) => self.keymap = Some(keymap.clone()),
                // The app's `focus()` and `blur()` (on the root, of
                // whatever has the focus), as the platforms apply them.
                MountItem::Command {
                    id,
                    command: Command::Focus,
                } if self.is_input(*id) => self.focus = Some(*id),
                MountItem::Command {
                    id,
                    command: Command::Blur,
                } if *id == ROOT || self.focus == Some(*id) => self.focus = None,
                MountItem::Command {
                    command: Command::Back,
                    ..
                } => self.back_asked = true,
                MountItem::Events { id, events } => {
                    self.nodes.entry(*id).or_default().pressable =
                        events.contains(EventKind::Press);
                }
                MountItem::ViewProps { id, props } => {
                    let view = self.nodes.entry(*id).or_default();
                    view.test_id = props.test_id.clone();
                    view.label = props.accessibility.label.clone();
                }
                MountItem::Text { id, text, .. } => {
                    let view = self.nodes.entry(*id).or_default();
                    view.text = Some(text.text.clone());
                    view.links = text.links.clone();
                }
                MountItem::Frame { id, frame, .. } => {
                    self.nodes.entry(*id).or_default().frame = Some(Rect::from(*frame));
                }
                _ => {}
            }
            values::animated(Some(item));
            if let Some(id) = Mounted::target(item) {
                self.nodes.entry(id).or_default().mounted.apply(item);
            }
        }
    }

    fn detach(&mut self, id: NodeId) {
        if let Some(parent) = self.nodes.get_mut(&id).and_then(|v| v.parent.take())
            && let Some(siblings) = self.children.get_mut(&parent)
        {
            siblings.retain(|s| *s != id);
        }
    }

    fn is_input(&self, id: NodeId) -> bool {
        self.nodes.get(&id).is_some_and(|v| v.mounted.is_input())
    }

    /// The platform reported view `id` focused or blurred (an `event`
    /// step's `onFocus` / `onBlur`).
    fn focused(&mut self, id: NodeId, focused: bool) {
        if focused {
            self.focus = Some(id);
        } else if self.focus == Some(id) {
            self.focus = None;
        }
    }

    /// What a hardware keyboard's `keystroke` does with the app's keymap,
    /// as the platforms look it up: the newest binding of it that fires
    /// where the focus is (while an input on screen has it, only those
    /// that fire while editing).
    fn key_match(&self, keystroke: Keystroke) -> KeyMatch {
        self.keymap
            .as_ref()
            .map_or(KeyMatch::None, |k| k.find(keystroke, self.editing(), false))
    }

    /// Whether an input on screen has the focus.
    fn editing(&self) -> bool {
        self.focus
            .is_some_and(|id| self.is_input(id) && self.shown().contains(&id))
    }

    /// Whether view `id` reports `event`.
    fn hears(&self, id: NodeId, event: EventKind) -> bool {
        self.nodes
            .get(&id)
            .is_some_and(|v| v.mounted.listens(event))
    }

    /// The screens stack `id` shows, bottom to top (without those a back
    /// popped).
    fn screens(&self, id: NodeId) -> Vec<NodeId> {
        self.children
            .get(&id)
            .map(|c| {
                c.iter()
                    .copied()
                    .filter(|s| !self.popped.contains(s))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What going back does now, as a platform's back does it (`nsh
    /// test`'s): in the newest modal (else the whole screen) the innermost
    /// stack that can go back pops its top screen, or asks it when it
    /// prevents going back; without one, the newest modal asks to close.
    fn back_target(&self) -> Back {
        let attached = self.attached();
        let modal = attached
            .iter()
            .rev()
            .copied()
            .find(|id| self.kind(*id) == Some(ViewKind::Modal));
        let prevents = |id: NodeId| {
            self.nodes
                .get(&id)
                .is_some_and(|v| v.mounted.prevent_back())
        };
        let can_go_back = |stack: NodeId| {
            let screens = self.screens(stack);
            screens.len() >= 2 || screens.last().is_some_and(|s| prevents(*s))
        };
        let stacks: Vec<NodeId> = attached
            .iter()
            .copied()
            .filter(|id| {
                self.kind(*id) == Some(ViewKind::Stack)
                    && modal.is_none_or(|m| self.is_inside(*id, m))
                    && can_go_back(*id)
            })
            .collect();
        let innermost = stacks
            .iter()
            .copied()
            .find(|s| !stacks.iter().any(|o| o != s && self.is_inside(*o, *s)));
        if let Some(stack) = innermost
            && let Some(&top) = self.screens(stack).last()
        {
            return if prevents(top) {
                Back::Ask(top)
            } else {
                Back::Pop(top)
            };
        }
        match modal {
            Some(modal) => Back::Close {
                modal,
                close_on_escape: self
                    .nodes
                    .get(&modal)
                    .is_some_and(|v| v.mounted.close_on_escape()),
            },
            None => Back::Nothing,
        }
    }

    /// Goes back as a platform does (`nsh test`'s `back`): the events it
    /// reports, in order, with the screen it pops or the modal it closes
    /// taken off natively; `None` when nothing could go back. `ask`: the
    /// root's back listener hears it first, outside modals (the overlays'
    /// `onCancel`, `BackHandler`; what none takes comes back as the app's
    /// `Command::Back`, which goes back without asking).
    fn back(&mut self, ask: bool) -> Option<Vec<(NodeId, EventKind, EventPayload)>> {
        let target = self.back_target();
        if ask && !matches!(target, Back::Close { .. }) && self.hears(ROOT, EventKind::BackPress) {
            return Some(vec![(ROOT, EventKind::BackPress, EventPayload::None)]);
        }
        let mut events = Vec::new();
        match target {
            Back::Pop(screen) => {
                // Platforms tell a page that listens of every back press.
                if self.hears(screen, EventKind::BackRequest) {
                    events.push((screen, EventKind::BackRequest, EventPayload::None));
                }
                self.popped.insert(screen);
                events.push((screen, EventKind::Dismiss, EventPayload::None));
            }
            Back::Ask(screen) => {
                events.push((screen, EventKind::BackRequest, EventPayload::None));
            }
            Back::Close {
                modal,
                close_on_escape,
            } => {
                let reason =
                    EventPayload::Map(vec![("reason", PayloadValue::Str("cancel".into()))]);
                events.push((modal, EventKind::RequestClose, reason));
                if close_on_escape {
                    self.closed.insert(modal);
                    events.push((modal, EventKind::Dismiss, EventPayload::None));
                }
            }
            Back::Nothing => return None,
        }
        Some(events)
    }

    /// The views attached below the root, in tree order.
    fn attached(&self) -> Vec<NodeId> {
        self.walk(true)
    }

    /// The views on screen: those attached below the root that neither
    /// they nor a view around them hide (`visibility: hidden`: a list
    /// keeps the cells it does not use, with their last row, hidden for
    /// reuse) and no screen covers, in tree order, as `nsh test` finds
    /// them.
    fn shown(&self) -> Vec<NodeId> {
        self.walk(false)
    }

    /// Every view below the root in tree order, the screens a stack covers
    /// and hidden views included: what a step that reads a view falls back
    /// to last.
    fn all(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![ROOT];
        while let Some(id) = stack.pop() {
            if id != ROOT {
                out.push(id);
            }
            if let Some(children) = self.children.get(&id) {
                stack.extend(children.iter().rev());
            }
        }
        out
    }

    /// The views below the root in tree order, of a navigation stack only
    /// its top screen (the others are covered, as `nsh test` sees them),
    /// without the screens and modals a back took away; without `hidden`,
    /// a hidden view and what it holds are left out.
    fn walk(&self, hidden: bool) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![ROOT];
        while let Some(id) = stack.pop() {
            if self.closed.contains(&id) {
                continue;
            }
            if id != ROOT {
                if !hidden && self.nodes.get(&id).is_some_and(|v| v.mounted.hidden()) {
                    continue;
                }
                out.push(id);
            }
            if let Some(children) = self.children.get(&id) {
                if self.kind(id) == Some(ViewKind::Stack) {
                    stack.extend(self.screens(id).last());
                } else {
                    stack.extend(children.iter().rev());
                }
            }
        }
        out
    }

    fn kind(&self, id: NodeId) -> Option<ViewKind> {
        self.nodes.get(&id)?.mounted.kind()
    }

    /// Whether `id` is `ancestor` or inside it.
    fn is_inside(&self, mut id: NodeId, ancestor: NodeId) -> bool {
        loop {
            if id == ancestor {
                return true;
            }
            match self.nodes.get(&id).and_then(|v| v.parent) {
                Some(parent) => id = parent,
                None => return false,
            }
        }
    }

    /// `views` (in tree order) in the order a user's eyes find them, as
    /// `nsh test` resolves selectors: the newest modal (the last in the
    /// tree) and what it holds first, then the rest.
    fn by_layer(&self, views: Vec<NodeId>) -> Vec<NodeId> {
        let Some(top) = views
            .iter()
            .rev()
            .copied()
            .find(|id| self.kind(*id) == Some(ViewKind::Modal))
        else {
            return views;
        };
        let (mut first, rest): (Vec<NodeId>, Vec<NodeId>) =
            views.into_iter().partition(|id| self.is_inside(*id, top));
        first.extend(rest);
        first
    }

    /// The texts on screen: a text laid out to no size is not (the core
    /// gives a view out of the layout, `display: none` on it or around it,
    /// a frame of 0 × 0, as a `keepMounted` panel's content while closed).
    fn texts(&self) -> Vec<String> {
        self.shown()
            .iter()
            .filter_map(|id| {
                let view = self.nodes.get(id)?;
                let no_size = view
                    .frame
                    .is_some_and(|f| f.width == 0.0 && f.height == 0.0);
                (!no_size).then_some(view.text.as_deref()?)
            })
            .map(|t| t.trim().to_owned())
            .collect()
    }

    fn shows(&self, text: &str) -> bool {
        self.texts().iter().any(|t| t == text)
    }

    /// The first view on screen `selector` names, or its nearest pressable
    /// ancestor.
    fn find(&self, selector: &str) -> Option<NodeId> {
        self.find_shown(selector)
            .and_then(|id| self.pressable_from(id))
    }

    /// For `span:<text>`, the innermost pressable span where `<text>`
    /// starts in the first text on screen that contains it.
    fn link(&self, selector: &str) -> Option<NodeId> {
        let want = selector.strip_prefix("span:")?;
        let view = self.nodes.get(&self.find_shown(selector)?)?;
        let text = view.text.as_deref()?;
        let at = text[..text.find(want)?].encode_utf16().count() as u32;
        view.links
            .iter()
            .filter(|l| l.start <= at && at < l.end)
            .min_by_key(|l| l.end - l.start)
            .map(|l| l.node)
    }

    /// The first view on screen `selector` names, or, when none on screen
    /// does, the first hidden one (`visibility: hidden`), else the first in
    /// a screen a stack covers, so a step can read what such a view itself
    /// was given (a covered screen's options): a view on screen comes
    /// first, a list's spare cell or a covered screen's view never stands
    /// in for it.
    fn find_view(&self, selector: &str) -> Option<NodeId> {
        self.select(selector, Reach::Shown)
            .or_else(|| self.select(selector, Reach::Attached))
            .or_else(|| self.select(selector, Reach::All))
    }

    /// The first view on screen `selector` names, never a hidden or covered
    /// one: what the user's taps, touches, typing and scrolls reach.
    fn find_shown(&self, selector: &str) -> Option<NodeId> {
        self.select(selector, Reach::Shown)
    }

    /// The first view among those `reach` takes that `selector` names.
    fn select(&self, selector: &str, reach: Reach) -> Option<NodeId> {
        let found = self.select_in(selector, reach)?;
        // The root is the platform's: it always shows.
        (reach != Reach::Shown || found == ROOT || self.shown().contains(&found)).then_some(found)
    }

    fn select_in(&self, selector: &str, reach: Reach) -> Option<NodeId> {
        if let Some(inner) = selector.strip_prefix("scroll:") {
            return self.scroll_around(inner, reach);
        }
        // The view around another: a component's own part (a popup) that
        // only the app's content in it names.
        if let Some(inner) = selector.strip_prefix("parent:") {
            return self.nodes.get(&self.select_in(inner, reach)?)?.parent;
        }
        // The view below another among its siblings (a popup's backdrop).
        if let Some(inner) = selector.strip_prefix("below:") {
            let id = self.select_in(inner, reach)?;
            let siblings = self.children.get(&self.nodes.get(&id)?.parent?)?;
            let at = siblings.iter().position(|&s| s == id)?;
            return at.checked_sub(1).map(|i| siblings[i]);
        }
        let is = |value: &Option<Arc<str>>, want: &str| value.as_deref() == Some(want);
        let test: Box<dyn Fn(&View) -> bool> = if let Some(want) = selector.strip_prefix("id:") {
            Box::new(move |v| is(&v.test_id, want))
        } else if let Some(want) = selector.strip_prefix("label:") {
            Box::new(move |v| is(&v.label, want))
        } else if let Some(want) = selector.strip_prefix("text:") {
            Box::new(move |v| v.text.as_deref().map(str::trim) == Some(want))
        } else if let Some(want) = selector.strip_prefix("span:") {
            Box::new(move |v| v.text.as_deref().is_some_and(|t| t.contains(want)))
        } else {
            panic!("selector `{selector}`: expected id:, label:, text: or span:")
        };
        let views = match reach {
            Reach::Shown => self.shown(),
            Reach::Attached => self.attached(),
            Reach::All => self.all(),
        };
        self.by_layer(views)
            .into_iter()
            .find(|id| self.nodes.get(id).is_some_and(&test))
    }

    /// The nearest scroll view around the first view `selector` names (a
    /// list has no `testID` of its own: its rows name it), among those
    /// `reach` takes.
    fn scroll_around(&self, selector: &str, reach: Reach) -> Option<NodeId> {
        let mut id = self.select_in(selector, reach)?;
        loop {
            id = self.nodes.get(&id)?.parent?;
            if self.nodes.get(&id)?.mounted.is_scroll() {
                return Some(id);
            }
        }
    }

    /// The mounted state of the first view on screen `selector` names, and
    /// for `span:<text>`, the UTF-16 offset in its text where `<text>`
    /// starts.
    fn mounted(&self, selector: &str) -> Option<(&Mounted, Option<u32>)> {
        let view = self.nodes.get(&self.find_view(selector)?)?;
        let at = selector.strip_prefix("span:").and_then(|want| {
            let text = view.text.as_deref()?;
            let byte = text.find(want)?;
            Some(text[..byte].encode_utf16().count() as u32)
        });
        Some((&view.mounted, at))
    }

    /// What the animations bound to the view `selector` names set on it,
    /// or, when none is, on the first view inside it they are bound to (a
    /// component's mark wrapped in views of its own, as `drawing_of`).
    fn animated_of(&self, selector: &str) -> Option<Vec<(String, String)>> {
        let mut stack = vec![self.find_view(selector)?];
        while let Some(id) = stack.pop() {
            let live = values::live(id);
            if !live.is_empty() {
                return Some(live);
            }
            if let Some(children) = self.children.get(&id) {
                stack.extend(children.iter().rev());
            }
        }
        Some(Vec::new())
    }

    /// What the view `selector` names draws, or, when it draws nothing of
    /// its own, the first canvas inside it (a component's mark wrapped in
    /// views of its own: a plot's `Area`, a `Pie`'s first slice).
    fn drawing_of(&self, selector: &str) -> Option<Vec<String>> {
        let mut stack = vec![self.find_view(selector)?];
        while let Some(id) = stack.pop() {
            let view = self.nodes.get(&id)?;
            if view.mounted.draws() {
                return Some(view.mounted.drawing());
            }
            if let Some(children) = self.children.get(&id) {
                stack.extend(children.iter().rev());
            }
        }
        Some(Vec::new())
    }

    /// The outermost view around `id`: the root the app renders into.
    fn top_of(&self, mut id: NodeId) -> NodeId {
        while let Some(parent) = self.nodes.get(&id).and_then(|v| v.parent) {
            if parent == ROOT {
                break;
            }
            id = parent;
        }
        id
    }

    fn pressable_from(&self, mut id: NodeId) -> Option<NodeId> {
        loop {
            let view = self.nodes.get(&id)?;
            if view.pressable {
                return Some(id);
            }
            id = view.parent?;
        }
    }
}

struct Run {
    _engine: Engine,
    control: EngineControl,
    rx: Receiver<MountBatch>,
    errors: Arc<Mutex<Vec<String>>>,
    thread: Arc<OnceLock<ThreadId>>,
    views: Views,
}

impl Run {
    fn start(
        bundle: Bundle,
        viewport: Size<Points>,
        environment: Environment,
        assets: &Path,
        data_dir: Option<PathBuf>,
    ) -> Run {
        crate::logs::capture();
        let (tx, rx) = crossbeam_channel::unbounded();
        let errors = Arc::new(Mutex::new(Vec::new()));
        let thread = Arc::new(OnceLock::new());
        let config = EngineConfig {
            viewport,
            trace: false,
            network: None,
            websocket: None,
            filesystem: None,
            assets: Some(Arc::new(AppAssets(assets.to_path_buf()))),
            environment,
            header_heights: (nsh_core::Points(44.0), nsh_core::Points(96.0)),
            limits: nsh_engine::Limits::default(),
            ui_sets: vec![nsh_base::registry()],
            runtime: None,
            data_dir,
            storage: None,
            clock: Default::default(),
            pinned: Default::default(),
            clipboard: None,
            ..EngineConfig::new(bundle)
        };
        let (frames, requests) = crossbeam_channel::unbounded();
        let sink = Sink {
            batches: tx,
            errors: errors.clone(),
            thread: thread.clone(),
            frames,
        };
        let engine = Engine::start(config, || ApproxTextEngine, sink).unwrap();
        // The display: each frame the engine asks for begins a frame
        // interval later. It ends with the engine, which holds the sink.
        let control = engine.control();
        std::thread::spawn(move || {
            while requests.recv().is_ok() {
                std::thread::sleep(FRAME);
                control.frame();
            }
        });
        Run {
            control: engine.control(),
            _engine: engine,
            rx,
            errors,
            thread,
            views: Views::default(),
        }
    }

    /// Applies `batch`, and goes back when the app asked (a back press no
    /// listener took), as the platforms do.
    fn apply(&mut self, batch: &MountBatch) {
        self.views.apply(batch);
        if std::mem::take(&mut self.views.back_asked) {
            self.go_back(false);
        }
    }

    /// Goes back as a platform does ([`Views::back`]); false when nothing
    /// could.
    fn go_back(&mut self, ask: bool) -> bool {
        let Some(events) = self.views.back(ask) else {
            return false;
        };
        for (id, kind, payload) in events {
            self.control.dispatch_event(id, kind, payload);
        }
        true
    }

    /// Whether the app printed a line containing `text`.
    fn printed(&self, text: &str) -> bool {
        self.thread
            .get()
            .is_some_and(|t| crate::logs::printed_by(*t).iter().any(|l| l.contains(text)))
    }

    /// Lets the app run until `done` holds or `limit` ends.
    fn poll(&mut self, limit: Duration, done: impl Fn(&Run) -> bool) -> bool {
        let end = Instant::now() + limit;
        loop {
            if done(self) {
                return true;
            }
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            match self.rx.recv_timeout(left.min(FRAME)) {
                Ok(batch) => self.apply(&batch),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => panic!("the engine stopped"),
            }
            self.frame();
        }
    }

    /// Applies the batches that arrive until `done` holds or `limit` ends.
    fn until(&mut self, limit: Duration, done: impl Fn(&Views) -> bool) -> bool {
        let end = Instant::now() + limit;
        while !done(&self.views) {
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            // A frame at a time: an animation that ends may be what makes
            // the app change.
            match self.rx.recv_timeout(left.min(FRAME)) {
                Ok(batch) => self.apply(&batch),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => panic!("the engine stopped"),
            }
            self.frame();
        }
        true
    }

    /// Applies batches until none came for [`QUIET`] (or `limit` ends).
    fn settle(&mut self, limit: Duration) {
        let end = Instant::now() + limit;
        let mut last = Instant::now();
        loop {
            let now = Instant::now();
            if now >= end || now - last >= QUIET {
                return;
            }
            let left = (end - now).min(QUIET - (now - last));
            match self.rx.recv_timeout(left.min(FRAME)) {
                Ok(batch) => {
                    self.apply(&batch);
                    last = Instant::now();
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            self.frame();
        }
    }

    /// Runs the animation graph to now and reports the animations that
    /// ended, as a platform does each frame.
    fn frame(&self) {
        let ended = values::tick();
        if !ended.is_empty() {
            self.control.animations_finished(ended);
        }
    }

    fn errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }
}

/// Runs the app in `dir` (`bundle.js`, `script.txt`); the first failed step
/// is the error.
pub(crate) fn run_app(dir: &Path) -> Result<(), String> {
    let code = std::fs::read_to_string(dir.join("bundle.js")).map_err(|e| e.to_string())?;
    let script = std::fs::read_to_string(dir.join("script.txt")).map_err(|e| e.to_string())?;
    run(code, &script, dir).map_err(|f| f.message)
}

/// `code` as the bundle of the fixture app in `dir`: with the capabilities
/// it declared and its id, when its build wrote them.
fn bundle(code: String, dir: &Path) -> Result<(Bundle, bool), String> {
    let bundle = Bundle::source(code);
    let Ok(json) = std::fs::read_to_string(dir.join("capabilities.json")) else {
        return Ok((bundle, false));
    };
    let id = std::fs::read_to_string(dir.join("app.json"))
        .ok()
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .and_then(|v| v.get("id")?.as_str().map(str::to_owned));
    let bundle = bundle
        .with_app_id(id.as_deref())
        .with_capabilities_json(Some(&json))
        .map_err(|e| format!("{}: {e}", dir.join("capabilities.json").display()))?;
    Ok((bundle, true))
}

/// A new, empty data directory for one run of an app, removed with it.
struct DataDir(PathBuf);

impl DataDir {
    fn new() -> DataDir {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        let dir = crate::temporary_root().join(format!(
            "nsh-compat-data-{}-{}",
            std::process::id(),
            RUNS.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        DataDir(dir)
    }
}

impl Drop for DataDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Why a script failed.
pub(crate) struct Failure {
    /// The step that failed, when one of its checks did (not an error the
    /// app reported, nor a malformed script).
    pub(crate) check: Option<String>,
    /// Whether the app reported errors.
    pub(crate) errors: bool,
    pub(crate) message: String,
}

impl From<String> for Failure {
    fn from(message: String) -> Failure {
        Failure {
            check: None,
            errors: false,
            message,
        }
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Failure {
        message.to_owned().into()
    }
}

/// Runs the bundle `code` of the fixture app in `dir` (its assets and
/// declared capabilities) through `script`.
pub(crate) fn run(code: String, script: &str, dir: &Path) -> Result<(), Failure> {
    let dump = std::env::var_os("NSH_COMPAT_DUMP").is_some();
    let mut lines = script
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .peekable();
    let mut viewport = Size::new(nsh_core::geometry::px(390.0), nsh_core::geometry::px(844.0));
    let mut environment = Environment::default();
    while let Some(line) = lines.peek() {
        if let Some(size) = line.strip_prefix("viewport ") {
            let (w, h) = size.split_once(' ').ok_or("viewport <width> <height>")?;
            viewport = Size::new(px(w.parse().unwrap()), px(h.parse().unwrap()));
        } else if let Some(platform) = line.strip_prefix("platform ") {
            let mut words = platform.split_whitespace();
            environment.os = match words.next() {
                Some("ios") => Os::Ios,
                Some("android") => Os::Android,
                Some("macos") => Os::MacOs,
                Some("windows") => Os::Windows,
                Some("linux") => Os::Linux,
                Some("web") => Os::Web,
                _ => {
                    return Err(format!(
                        "`{line}`: platform ios|android|macos|windows|linux|web [desktop]"
                    )
                    .into());
                }
            };
            let desktop = match words.next() {
                None => environment.os.is_desktop(),
                Some("desktop") => true,
                Some(other) => return Err(format!("`{line}`: `{other}`, not `desktop`").into()),
            };
            if desktop {
                environment.idiom = Idiom::Desktop;
            }
        } else if let Some(scale) = line.strip_prefix("fontscale ") {
            environment.font_scale = scale
                .trim()
                .parse()
                .map_err(|_| format!("`{line}`: fontscale <scale>"))?;
        } else {
            break;
        }
        lines.next();
    }
    // The surface's size, as a platform reports it with every environment:
    // `keyboard` and `pixelratio` send this one again.
    environment.window = viewport;
    values::animated(None);
    let turn = StartTurn::take();
    let (bundle, declares) = bundle(code, dir)?;
    // Declared before the run, so dropped after its engine, which may
    // still write there.
    let data = declares.then(DataDir::new);
    let os = environment.os;
    let mut run = Run::start(
        bundle,
        viewport,
        environment,
        &dir.join("assets"),
        data.as_ref().map(|d| d.0.clone()),
    );
    if !run.until(START_WAIT, |v| !v.attached().is_empty()) {
        return Err(Failure {
            check: None,
            errors: !run.errors().is_empty(),
            message: format!("the app rendered nothing; errors: {:?}", run.errors()),
        });
    }
    drop(turn);
    if dump {
        run.settle(EXPECT_WAIT);
        println!("(start)\n  {:?}", run.views.texts());
    }
    let mut shown = run.views.texts();
    let mut device = Device::default();
    // `NSH_COMPAT_KEEP_GOING=1`: a failed step skips to the next case
    // (`tap id:case-…`) instead of ending the run, and every case's first
    // failure is reported at the end.
    let keep_going = std::env::var_os("NSH_COMPAT_KEEP_GOING").is_some();
    let mut failures: Vec<Failure> = Vec::new();
    let mut skipping = false;
    for (n, line) in lines.enumerate() {
        if skipping {
            if !line.starts_with("tap id:case-") {
                continue;
            }
            skipping = false;
            device.last_press = None;
        }
        let (step, arg) = line.split_once(' ').unwrap_or((line, ""));
        // Only a `tap` right after a `tap` counts up its clicks.
        if step != "tap" {
            device.last_press = None;
        }
        let failed = |what: String, run: &Run| {
            let errors = run.errors();
            Err(Failure {
                check: (what != "the app reported errors").then(|| line.to_owned()),
                errors: !errors.is_empty(),
                message: format!(
                    "step {} `{line}`: {what}\non screen: {:?}\nerrors: {errors:?}",
                    n + 1,
                    run.views.texts(),
                ),
            })
        };
        let mut step_run = || -> Result<(), Failure> {
            match step {
                "expect" => {
                    if !run.until(EXPECT_WAIT, |v| v.shows(arg)) {
                        return failed("not shown".into(), &run);
                    }
                }
                "absent" => {
                    run.settle(EXPECT_WAIT);
                    if run.views.shows(arg) {
                        return failed("still shown".into(), &run);
                    }
                }
                "tap" => {
                    run.settle(TAP_WAIT);
                    let link = run.views.link(arg);
                    let Some(id) = link.or_else(|| run.views.find(arg)) else {
                        return failed("nothing to press".into(), &run);
                    };
                    // A span has no frame of its own: the press is in its
                    // text's view.
                    let at = match link {
                        Some(_) => run.views.find_shown(arg).unwrap_or(id),
                        None => id,
                    };
                    let payload = device.press(&run.views, id, at);
                    run.control.dispatch_event(id, EventKind::Press, payload);
                }
                "wait" => {
                    let ms: u64 = arg.parse().map_err(|_| format!("wait `{arg}`"))?;
                    run.until(Duration::from_millis(ms), |_| false);
                }
                "frame" => {
                    // The selector may quote a text with spaces (`text:"Taps: 0"`).
                    let (selector, rest) = selector(arg);
                    let selector = selector.as_str();
                    let want: Vec<f32> = rest.split(' ').filter_map(|p| p.parse().ok()).collect();
                    let [x, y, w, h] = want[..] else {
                        return Err(
                            format!("frame `{arg}`: <selector> <x> <y> <width> <height>").into(),
                        );
                    };
                    let at =
                        |v: &Views| v.find_view(selector).and_then(|id| v.nodes.get(&id)?.frame);
                    let fits = |v: &Views| {
                        at(v).is_some_and(|f| {
                            [f.x - x, f.y - y, f.width - w, f.height - h]
                                .iter()
                                .all(|d| d.abs() < 0.5)
                        })
                    };
                    if !run.until(EXPECT_WAIT, fits) {
                        let what = match at(&run.views) {
                            Some(f) => {
                                format!("laid out at {} {} {} {}", f.x, f.y, f.width, f.height)
                            }
                            None => "no such view".to_owned(),
                        };
                        return failed(what, &run);
                    }
                }
                "prop" | "style" => {
                    let (selector, rest) = selector(arg);
                    let (name, want) = rest
                        .split_once(' ')
                        .ok_or_else(|| format!("`{line}`: {step} <selector> <name> <value>"))?;
                    let (key, want) = (values::key(name), want.trim());
                    let all = |v: &Views| {
                        let (m, at) = v.mounted(&selector)?;
                        Some(if step == "prop" {
                            m.props()
                        } else {
                            m.styles(at)
                        })
                    };
                    let read = |v: &Views| all(v)?.remove(key);
                    if !run.until(EXPECT_WAIT, |v| read(v).as_deref() == Some(want)) {
                        let what = match all(&run.views) {
                            None => "no such view".to_owned(),
                            Some(all) => {
                                // What the view has, for writing the step.
                                let has: Vec<String> =
                                    all.iter().map(|(k, v)| format!("{k} {v}")).collect();
                                let got = match all.get(key) {
                                    None => format!("no {step} `{key}`"),
                                    Some(got) => format!("`{key}` is `{got}`"),
                                };
                                format!("{got}\nthe view's {step}s:\n  {}", has.join("\n  "))
                            }
                        };
                        return failed(what, &run);
                    }
                }
                "noprop" | "nostyle" => {
                    let (selector, name) = selector(arg);
                    let key = values::key(name.trim());
                    run.settle(EXPECT_WAIT);
                    let Some((m, at)) = run.views.mounted(&selector) else {
                        return failed("no such view".into(), &run);
                    };
                    let (got, unset) = if step == "noprop" {
                        (m.props().remove(key), m.blank().props().remove(key))
                    } else {
                        (m.styles(at).remove(key), m.blank().styles(at).remove(key))
                    };
                    if let Some(got) = got.filter(|g| Some(g) != unset.as_ref()) {
                        return failed(format!("`{key}` is `{got}`"), &run);
                    }
                }
                "drawing" => {
                    let (selector, want) = selector(arg);
                    let want = want.trim();
                    if want.is_empty() {
                        return Err(format!("`{line}`: drawing <selector> <operation>").into());
                    }
                    let ops = |v: &Views| v.drawing_of(&selector);
                    let has = |v: &Views| {
                        ops(v).is_some_and(|ops| ops.iter().any(|op| drawing::matches(op, want)))
                    };
                    if !run.until(EXPECT_WAIT, has) {
                        let what = match ops(&run.views) {
                            None => "no such view".to_owned(),
                            Some(ops) if ops.is_empty() => "it drew nothing".to_owned(),
                            // What it draws, for writing the step.
                            Some(ops) => format!(
                                "no operation `{want}`\nthe drawing's operations:\n  {}",
                                ops.join("\n  ")
                            ),
                        };
                        return failed(what, &run);
                    }
                }
                "nodrawing" => {
                    let (selector, want) = selector(arg);
                    let want = want.trim();
                    if want.is_empty() {
                        return Err(format!("`{line}`: nodrawing <selector> <operation>").into());
                    }
                    run.settle(EXPECT_WAIT);
                    let Some(ops) = run.views.drawing_of(&selector) else {
                        return failed("no such view".into(), &run);
                    };
                    if let Some(op) = ops.iter().find(|op| drawing::matches(op, want)) {
                        return failed(format!("it draws `{op}`"), &run);
                    }
                }
                "animated" => {
                    let (selector, rest) = selector(arg);
                    let (name, want) = rest
                        .split_once(' ')
                        .ok_or_else(|| format!("`{line}`: animated <selector> <name> <value>"))?;
                    let want = want.trim();
                    let read = |run: &Run| run.views.animated_of(&selector);
                    let has = |run: &Run| {
                        read(run).is_some_and(|all| all.iter().any(|(k, v)| k == name && v == want))
                    };
                    if !run.poll(EXPECT_WAIT, has) {
                        let what = match read(&run) {
                            None => "no such view".to_owned(),
                            Some(all) if all.is_empty() => "no animation is bound to it".to_owned(),
                            // What is bound, for writing the step.
                            Some(all) => format!(
                                "`{name}` is not `{want}`\nwhat its animations set:\n  {}",
                                all.iter()
                                    .map(|(k, v)| format!("{k} {v}"))
                                    .collect::<Vec<_>>()
                                    .join("\n  ")
                            ),
                        };
                        return failed(what, &run);
                    }
                }
                "listens" => {
                    let (selector, event) = selector(arg);
                    let event = event.trim();
                    let kinds = event_kinds(event);
                    if kinds.is_empty() {
                        return Err(format!("`{line}`: `{event}` is no event").into());
                    }
                    let listens = |v: &Views| {
                        v.mounted(&selector)
                            .is_some_and(|(m, _)| kinds.iter().any(|&k| m.listens(k)))
                    };
                    if !run.until(EXPECT_WAIT, listens) {
                        return failed(format!("`{event}` not reported"), &run);
                    }
                }
                "log" => {
                    if !run.poll(EXPECT_WAIT, |r| r.printed(arg)) {
                        return failed("not printed".into(), &run);
                    }
                }
                "event" => {
                    let (selector, rest) = selector(arg);
                    let rest = rest.trim();
                    let (name, fields) = rest.split_once(' ').unwrap_or((rest, ""));
                    let kinds = event_kinds(name);
                    if kinds.is_empty() {
                        return Err(format!("`{line}`: `{name}` is no event").into());
                    }
                    let fields = parse_fields(fields).map_err(|e| format!("`{line}`: {e}"))?;
                    run.settle(TAP_WAIT);
                    // The first of the events the name may be that a view
                    // around reports (`onCancel`: a dialog's, an overlay's).
                    let found = run.views.find_view(&selector).and_then(|id| {
                        kinds
                            .iter()
                            .find_map(|&k| listener(&run.views, id, k).map(|at| (at, k)))
                            // The host's messages go to the root whether
                            // anything listens there or not, as the
                            // platforms send them.
                            .or_else(|| {
                                kinds
                                    .contains(&EventKind::Host)
                                    .then(|| (run.views.top_of(id), EventKind::Host))
                            })
                    });
                    let Some((id, kind)) = found else {
                        return failed(format!("nothing there reports `{name}`"), &run);
                    };
                    let payload = event_payload(&run.views, id, kind, fields, &mut device)
                        .map_err(|e| format!("`{line}`: {e}"))?;
                    run.control.dispatch_event(id, kind, payload);
                    // The keys a control binds while focused follow.
                    match kind {
                        EventKind::Focus => run.views.focused(id, true),
                        EventKind::Blur => run.views.focused(id, false),
                        _ => {}
                    }
                }
                "touch" => {
                    let (phase, rest) = arg.split_once(' ').unwrap_or((arg, ""));
                    run.settle(TAP_WAIT);
                    let touch = match phase {
                        "start" => {
                            let (selector, at) = selector(rest);
                            let at = numbers::<2>(at).ok_or_else(|| {
                                format!("`{line}`: touch start <selector> <x> <y>")
                            })?;
                            let Some(target) = run.views.find_shown(&selector) else {
                                return failed("no such view".into(), &run);
                            };
                            let path = touch_path(&run.views, target);
                            if path.is_empty() {
                                return failed("nothing there listens to touches".into(), &run);
                            }
                            device.finger = Some((target, path, at));
                            device.touch(&run.views, TouchPhase::Start)
                        }
                        "move" | "end" | "cancel" => {
                            let Some(finger) = device.finger.as_mut() else {
                                return Err(format!("`{line}`: no finger is down").into());
                            };
                            if let Some(at) = numbers::<2>(rest) {
                                finger.2 = at;
                            } else if phase == "move" || !rest.trim().is_empty() {
                                return Err(format!("`{line}`: touch {phase} <x> <y>").into());
                            }
                            let phase = match phase {
                                "move" => TouchPhase::Move,
                                "end" => TouchPhase::End,
                                _ => TouchPhase::Cancel,
                            };
                            let touch = device.touch(&run.views, phase);
                            if phase != TouchPhase::Move {
                                device.finger = None;
                            }
                            touch
                        }
                        _ => return Err(format!("`{line}`: touch start|move|end|cancel").into()),
                    };
                    run.control.dispatch_event(
                        ROOT,
                        EventKind::Touch,
                        EventPayload::Touch(Box::new(touch)),
                    );
                }
                // `key <keys>`: a hardware keyboard's keystroke, for the
                // app's keymap.
                "key" if !is_selector(arg) => {
                    let keys = arg.trim();
                    let stroke = Keystroke::parse(keys, os)
                        .map_err(|e| format!("`{line}`: key <keys>: {e}"))?;
                    run.settle(TAP_WAIT);
                    let m = stroke.modifiers;
                    let plain = !(m.ctrl || m.alt || m.cmd || m.shift);
                    match run.views.key_match(stroke) {
                        KeyMatch::Fire(id) => run.control.dispatch_event(
                            ROOT,
                            EventKind::KeyCommand,
                            EventPayload::Value(PayloadValue::Number(f64::from(id))),
                        ),
                        // Escape no binding takes goes back, as the
                        // desktops' back key does, when an input does not
                        // have the focus and there is something to go back
                        // from.
                        KeyMatch::None
                            if stroke.key == Key::Escape
                                && plain
                                && !run.views.editing()
                                && run.views.back_target() != Back::Nothing =>
                        {
                            run.go_back(true);
                        }
                        _ => {}
                    }
                }
                "back" => {
                    run.settle(TAP_WAIT);
                    if !run.go_back(true) {
                        return failed("nothing to go back from".into(), &run);
                    }
                }
                "type" | "key" => {
                    let (selector, text) = selector(arg);
                    run.settle(TAP_WAIT);
                    let Some(id) = run.views.find_shown(&selector) else {
                        return failed("no such view".into(), &run);
                    };
                    let key = |key: &str| {
                        EventPayload::Map(vec![("key", PayloadValue::Str(key.to_owned()))])
                    };
                    if step == "key" {
                        run.control
                            .dispatch_event(id, EventKind::KeyPress, key(text.trim()));
                    } else {
                        let value = run
                            .views
                            .nodes
                            .get(&id)
                            .and_then(|v| v.mounted.props().remove("value"));
                        let typed = device.typed.entry(id).or_default();
                        // A value the app set since replaces what was typed.
                        if typed.1 != value {
                            typed.0 = value.clone().unwrap_or_default();
                            typed.1 = value;
                        }
                        for c in text.chars() {
                            typed.0.push(c);
                            typed.2 += 1;
                            let caret = PayloadValue::Number(typed.0.encode_utf16().count() as f64);
                            run.control.dispatch_event(
                                id,
                                EventKind::KeyPress,
                                key(&c.to_string()),
                            );
                            run.control.dispatch_event(
                                id,
                                EventKind::ChangeText,
                                EventPayload::Text {
                                    text: typed.0.clone(),
                                    count: typed.2,
                                },
                            );
                            run.control.dispatch_event(
                                id,
                                EventKind::SelectionChange,
                                EventPayload::Map(vec![("start", caret.clone()), ("end", caret)]),
                            );
                        }
                    }
                }
                "scroll" => {
                    let (selector, at) = selector(arg);
                    let [x, y] = numbers::<2>(at)
                        .ok_or_else(|| format!("`{line}`: scroll <selector> <x> <y>"))?;
                    run.settle(TAP_WAIT);
                    let Some(id) = run.views.find_shown(&selector) else {
                        return failed("no such view".into(), &run);
                    };
                    run.control.scroll(id, point(px(x), px(y)), 0);
                    if let Some(view) = run.views.nodes.get_mut(&id) {
                        view.mounted.scrolled(x, y);
                    }
                    values::scrolled(id, x, y);
                }
                "origin" => {
                    let (selector, at) = selector(arg);
                    let [x, y] = numbers::<2>(at)
                        .ok_or_else(|| format!("`{line}`: origin <selector> <x> <y>"))?;
                    let at = |v: &Views| v.find_view(&selector).map(|id| page_origin(v, id));
                    let fits = |v: &Views| {
                        at(v).is_some_and(|(left, top)| {
                            (left - x).abs() < 0.5 && (top - y).abs() < 0.5
                        })
                    };
                    if !run.until(EXPECT_WAIT, fits) {
                        let what = match at(&run.views) {
                            Some((left, top)) => format!("at {left} {top}"),
                            None => "no such view".to_owned(),
                        };
                        return failed(what, &run);
                    }
                }
                "state" => {
                    let (selector, names) = selector(arg);
                    let mut states = 0;
                    for name in names.split_whitespace() {
                        states |= match name {
                            "pressed" => state::PRESSED,
                            "hovered" => state::HOVERED,
                            "focused" => state::FOCUSED,
                            "none" => 0,
                            _ => {
                                return Err(format!(
                                    "`{line}`: state <selector> pressed|hovered|focused|none..."
                                )
                                .into());
                            }
                        };
                    }
                    let tracks = |v: &Views| {
                        v.find_shown(&selector)
                            .filter(|id| v.nodes.get(id).is_some_and(|n| n.mounted.tracks(states)))
                    };
                    if !run.until(EXPECT_WAIT, |v| tracks(v).is_some()) {
                        return failed(format!("the platform does not track `{names}`"), &run);
                    }
                    let id = tracks(&run.views).unwrap();
                    run.control.set_states(id, states);
                }
                "keyboard" => {
                    let [height, duration] = numbers::<2>(arg)
                        .ok_or_else(|| format!("`{line}`: keyboard <height> <duration>"))?;
                    environment.keyboard_height = px(height);
                    environment.keyboard_duration = duration;
                    run.control.set_environment(environment);
                }
                "pixelratio" => {
                    let [ratio] = numbers::<1>(arg)
                        .filter(|[r]| *r > 0.0)
                        .ok_or_else(|| format!("`{line}`: pixelratio <ratio>"))?;
                    environment.pixel_ratio = ratio;
                    run.control.set_environment(environment);
                }
                "statusbar" => {
                    let want = parse_fields(arg).map_err(|e| format!("`{line}`: {e}"))?;
                    if want.is_empty() {
                        return Err(format!("`{line}`: statusbar <key>=<value>...").into());
                    }
                    let want: Vec<(String, String)> = want
                        .into_iter()
                        .map(|(k, f)| {
                            let v = match f {
                                Field::Bool(b) => b.to_string(),
                                Field::Str(s) => s,
                                Field::Number(n) => values::num(n as f32),
                            };
                            (k, v)
                        })
                        .collect();
                    for (key, _) in &want {
                        if !["foreground", "hidden", "animated"].contains(&key.as_str()) {
                            return Err(format!("`{line}`: no status bar key `{key}`").into());
                        }
                    }
                    let read = |v: &Views| v.nodes.get(&ROOT)?.mounted.status_bar();
                    let fits = |v: &Views| {
                        read(v).is_some_and(|got| {
                            want.iter()
                                .all(|(k, v)| got.get(k.as_str()).is_some_and(|g| g == v))
                        })
                    };
                    if !run.until(EXPECT_WAIT, fits) {
                        let what = match read(&run.views) {
                            None => "the platform was told no status bar".to_owned(),
                            Some(got) => {
                                let got: Vec<String> =
                                    got.iter().map(|(k, v)| format!("{k}={v}")).collect();
                                format!("the status bar is {}", got.join(" "))
                            }
                        };
                        return failed(what, &run);
                    }
                }
                "covers" => return Ok(()),
                other => return Err(format!("unknown step `{other}`").into()),
            }
            if dump {
                run.settle(EXPECT_WAIT);
                let now = run.views.texts();
                let added: Vec<_> = now.iter().filter(|t| !shown.contains(t)).collect();
                let gone: Vec<_> = shown.iter().filter(|t| !now.contains(t)).collect();
                println!("{line}\n  + {added:?}\n  - {gone:?}");
                shown = now;
            }
            if !run.errors().is_empty() {
                return failed("the app reported errors".into(), &run);
            }
            Ok(())
        };
        if let Err(failure) = step_run() {
            if !keep_going {
                return Err(failure);
            }
            failures.push(failure);
            skipping = true;
        }
    }
    if let Some(first) = failures.first() {
        // The first failure's check, so the claims of `covers` read as
        // without the mode.
        return Err(Failure {
            check: first.check.clone(),
            errors: first.errors,
            message: failures
                .iter()
                .map(|f| f.message.as_str())
                .collect::<Vec<_>>()
                .join("\n\n"),
        });
    }
    run.settle(EXPECT_WAIT);
    match run.errors() {
        errors if errors.is_empty() => Ok(()),
        errors => Err(Failure {
            check: None,
            errors: true,
            message: format!("the app reported errors: {errors:?}"),
        }),
    }
}

/// Splits a step's argument into its selector and the rest: the selector
/// is one word, or a prefix and a quoted value (`text:"Taps: 0"`).
fn selector(arg: &str) -> (String, &str) {
    if let Some((prefix, quoted)) = arg.split_once(":\"")
        && !prefix.contains(' ')
        && let Some((value, rest)) = quoted.split_once('"')
    {
        return (format!("{prefix}:{value}"), rest.trim_start());
    }
    match arg.split_once(' ') {
        Some((selector, rest)) => (selector.to_owned(), rest),
        None => (arg.to_owned(), ""),
    }
}

/// Whether `arg`'s first word is a selector (`id:`, `label:`, ...), not a
/// keystroke.
fn is_selector(arg: &str) -> bool {
    let word = arg.split_whitespace().next().unwrap_or("");
    [
        "id:", "label:", "text:", "span:", "scroll:", "parent:", "below:",
    ]
    .iter()
    .any(|p| word.starts_with(p))
}

/// What the steps that stand in for the platform's input keep between
/// steps.
#[derive(Default)]
struct Device {
    /// The finger down: the view it went down on, the views around it
    /// that listen to touches (innermost first) and where it is in the
    /// view.
    finger: Option<(NodeId, Vec<NodeId>, [f32; 2])>,
    /// Touch events so far.
    touches: u32,
    /// Per input: its text, the value the app had set when it was read,
    /// and the edits reported.
    typed: HashMap<NodeId, (String, Option<String>, u32)>,
    /// The view the previous step pressed (a `tap`) and its clicks so far.
    last_press: Option<(NodeId, u32)>,
}

impl Device {
    /// The payload of a press of `id` at the center of view `at` (itself,
    /// or the text a pressed span is in), with its click count: a press of
    /// the view the previous step pressed counts up (2 is a double click),
    /// as presses within the platforms' double-click time do (`nsh test`'s
    /// `press` twice). Counted by the script's steps, not by the time
    /// between them, which waits for the app first. No payload for a view
    /// with no frame, as `nsh test` sends none.
    fn press(&mut self, views: &Views, id: NodeId, at: NodeId) -> EventPayload {
        let clicks = match self.last_press {
            Some((last, clicks)) if last == id => clicks + 1,
            _ => 1,
        };
        self.last_press = Some((id, clicks));
        let Some(frame) = views.nodes.get(&at).and_then(|v| v.frame) else {
            return EventPayload::None;
        };
        let (left, top) = page_origin(views, at);
        let (x, y) = (frame.width / 2.0, frame.height / 2.0);
        EventPayload::Press(PressEvent {
            position: point(px(x), px(y)),
            page_position: point(px(left + x), px(top + y)),
            modifiers: Modifiers::default(),
            clicks,
        })
    }

    /// The finger's touch event of `phase`.
    fn touch(&mut self, views: &Views, phase: TouchPhase) -> TouchEvent {
        let (target, path, [x, y]) = self.finger.clone().expect("a finger down");
        self.touches += 1;
        let (left, top) = page_origin(views, target);
        TouchEvent {
            phase,
            timestamp: 16.0 * self.touches as f64,
            activated: None,
            points: vec![TouchPoint {
                id: 1,
                target,
                path,
                page: point(px(left + x), px(top + y)),
                location: point(px(x), px(y)),
                changed: true,
            }],
        }
    }
}

/// A value of an `event` step's payload.
enum Field {
    Number(f64),
    Bool(bool),
    Str(String),
}

impl Field {
    fn number(&self) -> Option<f32> {
        match self {
            Field::Number(n) => Some(*n as f32),
            _ => None,
        }
    }

    fn value(self) -> PayloadValue {
        match self {
            Field::Number(n) => PayloadValue::Number(n),
            Field::Bool(b) => PayloadValue::Bool(b),
            Field::Str(s) => PayloadValue::Str(s),
        }
    }
}

/// `key=value key="a value" ...`.
fn parse_fields(text: &str) -> Result<Vec<(String, Field)>, String> {
    let mut out = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        let (key, after) = rest
            .split_once('=')
            .ok_or_else(|| format!("`{rest}`: expected <key>=<value>"))?;
        let (field, after) = if let Some(quoted) = after.strip_prefix('"') {
            let (value, after) = quoted
                .split_once('"')
                .ok_or_else(|| format!("`{after}`: unterminated quote"))?;
            (Field::Str(value.to_owned()), after)
        } else {
            let (value, after) = after.split_once(' ').unwrap_or((after, ""));
            let field = match value {
                "true" => Field::Bool(true),
                "false" => Field::Bool(false),
                _ => match value.parse::<f64>() {
                    Ok(n) => Field::Number(n),
                    Err(_) => Field::Str(value.to_owned()),
                },
            };
            (field, after)
        };
        out.push((key.trim().to_owned(), field));
        rest = after.trim_start();
    }
    Ok(out)
}

/// The event a script's `on…` name reports: a host element's handler as
/// apps write it (`onChange`), a handler of `nsh/base`'s `NavPage` and
/// `Dialog` (the runtime's own listeners of the platform's events), or the
/// platform protocol's name (`onChangeText`).
/// The events a handler's name may be, the components' first: a name a
/// component handles itself (`Dialog`'s `onCancel`) may also be a host
/// element's (`Overlay`'s).
fn event_kinds(name: &str) -> Vec<EventKind> {
    let component = match name {
        // `NavPage`.
        "onDismiss" => Some(EventKind::Dismiss),
        "onBackPress" => Some(EventKind::BackRequest),
        // `Dialog`.
        "onCancel" | "onOpenChange" => Some(EventKind::RequestClose),
        "onOpen" => Some(EventKind::Show),
        "onClose" => Some(EventKind::Dismiss),
        _ => None,
    };
    let host = EventKind::from_handler(name).or_else(|| EventKind::from_prop(name));
    let mut kinds: Vec<EventKind> = component.into_iter().chain(host).collect();
    kinds.dedup();
    kinds
}

/// The payload of an `event` step, as the platforms make it for `kind`.
fn event_payload(
    views: &Views,
    id: NodeId,
    kind: EventKind,
    fields: Vec<(String, Field)>,
    device: &mut Device,
) -> Result<EventPayload, String> {
    let get = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, f)| f);
    let num = |key: &str| get(key).and_then(Field::number);
    let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
    let only = |want: &[&str]| keys.len() == want.len() && want.iter().all(|k| keys.contains(k));
    Ok(match kind {
        // The mouse's payloads and a context menu's are a position and
        // the modifiers.
        EventKind::MouseEnter
        | EventKind::MouseMove
        | EventKind::MouseLeave
        | EventKind::ContextMenu => {
            let (left, top) = page_origin(views, id);
            // The keys as handlers read them (`position.x`, `modifiers.shift`),
            // or as scripts written before v0.1.2 name them (`locationX`,
            // `shiftKey`): scripts never change.
            let either = |key: &str, old: &str| num(key).or_else(|| num(old));
            let (location_x, location_y) = (
                either("position.x", "locationX").unwrap_or(0.0),
                either("position.y", "locationY").unwrap_or(0.0),
            );
            let flag = |key: &str, old: &str| {
                matches!(get(key), Some(Field::Bool(true)))
                    || matches!(get(old), Some(Field::Bool(true)))
            };
            let (page_x, page_y) = (
                either("pagePosition.x", "pageX").unwrap_or(left + location_x),
                either("pagePosition.y", "pageY").unwrap_or(top + location_y),
            );
            let modifiers = Modifiers {
                ctrl: flag("modifiers.control", "ctrlKey"),
                alt: flag("modifiers.alt", "altKey"),
                shift: flag("modifiers.shift", "shiftKey"),
                cmd: flag("modifiers.platform", "metaKey"),
            };
            if kind != EventKind::ContextMenu {
                // The button held down in a drag, as handlers name it.
                let pressed = match get("pressedButton") {
                    Some(Field::Str(b)) => match b.as_str() {
                        "left" => Some(MouseButton::Left),
                        "right" => Some(MouseButton::Right),
                        "middle" => Some(MouseButton::Middle),
                        "back" => Some(MouseButton::Back),
                        "forward" => Some(MouseButton::Forward),
                        other => return Err(format!("no mouse button `{other}`")),
                    },
                    _ => None,
                };
                EventPayload::Mouse(MouseEvent::new(
                    point(px(location_x), px(location_y)),
                    point(px(page_x), px(page_y)),
                    modifiers,
                    pressed,
                ))
            } else {
                EventPayload::ContextMenu(ContextMenuEvent {
                    position: point(px(location_x), px(location_y)),
                    page_position: point(px(page_x), px(page_y)),
                    modifiers,
                })
            }
        }
        _ if fields.is_empty() => EventPayload::None,
        EventKind::ChangeText => {
            // A number a script writes is the digits typed.
            let text = match get("text") {
                Some(Field::Str(text)) => text.clone(),
                Some(Field::Number(n)) => n.to_string(),
                _ => return Err("onChange takes text=<string>".into()),
            };
            let text = &text;
            let typed = device.typed.entry(id).or_default();
            typed.0 = text.clone();
            typed.2 += 1;
            EventPayload::Text {
                text: text.clone(),
                count: typed.2,
            }
        }
        EventKind::FileDrop if keys.iter().all(|k| *k == "uris") => EventPayload::FileDrop(
            fields
                .into_iter()
                .map(|(_, f)| match f {
                    Field::Str(s) => s,
                    Field::Number(n) => n.to_string(),
                    Field::Bool(b) => b.to_string(),
                })
                .collect(),
        ),
        EventKind::PanStart | EventKind::PanEnd => EventPayload::Pan {
            translation: point(
                px(num("translationX").unwrap_or(0.0)),
                px(num("translationY").unwrap_or(0.0)),
            ),
            velocity_x: num("velocityX").unwrap_or(0.0),
            velocity_y: num("velocityY").unwrap_or(0.0),
            value_x: num("valueX").map(px),
            value_y: num("valueY").map(px),
        },
        EventKind::ScrollEnd
        | EventKind::ScrollDragStart
        | EventKind::ScrollDragEnd
        | EventKind::MomentumStart
        | EventKind::MomentumEnd
            if only(&["x", "y"]) =>
        {
            EventPayload::Scroll(point(
                px(num("x").unwrap_or(0.0)),
                px(num("y").unwrap_or(0.0)),
            ))
        }
        // The return key: the input's text, as the platforms send it
        // (a number a script writes is the digits typed).
        EventKind::SubmitEditing if only(&["text"]) => {
            let text = match get("text") {
                Some(Field::Str(text)) => text.clone(),
                Some(Field::Number(n)) => n.to_string(),
                _ => return Err("onSubmit takes text=<string>".into()),
            };
            let count = device.typed.get(&id).map_or(0, |typed| typed.2);
            EventPayload::Text { text, count }
        }
        EventKind::Load if only(&["width", "height"]) => EventPayload::Image(Size::new(
            DevicePixels(num("width").unwrap_or(0.0) as i32),
            DevicePixels(num("height").unwrap_or(0.0) as i32),
        )),
        EventKind::Error if only(&["message"]) => match get("message") {
            Some(Field::Str(message)) => EventPayload::Error {
                message: message.clone(),
            },
            _ => return Err("onError takes message=<string>".into()),
        },
        _ => EventPayload::Map(
            fields
                .into_iter()
                // The payload's keys are static: a test's few leak.
                .map(|(k, f)| (&*Box::leak(k.into_boxed_str()), f.value()))
                .collect(),
        ),
    })
}

/// `id`, or its nearest ancestor, that the platform reports `kind` of.
fn listener(views: &Views, mut id: NodeId, kind: EventKind) -> Option<NodeId> {
    loop {
        let view = views.nodes.get(&id)?;
        if view.mounted.listens(kind) {
            return Some(id);
        }
        id = view.parent?;
    }
}

/// `id` and the views around it that listen to touches, innermost first.
fn touch_path(views: &Views, id: NodeId) -> Vec<NodeId> {
    let mut path = Vec::new();
    let mut at = Some(id);
    while let Some(id) = at {
        let Some(view) = views.nodes.get(&id) else {
            break;
        };
        if view.mounted.listens(EventKind::Touch) {
            path.push(id);
        }
        at = view.parent;
    }
    path
}

/// A view's frame in points relative to its parent, as the steps read it.
#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl From<nsh_core::Bounds<Points>> for Rect {
    fn from(b: nsh_core::Bounds<Points>) -> Self {
        Rect {
            x: b.origin.x.0,
            y: b.origin.y.0,
            width: b.size.width.0,
            height: b.size.height.0,
        }
    }
}

/// Where view `id` is on screen: its frames' origins added up.
fn page_origin(views: &Views, mut id: NodeId) -> (f32, f32) {
    let (mut x, mut y) = (0.0, 0.0);
    while let Some(view) = views.nodes.get(&id) {
        if let Some(frame) = view.frame {
            x += frame.x;
            y += frame.y;
        }
        match view.parent {
            Some(parent) => id = parent,
            None => break,
        }
    }
    (x, y)
}

/// `N` numbers separated by spaces.
fn numbers<const N: usize>(text: &str) -> Option<[f32; N]> {
    let parsed: Vec<f32> = text
        .split_whitespace()
        .map(|w| w.parse().ok())
        .collect::<Option<_>>()?;
    parsed.try_into().ok()
}

#[cfg(test)]
mod tests {
    use nsh_core::{AttributedText, EventMask, ViewKind, ViewProps, ViewStyle};

    use super::*;

    fn create(id: NodeId, kind: ViewKind) -> MountItem {
        MountItem::Create { id, kind }
    }

    fn insert(parent: NodeId, child: NodeId) -> MountItem {
        MountItem::Insert {
            parent,
            child,
            before: None,
        }
    }

    fn text(id: NodeId, value: &str) -> MountItem {
        MountItem::Text {
            id,
            text: Arc::new(AttributedText {
                text: value.into(),
                ..Default::default()
            }),
            layout: nsh_core::TextLayout(Arc::new(())),
            content: Default::default(),
        }
    }

    fn hidden(id: NodeId, hidden: bool) -> MountItem {
        let style = ViewStyle {
            hidden,
            ..Default::default()
        };
        MountItem::ViewStyle {
            id,
            style: Box::new(style),
        }
    }

    fn test_id(id: NodeId, value: &str) -> MountItem {
        let props = ViewProps {
            test_id: Some(value.into()),
            ..Default::default()
        };
        MountItem::ViewProps {
            id,
            props: Box::new(props),
        }
    }

    fn pressable(id: NodeId) -> MountItem {
        let mut events = EventMask::default();
        events.set(EventKind::Press, true);
        MountItem::Events { id, events }
    }

    /// A list of two cells, the first a spare one a list hid for reuse
    /// with its last row, `Row 7`, which the second, on screen, shows too.
    fn list() -> Views {
        let mut views = Views::default();
        views.apply(&MountBatch {
            items: vec![
                create(2, ViewKind::Scroll),
                create(4, ViewKind::View),
                create(6, ViewKind::Text),
                create(8, ViewKind::View),
                create(10, ViewKind::Text),
                insert(ROOT, 2),
                insert(2, 4),
                insert(4, 6),
                insert(2, 8),
                insert(8, 10),
                pressable(4),
                pressable(8),
                test_id(4, "row"),
                test_id(8, "row"),
                text(6, "Row 7"),
                text(10, "Row 7"),
                hidden(4, true),
            ],
        });
        views
    }

    fn frame(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> MountItem {
        MountItem::Frame {
            id,
            frame: nsh_core::Bounds::new(point(px(x), px(y)), Size::new(px(width), px(height))),
        }
    }

    /// The click count and where a press is (in the view, on the surface).
    fn pressed(payload: EventPayload) -> (u32, [f32; 2], [f32; 2]) {
        let EventPayload::Press(e) = payload else {
            panic!("no press payload: {payload:?}");
        };
        (
            e.clicks,
            [e.position.x.0, e.position.y.0],
            [e.page_position.x.0, e.page_position.y.0],
        )
    }

    #[test]
    fn taps_in_a_row_on_one_view_count_up_their_clicks() {
        let mut views = list();
        views.apply(&MountBatch {
            items: vec![
                frame(2, 0.0, 100.0, 390.0, 400.0),
                frame(8, 10.0, 20.0, 200.0, 40.0),
                frame(4, 0.0, 0.0, 200.0, 40.0),
            ],
        });
        let mut device = Device::default();
        // A press is at the view's center, in it and on the surface.
        assert_eq!(
            pressed(device.press(&views, 8, 8)),
            (1, [100.0, 20.0], [110.0, 140.0])
        );
        // The same view again: a double click, then a triple.
        assert_eq!(pressed(device.press(&views, 8, 8)).0, 2);
        assert_eq!(pressed(device.press(&views, 8, 8)).0, 3);
        // Another view starts again, and so does a step between.
        assert_eq!(pressed(device.press(&views, 4, 4)).0, 1);
        device.last_press = None;
        assert_eq!(pressed(device.press(&views, 4, 4)).0, 1);
        // A span is pressed in its text's view; with no frame, no payload,
        // but the click still counts.
        assert_eq!(pressed(device.press(&views, 99, 4)).0, 1);
        assert!(matches!(device.press(&views, 4, 6), EventPayload::None));
        assert!(matches!(
            device.press(&views, 4, 4),
            EventPayload::Press(PressEvent { clicks: 2, .. })
        ));
    }

    #[test]
    fn keys_reach_the_newest_binding_that_fires_where_the_focus_is() {
        use nsh_core::keymap::{KeyBinding, When};

        let stroke = |keys: &str| Keystroke::parse(keys, Os::MacOs).unwrap();
        let binding =
            |id: u32, keys: &str, when: When| KeyBinding::new(id, stroke(keys), when, false);
        let mut views = list();
        // No keymap yet: no key fires.
        assert_eq!(views.key_match(stroke("down")), KeyMatch::None);
        views.apply(&MountBatch {
            items: vec![
                create(12, ViewKind::Input),
                insert(ROOT, 12),
                MountItem::Keymap(Arc::new(Keymap::new(vec![
                    binding(1, "mod+k", When::Auto),
                    binding(3, "down", When::NotEditing),
                    // A focused control's arrows, bound after the app's.
                    binding(5, "down", When::NotEditing),
                ]))),
            ],
        });
        assert_eq!(views.key_match(stroke("mod+k")), KeyMatch::Fire(1));
        assert_eq!(views.key_match(stroke("down")), KeyMatch::Fire(5));
        assert_eq!(views.key_match(stroke("up")), KeyMatch::None);

        // An input focused by the app edits: only what fires while editing.
        views.apply(&MountBatch {
            items: vec![MountItem::Command {
                id: 12,
                command: Command::Focus,
            }],
        });
        assert_eq!(views.key_match(stroke("mod+k")), KeyMatch::Fire(1));
        assert_eq!(views.key_match(stroke("down")), KeyMatch::None);
        // Blurred on the root, then focused and blurred as reported.
        views.apply(&MountBatch {
            items: vec![MountItem::Command {
                id: ROOT,
                command: Command::Blur,
            }],
        });
        assert_eq!(views.key_match(stroke("down")), KeyMatch::Fire(5));
        views.focused(12, true);
        assert_eq!(views.key_match(stroke("down")), KeyMatch::None);
        views.focused(8, false);
        assert_eq!(views.key_match(stroke("down")), KeyMatch::None);
        views.focused(12, false);
        assert_eq!(views.key_match(stroke("down")), KeyMatch::Fire(5));
        // A view that is no input holds the focus without editing.
        views.focused(8, true);
        assert_eq!(views.key_match(stroke("down")), KeyMatch::Fire(5));

        // The control's keys go with its focus: the keymap is replaced.
        views.apply(&MountBatch {
            items: vec![MountItem::Keymap(Arc::new(Keymap::new(vec![binding(
                3,
                "down",
                When::NotEditing,
            )])))],
        });
        assert_eq!(views.key_match(stroke("down")), KeyMatch::Fire(3));

        assert!(is_selector("id:tree down"));
        assert!(is_selector("text:\"A b\" Enter"));
        assert!(!is_selector("mod+k"));
        assert!(!is_selector("escape"));
    }

    #[test]
    fn a_scroll_moves_the_values_that_follow_it_and_settles_them() {
        use nsh_core::animation::{AnimationOp, AnimationProp};

        values::animated(None);
        let mut views = list();
        views.apply(&MountBatch {
            items: vec![
                MountItem::Animation(AnimationOp::CreateValue {
                    node: 2,
                    value: 0.0,
                }),
                // Follows the offset's changes within 0..1 and, at rest,
                // goes to the nearer end (a header hiding as content
                // scrolls).
                MountItem::Animation(AnimationOp::CreateDiffClamp {
                    node: 3,
                    input: 2,
                    min: 0.0,
                    max: 1.0,
                    snap: true,
                }),
                MountItem::ScrollConfig {
                    id: 2,
                    horizontal: false,
                    scrollbar_visible: true,
                    replaced: false,
                    scroll_value: Some(2),
                    pull_value: None,
                    keyboard_dismiss: Default::default(),
                    mirrored: false,
                },
                MountItem::AnimationBindings {
                    id: 8,
                    bindings: vec![(AnimationProp::Opacity, 3)],
                },
            ],
        });
        let opacity = || {
            values::live(8)
                .into_iter()
                .find(|(name, _)| name == "opacity")
                .map(|(_, value)| value)
        };
        assert_eq!(opacity().as_deref(), Some("0"));
        // Another scroll view moves nothing.
        values::scrolled(4, 0.0, 300.0);
        assert_eq!(opacity().as_deref(), Some("0"));
        values::scrolled(2, 0.0, 200.0);
        assert_eq!(opacity().as_deref(), Some("1"));
        // Back by three quarters, then at rest: the nearer end, 0.
        values::scrolled(2, 0.0, 199.25);
        let end = Instant::now() + Duration::from_secs(5);
        while opacity().as_deref() != Some("0") {
            assert!(Instant::now() < end, "never settled: {:?}", opacity());
            std::thread::sleep(FRAME);
        }
    }

    #[test]
    fn hidden_views_and_what_they_hold_are_not_on_screen() {
        let mut views = list();
        assert_eq!(views.texts(), ["Row 7"]);
        assert_eq!(views.shown(), [2, 8, 10]);
        assert_eq!(views.attached(), [2, 4, 6, 8, 10]);
        // The view on screen is found, never the spare cell before it.
        assert_eq!(views.find("text:Row 7"), Some(8));
        assert_eq!(views.find_view("id:row"), Some(8));
        assert_eq!(views.find_shown("parent:text:Row 7"), Some(8));
        assert_eq!(views.find_shown("scroll:text:Row 7"), Some(2));

        // The cell on screen hides too: its row is gone from the screen,
        // and a user's steps reach neither cell.
        views.apply(&MountBatch {
            items: vec![hidden(8, true)],
        });
        assert!(views.texts().is_empty());
        assert!(!views.shows("Row 7"));
        assert_eq!(views.find("text:Row 7"), None);
        assert_eq!(views.find_shown("id:row"), None);
        assert_eq!(views.find_shown("below:id:row"), None);
        // A step that reads a view still finds a hidden one.
        assert_eq!(views.find_view("id:row"), Some(4));

        // The first cell is used again: it shows its new row.
        views.apply(&MountBatch {
            items: vec![hidden(4, false), text(6, "Row 1")],
        });
        assert_eq!(views.texts(), ["Row 1"]);
        assert_eq!(views.find("text:Row 1"), Some(4));
    }

    #[test]
    fn only_the_top_screen_of_a_stack_and_the_top_modal_first_are_seen() {
        let mut views = Views::default();
        // A stack of two screens, each with a text and a button `go`.
        views.apply(&MountBatch {
            items: vec![
                create(2, ViewKind::Stack),
                create(4, ViewKind::Screen),
                create(6, ViewKind::Text),
                create(8, ViewKind::View),
                create(10, ViewKind::Screen),
                create(12, ViewKind::Text),
                create(14, ViewKind::View),
                insert(ROOT, 2),
                insert(2, 4),
                insert(4, 6),
                insert(4, 8),
                insert(2, 10),
                insert(10, 12),
                insert(10, 14),
                text(6, "Home"),
                text(12, "Details"),
                pressable(8),
                pressable(14),
                test_id(8, "go"),
                test_id(14, "go"),
            ],
        });
        // The pushed screen covers the first: a user's steps do not reach
        // what it covers.
        assert_eq!(views.texts(), ["Details"]);
        assert!(!views.shows("Home"));
        assert_eq!(views.find("id:go"), Some(14));
        assert_eq!(views.find("text:Home"), None);
        assert_eq!(views.find_shown("text:Home"), None);
        // A step that reads a view still reaches a covered screen's, when
        // nothing on screen matches.
        assert_eq!(views.find_view("text:Home"), Some(6));
        assert_eq!(views.find_view("id:go"), Some(14));

        // A modal after the stack: its button comes first, though the
        // stack's is before it in the tree.
        views.apply(&MountBatch {
            items: vec![
                create(16, ViewKind::Modal),
                create(18, ViewKind::View),
                insert(ROOT, 16),
                insert(16, 18),
                pressable(18),
                test_id(18, "go"),
            ],
        });
        assert_eq!(views.find("id:go"), Some(18));
        assert_eq!(views.find_view("id:go"), Some(18));

        // The app pops the screen and closes the modal: the first screen
        // shows again.
        views.apply(&MountBatch {
            items: vec![MountItem::Remove { id: 10 }, MountItem::Remove { id: 16 }],
        });
        assert_eq!(views.texts(), ["Home"]);
        assert_eq!(views.find("id:go"), Some(8));
    }

    fn listens(id: NodeId, kinds: &[EventKind]) -> MountItem {
        let mut events = EventMask::default();
        for &kind in kinds {
            events.set(kind, true);
        }
        MountItem::Events { id, events }
    }

    /// The views and kinds of the events a back reports.
    fn back(views: &mut Views, ask: bool) -> Option<Vec<(NodeId, EventKind)>> {
        let events = views.back(ask)?;
        Some(events.into_iter().map(|(id, kind, _)| (id, kind)).collect())
    }

    #[test]
    fn a_back_pops_the_top_screen_asks_one_that_prevents_it_and_closes_modals() {
        let mut views = Views::default();
        views.apply(&MountBatch {
            items: vec![
                create(2, ViewKind::Stack),
                create(4, ViewKind::Screen),
                create(6, ViewKind::Text),
                create(8, ViewKind::Screen),
                create(10, ViewKind::Text),
                insert(ROOT, 2),
                insert(2, 4),
                insert(4, 6),
                insert(2, 8),
                insert(8, 10),
                text(6, "Home"),
                text(10, "Details"),
                listens(8, &[EventKind::BackRequest]),
            ],
        });
        // The top screen hears the back and is popped: the one below shows.
        assert_eq!(views.back_target(), Back::Pop(8));
        assert_eq!(
            back(&mut views, true),
            Some(vec![(8, EventKind::BackRequest), (8, EventKind::Dismiss)])
        );
        assert_eq!(views.texts(), ["Home"]);
        // One screen left: nothing to go back from.
        assert_eq!(back(&mut views, true), None);
        // The app removes the popped screen and pushes one that prevents
        // going back: it is only asked.
        views.apply(&MountBatch {
            items: vec![
                MountItem::Remove { id: 8 },
                create(12, ViewKind::Screen),
                MountItem::ScreenConfig {
                    id: 12,
                    config: Box::new(nsh_core::ScreenConfig {
                        prevent_back: true,
                        ..Default::default()
                    }),
                },
                insert(2, 12),
            ],
        });
        assert_eq!(
            back(&mut views, true),
            Some(vec![(12, EventKind::BackRequest)])
        );
        assert_eq!(views.back_target(), Back::Ask(12));

        // A modal over it whose stack cannot go back: it is asked to close,
        // and closes when the user may close it.
        views.apply(&MountBatch {
            items: vec![
                create(14, ViewKind::Modal),
                create(16, ViewKind::Text),
                insert(ROOT, 14),
                insert(14, 16),
                text(16, "Sheet"),
            ],
        });
        assert_eq!(
            back(&mut views, true),
            Some(vec![(14, EventKind::RequestClose)])
        );
        assert!(views.shows("Sheet"));
        views.apply(&MountBatch {
            items: vec![MountItem::ModalConfig {
                id: 14,
                config: nsh_core::ModalConfig {
                    close_on_escape: true,
                    ..Default::default()
                },
            }],
        });
        assert_eq!(
            back(&mut views, true),
            Some(vec![
                (14, EventKind::RequestClose),
                (14, EventKind::Dismiss)
            ])
        );
        assert!(!views.shows("Sheet"));
        assert_eq!(views.back_target(), Back::Ask(12));

        // The root listens to the back button (an overlay, `BackHandler`):
        // it hears a back first, and what none takes comes back as the
        // app's own back, which goes back without asking.
        views.apply(&MountBatch {
            items: vec![listens(ROOT, &[EventKind::BackPress])],
        });
        assert_eq!(
            back(&mut views, true),
            Some(vec![(ROOT, EventKind::BackPress)])
        );
        views.apply(&MountBatch {
            items: vec![MountItem::Command {
                id: ROOT,
                command: Command::Back,
            }],
        });
        assert!(std::mem::take(&mut views.back_asked));
        assert_eq!(
            back(&mut views, false),
            Some(vec![(12, EventKind::BackRequest)])
        );
    }
}
