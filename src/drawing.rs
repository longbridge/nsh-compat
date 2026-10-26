//! What a canvas draws, written in the words apps use: the `drawing` and
//! `nodrawing` steps of the fixtures' scripts (apps.rs) compare against it.
//!
//! A canvas's drawing reaches the platform as one list of operations
//! (`MountItem::Canvas`), the latest replacing the one before. What a Rust
//! component draws with the canvas view's paint properties (`fill`,
//! `stroke`, `strokeWidth`, `strokeDasharray`, inherited from the views
//! around it) arrives with those properties resolved, so the operations
//! show what the app's styles made of it, and nothing else does: those
//! style keys reach no view of their own.
//!
//! As in values.rs, the core's types may change freely and the scripts are
//! frozen: every operation is written as the canvas method or property apps
//! call or set, one line each, never a Rust `Debug`. Change an operation's
//! form only by adding a new one.
//!
//! Forms, the numbers, colors and lists as values.rs writes them:
//!
//! - a method: its name and its arguments, separated by spaces, as apps
//!   pass them (`moveTo 0 10`, `fillRect 0 0 20 10`, `rotate 1.571`,
//!   angles in radians); `save`, `restore`, `beginPath`, `closePath` alone;
//!   `arc` and `ellipse` end in `true` when counterclockwise, and
//!   `roundRect` takes its four radii (top left, top right, bottom right,
//!   bottom left);
//! - `fill`, `stroke`, `clip`: the name, then the `Path2D` they take, if
//!   any, as `path(<operation>, ...)` (`fill path(moveTo 0 0, lineTo 10 0,
//!   closePath)`), then `evenodd` for that fill rule (`nonzero`, the
//!   default, is not written);
//! - `fillText` / `strokeText`: the text quoted (`"` and `\` escaped with
//!   `\`), its position, and its `maxWidth` when given (`fillText "Q1" 4 20`);
//! - `drawImage`: the image's URL (`asset:/<path>` for an imported asset),
//!   the part of it (in the image's pixels) when given, and where it goes
//!   (`drawImage asset:/logo.png 0 0 32 32`);
//! - a property: its name and its value (`lineWidth 2`, `lineCap round`,
//!   `globalCompositeOperation multiply`, `textBaseline middle`), the
//!   shadow's offset as one `shadowOffset <x> <y>`;
//! - `fillStyle` / `strokeStyle`: a color (`#rrggbbaa`), a gradient as the
//!   calls that made it (`createLinearGradient(0, 0, 200, 0)
//!   addColorStop(0, #ff0000ff) addColorStop(1, #0000ffff)`; radial and
//!   conic ones likewise), or `animated` for a color an animated value
//!   drives;
//! - `setLineDash`: the dashes and gaps separated by spaces, `none` for a
//!   solid line (as `strokeDasharray` reads);
//! - `font`: CSS's shorthand, its normal parts left out: `italic`, the
//!   variant names, the weight when not 400, the size in `px`, the family
//!   (`sans-serif` for the system font): `font 600 12px Inter`;
//!   `fontVariantNumeric`: the variant names, `normal` for none.
//!
//! A number an animated value drives is written as the value it was created
//! with (as values.rs writes animated values), `animated` when the run never
//! saw it created.

use nsh_core::{
    FontVariant,
    canvas::{
        CanvasGradient, CanvasOp, CanvasTextAlign, CanvasTextBaseline, Composite, Drawing,
        FillRule, GradientShape, LineCap, LineJoin, Num, Paint, PathOp,
    },
};

use super::values::{self, color, num};

/// The operations of `drawing`, in order, as the steps read them.
pub(crate) fn ops(drawing: &Drawing) -> Vec<String> {
    let w = Writer { drawing };
    drawing.ops.iter().map(|op| w.op(op)).collect()
}

/// Whether `op`, an operation as `ops` writes it, is what a step's `want`
/// names: the same text, or, for a `want` ending in `...`, text starting
/// with what comes before it.
pub(crate) fn matches(op: &str, want: &str) -> bool {
    match want.strip_suffix("...") {
        Some(start) => op.starts_with(start),
        None => op == want,
    }
}

struct Writer<'a> {
    drawing: &'a Drawing,
}

