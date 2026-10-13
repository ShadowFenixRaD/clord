use dioxus::prelude::*;
use shadow_design::alert::{Alert, AlertVariant};
use shadow_design::button::{Button, ButtonSize, ButtonVariant};
use shadow_design::icon::{Icon, IconKind};
use shadow_design::modal::Modal;
use shadow_design::notification::use_notification;
use shadow_design::typography::{Code, Text, TextSize};

#[component]
pub fn EmbedModal(
    code: String,
    origin: Option<String>,
    is_open: Signal<bool>,
) -> Element {
    let mut resolved_origin = use_signal(|| origin.unwrap_or_default());
    let mut notifications = use_notification();

    // If origin is empty, query window.location.origin
    use_effect(move || {
        if resolved_origin().is_empty() {
            spawn(async move {
                let mut eval = document::eval("dioxus.send(window.location.origin);");
                if let Ok(orig) = eval.recv::<String>().await {
                    resolved_origin.set(orig);
                }
            });
        }
    });

    let current_orig = resolved_origin();
    let full_url = if current_orig.is_empty() {
        format!("/wc/{}", code)
    } else {
        format!("{}/wc/{}", current_orig.trim_end_matches('/'), code)
    };

    let embed_snippet = format!(
        r#"<iframe src="{}" width="100%" height="650" style="border:none;border-radius:12px;width:100%;min-height:500px;" allow="clipboard-write" allowfullscreen loading="lazy"></iframe>"#,
        full_url
    );

    let snippet_for_copy = embed_snippet.clone();

    rsx! {
        Modal {
            is_open,
            title: Some("Incorpora come Iframe".to_string()),
            glow: Some(true),
            div {
                style: "display: flex; flex-direction: column; gap: 1rem; padding: 0.5rem 0;",
                Text {
                    size: TextSize::Sm,
                    dimmed: true,
                    "Copia questo snippet per includere la nuvola in tempo reale su qualsiasi sito web o dominio esterno (Notion, WordPress, presentazioni, dashboard aziendali)."
                }

                div {
                    style: "display: flex; flex-direction: column; gap: 0.35rem; font-size: 0.82rem; color: #a5b4fc;",
                    span { "🌐 URL Assoluto per domini esterni:" }
                    Code { "{full_url}" }
                }

                div {
                    style: "margin: 0.25rem 0;",
                    Code { "{embed_snippet}" }
                }

                Alert {
                    variant: AlertVariant::Success,
                    title: Some("Cross-Origin Ready".to_string()),
                    glow: Some(true),
                    "Compatibile Cross-Origin: accessibile liberamente da qualsiasi dominio esterno senza blocchi CORS o X-Frame-Options."
                }

                div {
                    style: "display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 0.5rem;",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            let mut s = is_open;
                            s.set(false);
                        },
                        "Chiudi"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Sm,
                        glow: Some(true),
                        left_icon: Some(rsx! { Icon { kind: IconKind::Copy, size: 14 } }),
                        onclick: {
                            let text = snippet_for_copy.clone();
                            move |_| {
                                let js = format!(r#"navigator.clipboard.writeText({:?});"#, text);
                                let _ = document::eval(&js);
                                notifications.success("Codice iframe copiato negli appunti!");
                            }
                        },
                        "Copia Codice Iframe"
                    }
                }
            }
        }
    }
}
