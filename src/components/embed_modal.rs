use dioxus::prelude::*;

#[component]
pub fn EmbedModal(code: String, origin: Option<String>, on_close: EventHandler<()>) -> Element {
    let mut resolved_origin = use_signal(|| origin.unwrap_or_default());
    let mut copied = use_signal(|| false);

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
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "modal-content",
                onclick: move |e| e.stop_propagation(),
                div {
                    class: "modal-header",
                    h3 { "Incorpora come Iframe" }
                    button {
                        class: "modal-close",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
                p {
                    style: "font-size: 0.9rem; color: var(--text-muted);",
                    "Copia questo snippet per includere la nuvola in tempo reale su qualsiasi sito web o dominio esterno (Notion, WordPress, presentazioni, dashboard aziendali)."
                }
                div {
                    style: "margin: 0.75rem 0 0.25rem; font-size: 0.8rem; color: #a5b4fc; display: flex; align-items: center; gap: 0.4rem;",
                    span { "🌐" }
                    span { "URL Assoluto per domini esterni: " }
                    code {
                        style: "background: rgba(255,255,255,0.08); padding: 0.15rem 0.4rem; border-radius: 4px;",
                        "{full_url}"
                    }
                }
                div {
                    class: "code-snippet",
                    "{embed_snippet}"
                }
                div {
                    style: "font-size: 0.78rem; color: #6ee7b7; margin-bottom: 1.25rem; display: flex; align-items: center; gap: 0.35rem;",
                    span { "✓" }
                    span { "Compatibile Cross-Origin: accessibile liberamente da qualsiasi dominio esterno senza blocchi CORS o X-Frame-Options." }
                }
                div {
                    style: "display: flex; justify-content: flex-end; gap: 0.75rem;",
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: move |_| on_close.call(()),
                        "Chiudi"
                    }
                    button {
                        class: "btn btn-primary btn-sm",
                        onclick: {
                            let text = snippet_for_copy.clone();
                            move |_| {
                                let js = format!(r#"navigator.clipboard.writeText({:?});"#, text);
                                let _ = document::eval(&js);
                                copied.set(true);
                            }
                        },
                        if copied() { "✅ Copiato!" } else { "📋 Copia Codice Iframe" }
                    }
                }
            }
        }
    }
}
