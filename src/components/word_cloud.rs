use crate::model::WordCloudSession;
use dioxus::prelude::*;
use shadow_design::alert::{Alert, AlertVariant};
use shadow_design::badge::{Badge, BadgeVariant};

#[component]
pub fn WordCloud(
    session: WordCloudSession,
    on_delete_word: Option<EventHandler<String>>,
) -> Element {
    let placed_words = session.to_placed_words();

    if placed_words.is_empty() {
        return rsx! {
            div {
                style: "display: flex; align-items: center; justify-content: center; height: 100%; width: 100%; padding: 2rem;",
                Alert {
                    variant: AlertVariant::Info,
                    title: Some("Nuvola in attesa di parole".to_string()),
                    glow: Some(true),
                    "Scansiona il QR code o inserisci il codice sul tuo smartphone per iniziare a inviare le prime parole!"
                }
            }
        };
    }

    rsx! {
        div {
            class: "word-cloud-stage",
            div {
                class: "word-cloud-cluster",
                for item in placed_words {
                    {
                        let class_name = if item.is_vertical {
                            "cloud-word word-vertical"
                        } else {
                            "cloud-word word-horizontal"
                        };
                        let style = format!(
                            "font-size: {:.1}px; color: {}; text-shadow: 0 0 16px {}44; left: calc(50% + {:.1}px); top: calc(50% + {:.1}px);",
                            item.font_size_px, item.color_hsl, item.color_hsl, item.x, item.y
                        );
                        let word_to_delete = item.text.clone();
                        let badge_text = format!("{} ✕", item.count);

                        rsx! {
                            span {
                                key: "{item.text}",
                                class: class_name,
                                style: "{style}",
                                title: "{item.text} ({item.count}) • Clicca per rimuovere",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    if let Some(handler) = on_delete_word {
                                        handler.call(word_to_delete.clone());
                                    }
                                },
                                "{item.text}"
                                span {
                                    class: "word-badge-hover",
                                    Badge {
                                        label: badge_text,
                                        variant: BadgeVariant::Danger,
                                        pill: true,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