impl Writer<'_> {
    fn num(&self, n: Num) -> String {
        match n {
            Num::Const(v) => num(v),
            Num::Slot(slot) => self
                .drawing
                .slots
                .get(slot as usize)
                .and_then(|&node| values::animated_value(node))
                .map_or_else(|| "animated".into(), num),
        }
    }

    fn nums(&self, list: &[Num]) -> String {
        list.iter()
            .map(|&n| self.num(n))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn op(&self, op: &CanvasOp) -> String {
        match op {
            CanvasOp::Save => "save".into(),
            CanvasOp::Restore => "restore".into(),
            CanvasOp::Translate(x, y) => format!("translate {}", self.nums(&[*x, *y])),
            CanvasOp::Scale(x, y) => format!("scale {}", self.nums(&[*x, *y])),
            CanvasOp::Rotate(a) => format!("rotate {}", self.num(*a)),
            CanvasOp::Transform(m) => format!("transform {}", self.nums(m)),
            CanvasOp::SetTransform(m) => format!("setTransform {}", self.nums(m)),
            CanvasOp::BeginPath => "beginPath".into(),
            CanvasOp::Path(p) => self.path_op(p),
            CanvasOp::Fill(path, rule) => self.paint_path("fill", path.as_deref(), *rule),
            CanvasOp::Stroke(path) => self.paint_path("stroke", path.as_deref(), FillRule::NonZero),
            CanvasOp::Clip(path, rule) => self.paint_path("clip", path.as_deref(), *rule),
            CanvasOp::FillRect(r) => format!("fillRect {}", self.nums(r)),
            CanvasOp::StrokeRect(r) => format!("strokeRect {}", self.nums(r)),
            CanvasOp::ClearRect(r) => format!("clearRect {}", self.nums(r)),
            CanvasOp::FillStyle(p) => format!("fillStyle {}", paint(p)),
            CanvasOp::StrokeStyle(p) => format!("strokeStyle {}", paint(p)),
            CanvasOp::LineWidth(n) => format!("lineWidth {}", self.num(*n)),
            CanvasOp::LineCap(c) => format!(
                "lineCap {}",
                match c {
                    LineCap::Butt => "butt",
                    LineCap::Round => "round",
                    LineCap::Square => "square",
                }
            ),
            CanvasOp::LineJoin(j) => format!(
                "lineJoin {}",
                match j {
                    LineJoin::Miter => "miter",
                    LineJoin::Round => "round",
                    LineJoin::Bevel => "bevel",
                }
            ),
            CanvasOp::MiterLimit(v) => format!("miterLimit {}", num(*v)),
            CanvasOp::LineDash(dashes) => format!(
                "setLineDash {}",
                if dashes.is_empty() {
                    "none".into()
                } else {
                    dashes.iter().map(|d| num(*d)).collect::<Vec<_>>().join(" ")
                }
            ),
            CanvasOp::LineDashOffset(n) => format!("lineDashOffset {}", self.num(*n)),
            CanvasOp::GlobalAlpha(n) => format!("globalAlpha {}", self.num(*n)),
            CanvasOp::Composite(c) => format!("globalCompositeOperation {}", composite(*c)),
            CanvasOp::ShadowColor(c) => format!("shadowColor {}", color(*c)),
            CanvasOp::ShadowBlur(v) => format!("shadowBlur {}", num(*v)),
            CanvasOp::ShadowOffset(p) => format!("shadowOffset {} {}", num(p.x), num(p.y)),
            CanvasOp::Font(f) => {
                let mut words = Vec::new();
                if f.italic {
                    words.push("italic".to_owned());
                }
                words.extend(variants(f.variant));
                if f.weight != 400 {
                    words.push(f.weight.to_string());
                }
                words.push(format!("{}px", num(f.size)));
                words.push(f.family.as_deref().unwrap_or("sans-serif").to_owned());
                format!("font {}", words.join(" "))
            }
            CanvasOp::FontVariantNumeric(v) => {
                let names = variants(*v);
                format!(
                    "fontVariantNumeric {}",
                    if names.is_empty() {
                        "normal".into()
                    } else {
                        names.join(" ")
                    }
                )
            }
            CanvasOp::TextAlign(a) => format!(
                "textAlign {}",
                match a {
                    CanvasTextAlign::Start => "start",
                    CanvasTextAlign::End => "end",
                    CanvasTextAlign::Left => "left",
                    CanvasTextAlign::Right => "right",
                    CanvasTextAlign::Center => "center",
                }
            ),
            CanvasOp::TextBaseline(b) => format!(
                "textBaseline {}",
                match b {
                    CanvasTextBaseline::Alphabetic => "alphabetic",
                    CanvasTextBaseline::Top => "top",
                    CanvasTextBaseline::Hanging => "hanging",
                    CanvasTextBaseline::Middle => "middle",
                    CanvasTextBaseline::Ideographic => "ideographic",
                    CanvasTextBaseline::Bottom => "bottom",
                }
            ),
            CanvasOp::FillText {
                text,
                x,
                y,
                max_width,
            } => self.text("fillText", text, *x, *y, *max_width),
            CanvasOp::StrokeText {
                text,
                x,
                y,
                max_width,
            } => self.text("strokeText", text, *x, *y, *max_width),
            CanvasOp::DrawImage {
                image,
                source,
                dest,
            } => {
                let mut out = format!("drawImage {}", image.uri);
                if let Some(s) = source {
                    for v in [s.origin.x, s.origin.y, s.size.width, s.size.height] {
                        out += &format!(" {}", num(v.0));
                    }
                }
                format!("{out} {}", self.nums(dest))
            }
        }
    }

    fn path_op(&self, op: &PathOp) -> String {
        let ccw = |ccw: bool| if ccw { " true" } else { "" };
        match op {
            PathOp::MoveTo(x, y) => format!("moveTo {}", self.nums(&[*x, *y])),
            PathOp::LineTo(x, y) => format!("lineTo {}", self.nums(&[*x, *y])),
            PathOp::QuadTo(a) => format!("quadraticCurveTo {}", self.nums(a)),
            PathOp::CubicTo(a) => format!("bezierCurveTo {}", self.nums(a)),
            PathOp::Arc {
                x,
                y,
                radius,
                start,
                end,
                ccw: c,
            } => format!(
                "arc {}{}",
                self.nums(&[*x, *y, *radius, *start, *end]),
                ccw(*c)
            ),
            PathOp::ArcTo(a) => format!("arcTo {}", self.nums(a)),
            PathOp::Ellipse {
                geometry,
                start,
                end,
                ccw: c,
            } => {
                let mut all = geometry.to_vec();
                all.extend([*start, *end]);
                format!("ellipse {}{}", self.nums(&all), ccw(*c))
            }
            PathOp::Rect(r) => format!("rect {}", self.nums(r)),
            PathOp::RoundRect(r, radii) => {
                format!("roundRect {} {}", self.nums(r), self.nums(radii))
            }
            PathOp::Close => "closePath".into(),
        }
    }

    fn paint_path(&self, name: &str, path: Option<&[PathOp]>, rule: FillRule) -> String {
        let mut out = name.to_owned();
        if let Some(path) = path {
            let ops: Vec<String> = path.iter().map(|op| self.path_op(op)).collect();
            out += &format!(" path({})", ops.join(", "));
        }
        if rule == FillRule::EvenOdd {
            out += " evenodd";
        }
        out
    }

    fn text(
        &self,
        name: &str,
        text: &str,
        x: Num,
        y: Num,
        max_width: Option<nsh_core::Points>,
    ) -> String {
        let quoted = text.replace('\\', "\\\\").replace('"', "\\\"");
        let mut out = format!("{name} \"{quoted}\" {}", self.nums(&[x, y]));
        if let Some(w) = max_width {
            out += &format!(" {}", num(w));
        }
        out
    }
}

