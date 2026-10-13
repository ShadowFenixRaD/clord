use dioxus::prelude::*;

mod backend;
mod components;
mod model;

use components::{EmbedModal, QrCodeView, WordCloud};
use model::WordCloudSession;

const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/wc/:code")]
    WordCloudView { code: String },
    #[route("/wc/:code/send")]
    SendWordView { code: String },
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Clord - Nuvola di Parole Interattiva" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

/// Home Page
#[component]
fn Home() -> Element {
    let mut input_code = use_signal(String::new);
    let mut is_creating = use_signal(|| false);
    let mut err_msg = use_signal(|| None::<String>);
    let nav = navigator();

    rsx! {
        div {
            class: "app-container",
            header {
                class: "header-nav",
                Link {
                    to: Route::Home {},
                    class: "brand-link",
                    span { class: "brand-icon", "☁️" }
                    span { "Clord" }
                    span { class: "brand-badge", "Dioxus 0.7" }
                }
            }

            main {
                class: "home-container",
                div {
                    class: "home-hero",
                    h1 {
                        "Nuvole di parole in tempo reale, "
                        span { class: "text-gradient", "semplici e aperte." }
                    }
                    p {
                        "L'alternativa leggera a Mentimeter senza registrazione. "
                        "Crea una sessione con codice numerico da 1 a 8 cifre, condividi il QR code e raccogli risposte dal vivo."
                    }
                    div {
                        class: "hero-actions",
                        button {
                            class: "btn btn-primary btn-pill",
                            style: "font-size: 1.1rem; padding: 0.85rem 2rem;",
                            disabled: is_creating(),
                            onclick: move |_| {
                                is_creating.set(true);
                                spawn(async move {
                                    match backend::create_new_cloud().await {
                                        Ok(new_code) => {
                                            nav.push(Route::WordCloudView { code: new_code });
                                        }
                                        Err(err) => {
                                            err_msg.set(Some(format!("Errore creazione: {}", err)));
                                            is_creating.set(false);
                                        }
                                    }
                                });
                            },
                            if is_creating() { "⏳ Creazione in corso..." } else { "🚀 Crea Nuova Nuvola" }
                        }
                    }

                    if let Some(err) = err_msg() {
                        div {
                            style: "color: #f87171; margin-top: 1rem; font-weight: 600;",
                            "{err}"
                        }
                    }
                }

                div {
                    class: "join-card",
                    h3 { "Partecipa o visualizza una sessione" }
                    p { "Hai già un codice da 1 a 8 cifre? Inseriscilo qui sotto." }
                    div {
                        class: "join-form",
                        div {
                            class: "code-input-group",
                            input {
                                class: "input-code",
                                placeholder: "Es. 1024",
                                value: "{input_code}",
                                maxlength: 8,
                                oninput: move |e| input_code.set(e.value().trim().to_string()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter && !input_code().is_empty() {
                                        nav.push(Route::WordCloudView { code: input_code() });
                                    }
                                }
                            }
                        }
                        div {
                            style: "display: flex; gap: 0.75rem; margin-top: 0.25rem;",
                            button {
                                class: "btn btn-primary",
                                style: "flex: 1;",
                                disabled: input_code().is_empty(),
                                onclick: move |_| {
                                    if !input_code().is_empty() {
                                        nav.push(Route::WordCloudView { code: input_code() });
                                    }
                                },
                                "📊 Mostra Nuvola"
                            }
                            button {
                                class: "btn btn-secondary",
                                style: "flex: 1;",
                                disabled: input_code().is_empty(),
                                onclick: move |_| {
                                    if !input_code().is_empty() {
                                        nav.push(Route::SendWordView { code: input_code() });
                                    }
                                },
                                "✏️ Invia Parole"
                            }
                        }
                    }
                }

                div {
                    class: "feature-grid",
                    div {
                        class: "feature-box",
                        div { class: "feature-icon", "↔️" }
                        h4 { "Orizzontale & Verticale" }
                        p { "Le parole vengono posizionate armonicamente in entrambe le direzioni senza fastidiosi scatti." }
                    }
                    div {
                        class: "feature-box",
                        div { class: "feature-icon", "📱" }
                        h4 { "QR Code Istantaneo" }
                        p { "I partecipanti inquadrano il codice QR e inviano le loro risposte all'istante dallo smartphone." }
                    }
                    div {
                        class: "feature-box",
                        div { class: "feature-icon", "🖼️" }
                        h4 { "Pronto per Iframe" }
                        p { "Includi facilmente la nuvola in qualsiasi sito esterno, slide interattiva o Notion con 1 riga." }
                    }
                    div {
                        class: "feature-box",
                        div { class: "feature-icon", "🔄" }
                        h4 { "Rotazione & Reset Rapido" }
                        p { "Salvataggio locale con riciclo automatico delle sessioni più vecchie e reset immediato." }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    CloudOnly,
    Split,
    Welcome,
}

/// Word Cloud Presentation View (/wc/:code)
#[component]
fn WordCloudView(code: String) -> Element {
    let mut cloud_data = use_signal(|| None::<WordCloudSession>);
    let mut send_url = use_signal(|| format!("/wc/{}/send", code));
    let mut view_mode = use_signal(|| ViewMode::Split);
    let mut hide_code = use_signal(|| false);
    let mut show_embed = use_signal(|| false);
    let mut copied_link = use_signal(|| false);

    // Initial load and periodic polling
    {
        let code = code.clone();
        use_effect(move || {
            let c = code.clone();
            spawn(async move {
                // Ensure session exists
                let _ = backend::get_or_create_cloud(c.clone()).await;
                loop {
                    if let Ok(Some(data)) = backend::get_cloud(c.clone()).await {
                        cloud_data.set(Some(data));
                    }
                    model::sleep_ms(1500).await;
                }
            });
        });
    }

    let mut site_origin = use_signal(String::new);

    // Determine window.location.origin on client for absolute URLs
    {
        let code = code.clone();
        use_effect(move || {
            let c = code.clone();
            spawn(async move {
                let mut eval = document::eval("dioxus.send(window.location.origin);");
                if let Ok(origin) = eval.recv::<String>().await {
                    site_origin.set(origin.clone());
                    send_url.set(format!("{}/wc/{}/send", origin, c));
                }
            });
        });
    }

    let default_session = WordCloudSession::new(code.clone(), 0);
    let current_session = cloud_data().unwrap_or(default_session);
    let total_words = current_session.total_words();
    let unique_words = current_session.unique_words();

    rsx! {
        div {
            class: "wc-screen",
            // Header / Controls Toolbar
            div {
                class: "wc-toolbar",
                div {
                    style: "display: flex; align-items: center; gap: 1rem;",
                    Link {
                        to: Route::Home {},
                        class: "brand-link",
                        style: "font-size: 1.15rem;",
                        span { class: "brand-icon", "☁️" }
                        span { "Clord" }
                    }
                    // 3 Views Selector
                    div {
                        class: "view-selector",
                        button {
                            class: if view_mode() == ViewMode::CloudOnly { "view-btn active" } else { "view-btn" },
                            onclick: move |_| view_mode.set(ViewMode::CloudOnly),
                            "☁️ Nuvola"
                        }
                        button {
                            class: if view_mode() == ViewMode::Split { "view-btn active" } else { "view-btn" },
                            onclick: move |_| view_mode.set(ViewMode::Split),
                            "📊 Nuvola + QR"
                        }
                        button {
                            class: if view_mode() == ViewMode::Welcome { "view-btn active" } else { "view-btn" },
                            onclick: move |_| view_mode.set(ViewMode::Welcome),
                            "📱 Codice + QR"
                        }
                    }
                }

                div {
                    class: "toolbar-actions",
                    // Hide/Show code button only in CloudOnly mode
                    if view_mode() == ViewMode::CloudOnly {
                        button {
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| hide_code.set(!hide_code()),
                            if hide_code() { "👁️ Mostra Codice" } else { "🙈 Nascondi Codice" }
                        }
                    }

                    // Copy participant link
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: {
                            let url = send_url();
                            move |_| {
                                let js = format!(r#"navigator.clipboard.writeText({:?});"#, url);
                                let _ = document::eval(&js);
                                copied_link.set(true);
                            }
                        },
                        if copied_link() { "✅ Copiato!" } else { "🔗 Copia Link" }
                    }

                    // Embed iframe button
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: move |_| show_embed.set(true),
                        "🖼️ Iframe"
                    }

                    // Export SVG
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: {
                            let svg_data = current_session.generate_svg(1600, 1000);
                            let code = code.clone();
                            move |_| {
                                let filename = format!("clord_{}.svg", code);
                                let js = format!(
                                    r#"const blob = new Blob([{:?}], {{ type: 'image/svg+xml;charset=utf-8' }});
                                    const url = URL.createObjectURL(blob);
                                    const a = document.createElement('a');
                                    a.href = url;
                                    a.download = {:?};
                                    a.click();
                                    URL.revokeObjectURL(url);"#,
                                    svg_data, filename
                                );
                                let _ = document::eval(&js);
                            }
                        },
                        "🎨 SVG"
                    }

                    // Export PNG
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: {
                            let svg_data = current_session.generate_svg(1600, 1000);
                            let code = code.clone();
                            move |_| {
                                let filename = format!("clord_{}.png", code);
                                let js = format!(
                                    r#"const svgStr = {:?};
                                    const filename = {:?};
                                    const img = new Image();
                                    img.onload = () => {{
                                        const canvas = document.createElement('canvas');
                                        canvas.width = 1600;
                                        canvas.height = 1000;
                                        const ctx = canvas.getContext('2d');
                                        ctx.drawImage(img, 0, 0);
                                        const a = document.createElement('a');
                                        a.href = canvas.toDataURL('image/png');
                                        a.download = filename;
                                        a.click();
                                    }};
                                    img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgStr);"#,
                                    svg_data, filename
                                );
                                let _ = document::eval(&js);
                            }
                        },
                        "📸 PNG"
                    }
                }
            }

            // Floating code banner if in Solo Nuvola mode and code is NOT hidden
            if view_mode() == ViewMode::CloudOnly && !hide_code() {
                div {
                    class: "floating-code-banner",
                    span { "Codice di accesso:" }
                    span { class: "code-highlight", "#{code}" }
                    button {
                        class: "btn btn-secondary btn-sm btn-pill",
                        style: "padding: 0.2rem 0.55rem; font-size: 0.75rem;",
                        onclick: move |_| hide_code.set(true),
                        "Nascondi"
                    }
                }
            }

            // Main Content Area based on View Mode
            div {
                class: "wc-layout",
                match view_mode() {
                    ViewMode::CloudOnly => rsx! {
                        div {
                            class: "wc-cloud-area",
                            WordCloud {
                                session: current_session.clone(),
                                on_delete_word: {
                                    let c = code.clone();
                                    move |word: String| {
                                        let c = c.clone();
                                        cloud_data.with_mut(|opt| {
                                            if let Some(sess) = opt {
                                                sess.words.remove(&word.to_lowercase());
                                                sess.words.remove(&word);
                                            }
                                        });
                                        spawn(async move {
                                            let _ = backend::delete_word(c, word).await;
                                        });
                                    }
                                }
                            }
                        }
                    },
                    ViewMode::Split => rsx! {
                        div {
                            class: "wc-cloud-area",
                            WordCloud {
                                session: current_session.clone(),
                                on_delete_word: {
                                    let c = code.clone();
                                    move |word: String| {
                                        let c = c.clone();
                                        cloud_data.with_mut(|opt| {
                                            if let Some(sess) = opt {
                                                sess.words.remove(&word.to_lowercase());
                                                sess.words.remove(&word);
                                            }
                                        });
                                        spawn(async move {
                                            let _ = backend::delete_word(c, word).await;
                                        });
                                    }
                                }
                            }
                        }
                        aside {
                            class: "wc-sidebar",
                            div {
                                class: "sidebar-card",
                                div { class: "sidebar-code-label", "Codice sessione" }
                                div { class: "sidebar-code-val", "#{code}" }
                                QrCodeView { url: send_url(), class: Some("qr-wrapper".to_string()) }
                                p {
                                    style: "font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.75rem;",
                                    "Inquadra con il telefono per inviare le tue parole"
                                }
                                a {
                                    class: "btn btn-primary btn-sm",
                                    style: "width: 100%;",
                                    href: "{send_url}",
                                    target: "_blank",
                                    "✏️ Apri pagina invio ↗"
                                }
                            }
                            div {
                                class: "sidebar-stats",
                                div {
                                    class: "stat-item",
                                    div { class: "stat-val", "{total_words}" }
                                    div { class: "stat-lbl", "Parole Totali" }
                                }
                                div {
                                    class: "stat-item",
                                    div { class: "stat-val", "{unique_words}" }
                                    div { class: "stat-lbl", "Parole Distinte" }
                                }
                            }
                        }
                    },
                    ViewMode::Welcome => rsx! {
                        div {
                            class: "welcome-screen",
                            div {
                                class: "welcome-card",
                                h2 {
                                    style: "font-size: 2rem; font-weight: 800;",
                                    "Unisciti alla sessione dal vivo"
                                }
                                p {
                                    style: "font-size: 1.15rem; color: var(--text-muted); margin-top: 0.25rem;",
                                    "Scansiona il codice QR o collegati inserendo il codice:"
                                }
                                div { class: "welcome-code-hero", "#{code}" }
                                QrCodeView { url: send_url(), class: Some("welcome-qr-wrapper".to_string()) }
                                button {
                                    class: "btn btn-primary btn-pill",
                                    style: "font-size: 1.1rem; padding: 0.75rem 2rem;",
                                    onclick: move |_| view_mode.set(ViewMode::Split),
                                    "🚀 Mostra Nuvola di Parole"
                                }
                            }
                        }
                    }
                }
            }

            // Embed Iframe Modal
            if show_embed() {
                EmbedModal {
                    code: code.clone(),
                    origin: Some(site_origin()),
                    on_close: move |_| show_embed.set(false)
                }
            }
        }
    }
}

/// Participant Word Submission View (/wc/:code/send)
#[component]
fn SendWordView(code: String) -> Element {
    let mut word_input = use_signal(String::new);
    let mut sent_words = use_signal(Vec::<String>::new);
    let mut feedback_msg = use_signal(|| None::<String>);
    let mut is_submitting = use_signal(|| false);

    rsx! {
        div {
            class: "send-container",
            header {
                class: "send-header",
                Link {
                    to: Route::Home {},
                    class: "brand-link",
                    style: "justify-content: center; margin-bottom: 0.5rem;",
                    span { class: "brand-icon", "☁️" }
                    span { "Clord" }
                }
                h2 { "Invia la tua parola" }
                div {
                    class: "send-code-pill",
                    "Sessione #{code}"
                }
            }

            div {
                class: "send-form-card",
                form {
                    onsubmit: {
                        let code = code.clone();
                        move |e| {
                            e.stop_propagation();
                            let w = word_input().trim().to_string();
                            if !w.is_empty() && !is_submitting() {
                                is_submitting.set(true);
                                let c = code.clone();
                                spawn(async move {
                                    match backend::submit_word(c, w.clone()).await {
                                        Ok(()) => {
                                            feedback_msg.set(Some(format!("✅ '{}' inviata con successo!", w)));
                                            sent_words.write().push(w);
                                            word_input.set(String::new());
                                        }
                                        Err(err) => {
                                            feedback_msg.set(Some(format!("⚠️ Errore: {}", err)));
                                        }
                                    }
                                    is_submitting.set(false);
                                });
                            }
                        }
                    },
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.85rem;",
                        input {
                            class: "input-word",
                            placeholder: "Scrivi qui una parola...",
                            value: "{word_input}",
                            maxlength: 35,
                            autofocus: true,
                            disabled: is_submitting(),
                            oninput: move |e| word_input.set(e.value()),
                        }
                        button {
                            class: "btn btn-primary",
                            r#type: "submit",
                            disabled: word_input().trim().is_empty() || is_submitting(),
                            style: "width: 100%; padding: 0.85rem; font-size: 1.05rem;",
                            if is_submitting() { "Invio in corso..." } else { "Invia Parola 🚀" }
                        }
                    }
                }

                if let Some(msg) = feedback_msg() {
                    div {
                        class: "toast-feedback",
                        "{msg}"
                    }
                }

                if !sent_words().is_empty() {
                    div {
                        class: "sent-history",
                        div { class: "sent-history-title", "Le tue parole inviate:" }
                        div {
                            class: "sent-chips",
                            for (i, word) in sent_words().iter().enumerate() {
                                span {
                                    key: "{i}-{word}",
                                    class: "sent-chip",
                                    "{word}"
                                }
                            }
                        }
                    }
                }

                div {
                    style: "margin-top: 1rem; text-align: center; border-top: 1px solid var(--border-color); padding-top: 1rem;",
                    Link {
                        to: Route::WordCloudView { code: code.clone() },
                        class: "btn btn-secondary btn-sm",
                        style: "width: 100%;",
                        "👀 Guarda la nuvola in diretta ↗"
                    }
                }
            }
        }
    }
}
