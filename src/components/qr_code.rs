use crate::model::generate_qr_svg;
use dioxus::prelude::*;

#[component]
pub fn QrCodeView(url: String, class: Option<String>) -> Element {
    let svg_data = generate_qr_svg(&url);
    rsx! {
        div {
            class: class.unwrap_or_else(|| "qr-wrapper".to_string()),
            dangerous_inner_html: "{svg_data}",
        }
    }
}