/// `fillStyle` / `strokeStyle`'s value.
fn paint(p: &Paint) -> String {
    match p {
        Paint::Color(c) => color(*c),
        Paint::Slot(_) => "animated".into(),
        Paint::Gradient(g) => gradient(g),
    }
}

/// A gradient as the calls that made it.
fn gradient(g: &CanvasGradient) -> String {
    let args = |list: &[f32]| list.iter().map(|v| num(*v)).collect::<Vec<_>>().join(", ");
    let mut out = match g.shape {
        GradientShape::Linear { start, end } => format!(
            "createLinearGradient({})",
            args(&[start.x.0, start.y.0, end.x.0, end.y.0])
        ),
        GradientShape::Radial {
            start,
            start_radius,
            end,
            end_radius,
        } => format!(
            "createRadialGradient({})",
            args(&[
                start.x.0,
                start.y.0,
                start_radius.0,
                end.x.0,
                end.y.0,
                end_radius.0
            ])
        ),
        GradientShape::Conic {
            start_angle,
            center,
        } => format!(
            "createConicGradient({})",
            args(&[start_angle.0, center.x.0, center.y.0])
        ),
    };
    for (offset, c) in &g.stops {
        out += &format!(" addColorStop({}, {})", num(offset.0), color(*c));
    }
    out
}

/// The names of the variants set in `v`, in CSS's order.
fn variants(v: FontVariant) -> Vec<String> {
    [
        (FontVariant::SMALL_CAPS, "small-caps"),
        (FontVariant::OLDSTYLE_NUMS, "oldstyle-nums"),
        (FontVariant::LINING_NUMS, "lining-nums"),
        (FontVariant::TABULAR_NUMS, "tabular-nums"),
        (FontVariant::PROPORTIONAL_NUMS, "proportional-nums"),
    ]
    .into_iter()
    .filter(|(bit, _)| v.0 & bit != 0)
    .map(|(_, name)| name.to_owned())
    .collect()
}

