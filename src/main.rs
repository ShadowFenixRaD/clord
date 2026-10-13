use dioxus::prelude::*;
use shadow_design::alert::{Alert, AlertVariant};
use shadow_design::badge::{Badge, BadgeVariant};
use shadow_design::button::{Button, ButtonSize, ButtonVariant};
use shadow_design::card::{Card, CardShadow, CardVariant};
use shadow_design::icon::{Icon, IconKind, SvgIcon};
use shadow_design::input::TextInput;
use shadow_design::navbar::Navbar;
use shadow_design::notification::{use_notification, NotificationProvider};
use shadow_design::tabs::{Tab, TabList};
use shadow_design::typography::{Heading, HeadingOrder, Text, TextSize};

mod backend;
mod components;
mod model;

use components::{EmbedModal, QrCodeView, WordCloud};
use model::WordCloudSession;

const CLOUD_CSS: Asset = asset!("/assets/cloud.css");

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
        document::Link { rel: "stylesheet", href: shadow_design::STYLES }
        document::Link { rel: "stylesheet", href: CLOUD_CSS }
        NotificationProvider {
            Router::<Route> {}
        }
    }
}

/// Home Page
#[component]
fn Home() -> Element {
    let input_code = use_signal(String::new);
    let mut is_creating = use_signal(|| false);
    let mut err_msg = use_signal(|| None::<String>);
    let nav = navigator();

    rsx! {
        div {
            class: "app-shell",
            Navbar {
                brand: rsx! {
                    Link {
                        to: Route::Home {},
                        style: "display: flex; align-items: center; gap: 0.65rem; text-decoration: none; color: inherit;",
                        Icon { kind: IconKind::Sparkles, size: 22 }
                        span { style: "font-weight: 700; font-size: 1.25rem;", "Clord" }
                        Badge {
                            label: "Dioxus 0.7".to_string(),
                            variant: BadgeVariant::Primary,
                            pill: true,
                        }
                    }
                },
                div {}
            }

            main {
                class: "home-wrapper",
                div {
                    class: "home-hero-content",
                    Heading {
                        order: HeadingOrder::H1,
                        "Nuvole di parole in tempo reale, semplici e aperte."
                    }
                    Text {
                        size: TextSize::Lg,
                        dimmed: true,
                        "L'alternativa leggera a Mentimeter senza registrazione. "
                        "Crea una sessione con codice numerico da 1 a 8 cifre, condividi il QR code e raccogli risposte dal vivo."
                    }
                    div {
                        style: "margin-top: 0.5rem;",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Lg,
                            glow: Some(true),
                            loading: is_creating(),
                            disabled: is_creating(),
                            left_icon: Some(rsx! { Icon { kind: IconKind::Sparkles, size: 18 } }),
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
                            if is_creating() { "Creazione in corso..." } else { "Crea Nuova Nuvola" }
                        }
                    }

                    if let Some(err) = err_msg() {
                        div {
                            style: "width: 100%; margin-top: 1rem;",
                            Alert {
                                variant: AlertVariant::Danger,
                                title: Some("Errore Creazione".to_string()),
                                glow: Some(true),
                                "{err}"
                            }
                        }
                    }
                }

                div {
                    style: "width: 100%; max-width: 480px;",
                    Card {
                        title: Some("Partecipa o visualizza una sessione".to_string()),
                        subtitle: Some("Hai già un codice da 1 a 8 cifre? Inseriscilo qui sotto.".to_string()),
                        variant: CardVariant::Elevated,
                        shadow: CardShadow::Lg,
                        glow: Some(true),
                        div {
                            style: "display: flex; flex-direction: column; gap: 1rem;",
                            div {
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter && !input_code().is_empty() {
                                        nav.push(Route::WordCloudView { code: input_code() });
                                    }
                                },
                                TextInput {
                                    value: input_code,
                                    placeholder: Some("Es. 1024".to_string()),
                                    left_icon: Some(SvgIcon::Kind(IconKind::Search)),
                                    clearable: true,
                                }
                            }
                            div {
                                style: "display: flex; gap: 0.75rem;",
                                Button {
                                    variant: ButtonVariant::Primary,
                                    full_width: true,
                                    disabled: input_code().is_empty(),
                                    left_icon: Some(rsx! { Icon { kind: IconKind::Layers, size: 16 } }),
                                    onclick: move |_| {
                                        if !input_code().is_empty() {
                                            nav.push(Route::WordCloudView { code: input_code() });
                                        }
                                    },
                                    "Mostra Nuvola"
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    full_width: true,
                                    disabled: input_code().is_empty(),
                                    left_icon: Some(rsx! { Icon { kind: IconKind::Terminal, size: 16 } }),
                                    onclick: move |_| {
                                        if !input_code().is_empty() {
                                            nav.push(Route::SendWordView { code: input_code() });
                                        }
                                    },
                                    "Invia Parole"
                                }
                            }
                        }
                    }
                }

                div {
                    class: "feature-grid-layout",
                    Card {
                        variant: CardVariant::Interactive,
                        shadow: CardShadow::Sm,
                        hover_glow: true,
                        title: Some("Orizzontale & Verticale".to_string()),
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            div { Icon { kind: IconKind::Box, size: 24 } }
                            Text {
                                size: TextSize::Sm,
                                dimmed: true,
                                "Le parole vengono posizionate armonicamente in entrambe le direzioni senza fastidiosi scatti."
                            }
                        }
                    }
                    Card {
                        variant: CardVariant::Interactive,
                        shadow: CardShadow::Sm,
                        hover_glow: true,
                        title: Some("QR Code Istantaneo".to_string()),
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            div { Icon { kind: IconKind::Sparkles, size: 24 } }
                            Text {
                                size: TextSize::Sm,
                                dimmed: true,
                                "I partecipanti inquadrano il codice QR e inviano le loro risposte all'istante dallo smartphone."
                            }
                        }
                    }
                    Card {
                        variant: CardVariant::Interactive,
                        shadow: CardShadow::Sm,
                        hover_glow: true,
                        title: Some("Pronto per Iframe".to_string()),
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            div { Icon { kind: IconKind::Code, size: 24 } }
                            Text {
                                size: TextSize::Sm,
                                dimmed: true,
                                "Includi facilmente la nuvola in qualsiasi sito esterno, slide interattiva o Notion con 1 riga."
                            }
                        }
                    }
                    Card {
                        variant: CardVariant::Interactive,
                        shadow: CardShadow::Sm,
                        hover_glow: true,
                        title: Some("Rotazione & Reset Rapido".to_string()),
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            div { Icon { kind: IconKind::Settings, size: 24 } }
                            Text {
                                size: TextSize::Sm,
                                dimmed: true,
                                "Salvataggio locale con riciclo automatico delle sessioni più vecchie e reset immediato."
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Word Cloud Presentation View (/wc/:code)
#[component]
fn WordCloudView(code: String) -> Element {
    let mut cloud_data = use_signal(|| None::<WordCloudSession>);
    let mut send_url = use_signal(|| format!("/wc/{}/send", code));
    let mut active_tab = use_signal(|| "split".to_string());
    let mut hide_code = use_signal(|| false);
    let mut show_embed = use_signal(|| false);
    let mut notifications = use_notification();

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
            Navbar {
                brand: rsx! {
                    div {
                        style: "display: flex; align-items: center; gap: 1rem;",
                        Link {
                            to: Route::Home {},
                            style: "display: flex; align-items: center; gap: 0.5rem; text-decoration: none; color: inherit;",
                            Icon { kind: IconKind::Sparkles, size: 20 }
                            span { style: "font-weight: 700; font-size: 1.15rem;", "Clord" }
                        }
                        TabList {
                            Tab {
                                value: "cloud".to_string(),
                                active: active_tab,
                                icon: Some(IconKind::Sparkles),
                                "Nuvola"
                            }
                            Tab {
                                value: "split".to_string(),
                                active: active_tab,
                                icon: Some(IconKind::Layers),
                                "Nuvola + QR"
                            }
                            Tab {
                                value: "welcome".to_string(),
                                active: active_tab,
                                icon: Some(IconKind::Box),
                                "Codice + QR"
                            }
                        }
                    }
                },
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    if active_tab() == "cloud" {
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            left_icon: Some(rsx! {
                                Icon {
                                    kind: if hide_code() { IconKind::Eye } else { IconKind::EyeOff },
                                    size: 14,
                                }
                            }),
                            onclick: move |_| hide_code.set(!hide_code()),
                            if hide_code() { "Mostra Codice" } else { "Nascondi Codice" }
                        }
                    }

                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        left_icon: Some(rsx! { Icon { kind: IconKind::Copy, size: 14 } }),
                        onclick: {
                            let url = send_url();
                            move |_| {
                                let js = format!(r#"navigator.clipboard.writeText({:?});"#, url);
                                let _ = document::eval(&js);
                                notifications.success("Link partecipanti copiato negli appunti!");
                            }
                        },
                        "Copia Link"
                    }

                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        left_icon: Some(rsx! { Icon { kind: IconKind::Code, size: 14 } }),
                        onclick: move |_| show_embed.set(true),
                        "Iframe"
                    }

                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
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
                        "SVG"
                    }

                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
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
                        "PNG"
                    }
                }
            }

            if active_tab() == "cloud" && !hide_code() {
                div {
                    class: "floating-code-banner",
                    span { style: "font-size: 0.85rem; color: #a1a1aa;", "Codice di accesso:" }
                    Badge {
                        label: format!("#{code}"),
                        variant: BadgeVariant::Primary,
                        pill: true,
                        glow: Some(true),
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| hide_code.set(true),
                        "Nascondi"
                    }
                }
            }

            div {
                class: "wc-layout",
                match active_tab().as_str() {
                    "cloud" => rsx! {
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
                    "split" => rsx! {
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
                            Card {
                                variant: CardVariant::Elevated,
                                shadow: CardShadow::Md,
                                title: Some("Sessione dal Vivo".to_string()),
                                div {
                                    style: "display: flex; flex-direction: column; align-items: center; text-align: center; gap: 0.75rem;",
                                    Badge {
                                        label: format!("#{code}"),
                                        variant: BadgeVariant::Primary,
                                        pill: true,
                                        glow: Some(true),
                                    }
                                    QrCodeView { url: send_url(), class: Some("qr-wrapper".to_string()) }
                                    Text {
                                        size: TextSize::Sm,
                                        dimmed: true,
                                        "Inquadra con il telefono per inviare le tue parole"
                                    }
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Sm,
                                        full_width: true,
                                        left_icon: Some(rsx! { Icon { kind: IconKind::ExternalLink, size: 14 } }),
                                        onclick: {
                                            let url = send_url();
                                            move |_| {
                                                let js = format!(r#"window.open({:?}, '_blank');"#, url);
                                                let _ = document::eval(&js);
                                            }
                                        },
                                        "Apri Pagina Invio"
                                    }
                                }
                            }
                            div {
                                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem;",
                                Card {
                                    variant: CardVariant::Metric,
                                    title: Some("Totali".to_string()),
                                    div {
                                        style: "text-align: center; padding: 0.25rem 0;",
                                        Heading { order: HeadingOrder::H3, "{total_words}" }
                                    }
                                }
                                Card {
                                    variant: CardVariant::Metric,
                                    title: Some("Distinte".to_string()),
                                    div {
                                        style: "text-align: center; padding: 0.25rem 0;",
                                        Heading { order: HeadingOrder::H3, "{unique_words}" }
                                    }
                                }
                            }
                        }
                    },
                    "welcome" => rsx! {
                        div {
                            class: "welcome-screen-container",
                            Card {
                                variant: CardVariant::Elevated,
                                shadow: CardShadow::Xl,
                                glow: Some(true),
                                div {
                                    style: "display: flex; flex-direction: column; align-items: center; text-align: center; gap: 1rem; padding: 1.5rem 2rem;",
                                    Heading { order: HeadingOrder::H2, "Unisciti alla sessione dal vivo" }
                                    Text {
                                        size: TextSize::Lg,
                                        dimmed: true,
                                        "Scansiona il codice QR o collegati inserendo il codice:"
                                    }
                                    div {
                                        style: "margin: 0.5rem 0;",
                                        Badge {
                                            label: format!("#{code}"),
                                            variant: BadgeVariant::Primary,
                                            pill: true,
                                            glow: Some(true),
                                        }
                                    }
                                    QrCodeView { url: send_url(), class: Some("welcome-qr-wrapper".to_string()) }
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        size: ButtonSize::Lg,
                                        glow: Some(true),
                                        left_icon: Some(rsx! { Icon { kind: IconKind::Sparkles, size: 18 } }),
                                        onclick: move |_| active_tab.set("split".to_string()),
                                        "Mostra Nuvola di Parole"
                                    }
                                }
                            }
                        }
                    },
                    _ => rsx! {},
                }
            }

            if show_embed() {
                EmbedModal {
                    code: code.clone(),
                    origin: Some(site_origin()),
                    is_open: show_embed,
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
    let mut is_submitting = use_signal(|| false);
    let mut notifications = use_notification();
    let nav = navigator();

    rsx! {
        div {
            class: "app-shell",
            Navbar {
                brand: rsx! {
                    Link {
                        to: Route::Home {},
                        style: "display: flex; align-items: center; gap: 0.65rem; text-decoration: none; color: inherit;",
                        Icon { kind: IconKind::Sparkles, size: 20 }
                        span { style: "font-weight: 700; font-size: 1.15rem;", "Clord" }
                        Badge {
                            label: format!("Sessione #{code}"),
                            variant: BadgeVariant::Primary,
                            pill: true,
                        }
                    }
                },
                div {}
            }

            div {
                class: "send-container",
                Card {
                    title: Some("Invia la tua parola".to_string()),
                    subtitle: Some("Partecipa in tempo reale alla nuvola di parole condivisa.".to_string()),
                    variant: CardVariant::Elevated,
                    shadow: CardShadow::Lg,
                    glow: Some(true),
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
                                                notifications.success(format!("'{}' inviata con successo!", w));
                                                sent_words.write().push(w);
                                                word_input.set(String::new());
                                            }
                                            Err(err) => {
                                                notifications.danger(format!("Errore: {}", err));
                                            }
                                        }
                                        is_submitting.set(false);
                                    });
                                }
                            }
                        },
                        div {
                            style: "display: flex; flex-direction: column; gap: 1rem; margin-top: 0.5rem;",
                            TextInput {
                                value: word_input,
                                placeholder: Some("Scrivi qui una parola...".to_string()),
                                clearable: true,
                                disabled: is_submitting(),
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Md,
                                full_width: true,
                                loading: is_submitting(),
                                disabled: word_input().trim().is_empty() || is_submitting(),
                                left_icon: Some(rsx! { Icon { kind: IconKind::Sparkles, size: 16 } }),
                                onclick: {
                                    let code = code.clone();
                                    move |_| {
                                        let w = word_input().trim().to_string();
                                        if !w.is_empty() && !is_submitting() {
                                            is_submitting.set(true);
                                            let c = code.clone();
                                            spawn(async move {
                                                match backend::submit_word(c, w.clone()).await {
                                                    Ok(()) => {
                                                        notifications.success(format!("'{}' inviata con successo!", w));
                                                        sent_words.write().push(w);
                                                        word_input.set(String::new());
                                                    }
                                                    Err(err) => {
                                                        notifications.danger(format!("Errore: {}", err));
                                                    }
                                                }
                                                is_submitting.set(false);
                                            });
                                        }
                                    }
                                },
                                if is_submitting() { "Invio in corso..." } else { "Invia Parola 🚀" }
                            }
                        }
                    }

                    if !sent_words().is_empty() {
                        div {
                            style: "margin-top: 1.5rem; display: flex; flex-direction: column; gap: 0.5rem; border-top: 1px solid rgba(255, 255, 255, 0.1); padding-top: 1rem;",
                            Text {
                                size: TextSize::Sm,
                                dimmed: true,
                                "Le tue parole inviate:"
                            }
                            div {
                                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                                for (i, word) in sent_words().iter().enumerate() {
                                    Badge {
                                        key: "{i}-{word}",
                                        label: word.clone(),
                                        variant: BadgeVariant::Outline,
                                        pill: true,
                                    }
                                }
                            }
                        }
                    }

                    div {
                        style: "margin-top: 1.5rem; border-top: 1px solid rgba(255, 255, 255, 0.1); padding-top: 1rem;",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            full_width: true,
                            left_icon: Some(rsx! { Icon { kind: IconKind::ExternalLink, size: 14 } }),
                            onclick: {
                                let c = code.clone();
                                move |_| {
                                    nav.push(Route::WordCloudView { code: c.clone() });
                                }
                            },
                            "Guarda la nuvola in diretta"
                        }
                    }
                }
            }
        }
    }
}
