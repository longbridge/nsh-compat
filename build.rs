use nsh_core::{Prop, Style, StyleValue, element::TsTypes, js::host::HostProps};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_macos_backdrop)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_windows_backdrop)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_glass_tint)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_accent_color)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_window_controls)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_web_element)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_accessibility_description)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_scrollbar_inset)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_header_shadow_visible)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_header_background_on_scroll)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_header_title_align)");
    println!("cargo:rustc-check-cfg=cfg(nsh_has_select_text_on_focus)");

    // The checker must still compile against NSH checkouts predating these
    // fields, so detect the public API before referring to their Rust fields.
    let mut specs = Vec::new();
    Prop::specs(&mut TsTypes::default(), &mut specs);
    if specs
        .iter()
        .any(|spec| spec.name == "backdrop" && format!("{:?}", spec.platform) == "Macos")
    {
        println!("cargo:rustc-cfg=nsh_has_macos_backdrop");
    }
    if specs
        .iter()
        .any(|spec| spec.name == "backdrop" && format!("{:?}", spec.platform) == "Windows")
        && nsh_core::js::host::declarations().contains("export interface WindowsBackdrop {")
    {
        println!("cargo:rustc-cfg=nsh_has_windows_backdrop");
    }
    if specs.iter().any(|spec| spec.name == "windowControl") {
        println!("cargo:rustc-cfg=nsh_has_window_controls");
    }
    // By the platform's name: older checkouts have no `Platform::Web`.
    if specs
        .iter()
        .any(|spec| spec.name == "element" && format!("{:?}", spec.platform) == "Web")
    {
        println!("cargo:rustc-cfg=nsh_has_web_element");
    }
    if specs
        .iter()
        .any(|spec| spec.name == "accessibilityDescription")
    {
        println!("cargo:rustc-cfg=nsh_has_accessibility_description");
    }
    if specs.iter().any(|spec| spec.name == "scrollbarInset") {
        println!("cargo:rustc-cfg=nsh_has_scrollbar_inset");
    }
    if specs.iter().any(|spec| spec.name == "headerShadowVisible") {
        println!("cargo:rustc-cfg=nsh_has_header_shadow_visible");
    }
    if specs
        .iter()
        .any(|spec| spec.name == "headerBackgroundOnScroll")
    {
        println!("cargo:rustc-cfg=nsh_has_header_background_on_scroll");
    }
    if specs.iter().any(|spec| spec.name == "headerTitleAlign") {
        println!("cargo:rustc-cfg=nsh_has_header_title_align");
    }
    if specs.iter().any(|spec| spec.name == "selectTextOnFocus") {
        println!("cargo:rustc-cfg=nsh_has_select_text_on_focus");
    }
    if Style::default()
        .set("glassTintColor", StyleValue::Str("#ff0000"))
        .is_ok()
    {
        println!("cargo:rustc-cfg=nsh_has_glass_tint");
    }
    if Style::default()
        .set("accentColor", StyleValue::Str("#ff0000"))
        .is_ok()
    {
        println!("cargo:rustc-cfg=nsh_has_accent_color");
    }
}