/// `globalCompositeOperation`'s name.
fn composite(c: Composite) -> &'static str {
    use Composite as C;
    match c {
        C::SourceOver => "source-over",
        C::SourceIn => "source-in",
        C::SourceOut => "source-out",
        C::SourceAtop => "source-atop",
        C::DestinationOver => "destination-over",
        C::DestinationIn => "destination-in",
        C::DestinationOut => "destination-out",
        C::DestinationAtop => "destination-atop",
        C::Lighter => "lighter",
        C::Copy => "copy",
        C::Xor => "xor",
        C::Multiply => "multiply",
        C::Screen => "screen",
        C::Overlay => "overlay",
        C::Darken => "darken",
        C::Lighten => "lighten",
        C::ColorDodge => "color-dodge",
        C::ColorBurn => "color-burn",
        C::HardLight => "hard-light",
        C::SoftLight => "soft-light",
        C::Difference => "difference",
        C::Exclusion => "exclusion",
        C::Hue => "hue",
        C::Saturation => "saturation",
        C::Color => "color",
        C::Luminosity => "luminosity",
    }
}

/// The forms of operations are what the scripts hold: they never change.
#[test]
fn operations_are_written_as_apps_write_them() {
    use std::sync::Arc;

    use nsh_core::{
        Color,
        canvas::{CanvasFont, CanvasImage},
        geometry::{Percentage, point, px},
    };

    let path: nsh_core::canvas::Path = Arc::from(vec![
        PathOp::MoveTo(Num::Const(0.0), Num::Const(0.0)),
        PathOp::LineTo(Num::Const(10.0), Num::Const(0.5)),
        PathOp::Close,
    ]);
    let drawing = Drawing {
        ops: vec![
            CanvasOp::Save,
            CanvasOp::FillStyle(Paint::Color(Color(0xff0000ff))),
            CanvasOp::StrokeStyle(Paint::Gradient(Arc::new(
                CanvasGradient::linear(point(px(0.0), px(0.0)), point(px(200.0), px(0.0)))
                    .add_color_stop(Percentage(0.0), Color(0xff0000ff))
                    .add_color_stop(Percentage(1.0), Color(0x0000ffff)),
            ))),
            CanvasOp::LineWidth(Num::Const(2.0)),
            CanvasOp::LineDash(Arc::from([px(4.0), px(2.0)])),
            CanvasOp::LineDash(Arc::from([])),
            CanvasOp::BeginPath,
            CanvasOp::Path(PathOp::Arc {
                x: Num::Const(5.0),
                y: Num::Const(5.0),
                radius: Num::Const(4.0),
                start: Num::Const(0.0),
                end: Num::Const(std::f32::consts::PI),
                ccw: true,
            }),
            CanvasOp::Fill(Some(path.clone()), FillRule::EvenOdd),
            CanvasOp::Stroke(None),
            CanvasOp::Font(CanvasFont {
                size: px(12.0),
                weight: 600,
                family: Some("Inter".into()),
                ..CanvasFont::default()
            }),
            CanvasOp::Font(CanvasFont::default()),
            CanvasOp::FontVariantNumeric(FontVariant(FontVariant::TABULAR_NUMS)),
            CanvasOp::FillText {
                text: "Say \"hi\"".into(),
                x: Num::Const(4.0),
                y: Num::Const(20.0),
                max_width: None,
            },
            CanvasOp::DrawImage {
                image: CanvasImage {
                    uri: "asset:/logo.png".into(),
                },
                source: None,
                dest: [
                    Num::Const(0.0),
                    Num::Const(0.0),
                    Num::Const(32.0),
                    Num::Const(32.0),
                ],
            },
            CanvasOp::Composite(Composite::Multiply),
            CanvasOp::LineWidth(Num::Slot(0)),
            CanvasOp::Restore,
        ],
        ..Drawing::default()
    };
    assert_eq!(
        ops(&drawing),
        [
            "save",
            "fillStyle #ff0000ff",
            "strokeStyle createLinearGradient(0, 0, 200, 0) addColorStop(0, #ff0000ff) \
             addColorStop(1, #0000ffff)",
            "lineWidth 2",
            "setLineDash 4 2",
            "setLineDash none",
            "beginPath",
            "arc 5 5 4 0 3.142 true",
            "fill path(moveTo 0 0, lineTo 10 0.5, closePath) evenodd",
            "stroke",
            "font 600 12px Inter",
            "font 10px sans-serif",
            "fontVariantNumeric tabular-nums",
            "fillText \"Say \\\"hi\\\"\" 4 20",
            "drawImage asset:/logo.png 0 0 32 32",
            "globalCompositeOperation multiply",
            "lineWidth animated",
            "restore",
        ]
    );
    assert!(matches(
        "fill path(moveTo 0 0, lineTo 10 0.5, closePath)",
        "fill path(..."
    ));
    assert!(!matches("fill", "fill path(..."));
    assert!(matches("stroke", "stroke"));
    assert!(!matches("stroke path(moveTo 0 0)", "stroke"));
}
