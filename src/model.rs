use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WordItem {
    pub text: String,
    pub count: u32,
    pub is_vertical: bool,
    pub color_hsl: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WordCloudSession {
    pub code: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub words: HashMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlacedWord {
    pub text: String,
    pub count: u32,
    pub is_vertical: bool,
    pub color_hsl: String,
    pub font_size_px: f32,
    pub x: f32, // Offset in pixels from center
    pub y: f32, // Offset in pixels from center
}

#[derive(Clone, Copy, Debug)]
struct BoundingBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl BoundingBox {
    fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.x + self.w <= other.x
            || self.x >= other.x + other.w
            || self.y + self.h <= other.y
            || self.y >= other.y + other.h)
    }
}

impl WordCloudSession {
    pub fn new(code: String, now: u64) -> Self {
        Self {
            code,
            created_at: now,
            updated_at: now,
            words: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn to_word_items(&self) -> Vec<WordItem> {
        let mut items: Vec<WordItem> = self
            .words
            .iter()
            .map(|(text, &count)| WordItem {
                text: text.clone(),
                count,
                is_vertical: word_is_vertical(text),
                color_hsl: word_color_hsl(text),
            })
            .collect();

        // Sort descending by count, then alphabetically
        items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.text.cmp(&b.text)));
        items
    }

    /// Computes dense, interlocking Archimedean spiral cloud layout (mosaic/wall style)
    pub fn to_placed_words(&self) -> Vec<PlacedWord> {
        if self.words.is_empty() {
            return Vec::new();
        }

        let mut raw_items: Vec<(&String, u32)> = self.words.iter().map(|(k, &v)| (k, v)).collect();
        // Highest frequency words first so they anchor in the center
        raw_items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

        let min_count = raw_items.iter().map(|item| item.1).min().unwrap_or(1);
        let max_count = raw_items.iter().map(|item| item.1).max().unwrap_or(1);

        let mut placed_boxes: Vec<BoundingBox> = Vec::with_capacity(raw_items.len());
        let mut results: Vec<PlacedWord> = Vec::with_capacity(raw_items.len());

        for (rank, (text, count)) in raw_items.into_iter().enumerate() {
            // Sizing: prominent words get bigger font sizes
            let font_size_px = if max_count == min_count {
                if rank == 0 { 54.0 } else { 28.0 }
            } else {
                let scale = (count - min_count) as f32 / (max_count - min_count) as f32;
                16.0 + scale * 46.0
            };

            // Main center word is horizontal; other words use deterministic orientation (~35% vertical)
            let is_vertical = if rank == 0 {
                false
            } else {
                word_is_vertical(text)
            };

            let color_hsl = word_color_hsl(text);

            // Approximate dimensions based on font metrics with interlock padding
            let char_len = text.chars().count().max(1) as f32;
            let pad = 6.0;
            let (box_w, box_h) = if is_vertical {
                (font_size_px * 1.1 + pad, char_len * font_size_px * 0.58 + pad)
            } else {
                (char_len * font_size_px * 0.58 + pad, font_size_px * 1.1 + pad)
            };

            let mut best_x = 0.0f32;
            let mut best_y = 0.0f32;
            let mut chosen_box = BoundingBox {
                x: -box_w / 2.0,
                y: -box_h / 2.0,
                w: box_w,
                h: box_h,
            };

            if rank == 0 {
                // Center anchor word
                placed_boxes.push(chosen_box);
                best_x = 0.0;
                best_y = 0.0;
            } else {
                let hash = text
                    .bytes()
                    .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
                // Angular offset based on hash so words spiral outwards organically in different directions
                let angle_offset = (hash % 360) as f32 * (std::f32::consts::PI / 180.0);

                let mut t = 0.0f32;
                let dt = 0.16f32; // Fine step for tight packing
                let dr = 1.8f32;  // Radial density
                let aspect = 1.35f32; // Elliptical aspect ratio

                while t < 350.0 {
                    let theta = t + angle_offset;
                    let r = dr * t;
                    let cx = r * theta.cos() * aspect;
                    let cy = r * theta.sin();

                    let test_box = BoundingBox {
                        x: cx - box_w / 2.0,
                        y: cy - box_h / 2.0,
                        w: box_w,
                        h: box_h,
                    };

                    let collides = placed_boxes.iter().any(|b| test_box.intersects(b));
                    if !collides {
                        chosen_box = test_box;
                        best_x = cx;
                        best_y = cy;
                        placed_boxes.push(chosen_box);
                        break;
                    }
                    t += dt;
                }
            }

            results.push(PlacedWord {
                text: text.clone(),
                count,
                is_vertical,
                color_hsl,
                font_size_px,
                x: best_x,
                y: best_y,
            });
        }

        // Auto-center the entire cluster
        if !placed_boxes.is_empty() {
            let min_x = placed_boxes.iter().map(|b| b.x).fold(f32::INFINITY, f32::min);
            let max_x = placed_boxes.iter().map(|b| b.x + b.w).fold(f32::NEG_INFINITY, f32::max);
            let min_y = placed_boxes.iter().map(|b| b.y).fold(f32::INFINITY, f32::min);
            let max_y = placed_boxes.iter().map(|b| b.y + b.h).fold(f32::NEG_INFINITY, f32::max);

            let center_offset_x = (min_x + max_x) / 2.0;
            let center_offset_y = (min_y + max_y) / 2.0;

            for item in &mut results {
                item.x -= center_offset_x;
                item.y -= center_offset_y;
            }
        }

        results
    }

    pub fn total_words(&self) -> u32 {
        self.words.values().sum()
    }

    pub fn unique_words(&self) -> usize {
        self.words.len()
    }

    /// Exports the current cloud as a high-resolution, standalone SVG string
    pub fn generate_svg(&self, width: u32, height: u32) -> String {
        let placed = self.to_placed_words();
        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" style="background-color:#090a10;">"##
        );
        svg.push_str(r#"<style>text { font-family: -apple-system, system-ui, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; font-weight: 700; }</style>"#);
        svg.push_str(r##"<rect width="100%" height="100%" fill="#090a10"/>"##);

        let half_w = (width as f32) / 2.0;
        let half_h = (height as f32) / 2.0;

        for word in placed {
            let cx = half_w + word.x;
            let cy = half_h + word.y;
            let escaped = xml_escape(&word.text);
            if word.is_vertical {
                svg.push_str(&format!(
                    r#"<text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}px" text-anchor="middle" dominant-baseline="central" transform="rotate(-90 {:.1} {:.1})">{}</text>"#,
                    cx, cy, word.color_hsl, word.font_size_px, cx, cy, escaped
                ));
            } else {
                svg.push_str(&format!(
                    r#"<text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}px" text-anchor="middle" dominant-baseline="central">{}</text>"#,
                    cx, cy, word.color_hsl, word.font_size_px, escaped
                ));
            }
        }
        svg.push_str("</svg>");
        svg
    }
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Deterministically calculate if a word should be rendered vertically (90deg).
/// ~30% of words will be vertical, consistently for that specific word.
pub fn word_is_vertical(text: &str) -> bool {
    let hash = text
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    hash % 3 == 0
}

/// Deterministically pick a bright, modern HSL color for the word.
pub fn word_color_hsl(text: &str) -> String {
    let hash = text
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u32));
    // Diverse vibrant palette of hues
    const HUES: &[u32] = &[185, 205, 230, 265, 285, 320, 345, 35, 145, 168];
    let hue = HUES[(hash as usize) % HUES.len()];
    format!("hsl({}, 88%, 68%)", hue)
}

/// Helper to generate SVG QR code string in pure Rust without external requests
pub fn generate_qr_svg(url: &str) -> String {
    match qrcode::QrCode::new(url.as_bytes()) {
        Ok(code) => code
            .render::<qrcode::render::svg::Color>()
            .min_dimensions(180, 180)
            .quiet_zone(true)
            .build(),
        Err(_) => String::new(),
    }
}

/// Cross-platform sleep helper for client and server polling
pub async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        gloo_timers::future::TimeoutFuture::new(ms).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_cloud_session_creation_and_words() {
        let mut session = WordCloudSession::new("4242".to_string(), 1000);
        assert_eq!(session.code, "4242");
        assert_eq!(session.total_words(), 0);
        assert_eq!(session.unique_words(), 0);

        session.words.insert("tecnologia".to_string(), 3);
        session.words.insert("innovazione".to_string(), 1);

        assert_eq!(session.total_words(), 4);
        assert_eq!(session.unique_words(), 2);

        let items = session.to_word_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "tecnologia");
        assert_eq!(items[0].count, 3);
        assert_eq!(items[1].text, "innovazione");
        assert_eq!(items[1].count, 1);
    }

    #[test]
    fn test_deterministic_orientation_and_colors() {
        let word = "progetto";
        let vert1 = word_is_vertical(word);
        let vert2 = word_is_vertical(word);
        assert_eq!(vert1, vert2, "Word orientation must be deterministic across renders");

        let color1 = word_color_hsl(word);
        let color2 = word_color_hsl(word);
        assert_eq!(color1, color2, "Word color must be deterministic across renders");
        assert!(color1.starts_with("hsl("));
    }

    #[test]
    fn test_qr_svg_generation() {
        let url = "https://example.com/wc/1024/send";
        let svg = generate_qr_svg(url);
        assert!(svg.contains("<svg"), "QR SVG should contain <svg tag");
        assert!(svg.contains("</svg>"), "QR SVG should contain closing </svg> tag");
    }

    #[test]
    fn test_cloud_spiral_interlocking_layout() {
        let mut session = WordCloudSession::new("9999".to_string(), 100);
        let sample_words = [
            ("canon", 12),
            ("aristotle", 9),
            ("interpretation", 8),
            ("mimesis", 6),
            ("rhetoric", 5),
            ("dryden", 4),
            ("homer", 4),
            ("poetics", 3),
            ("ideal", 3),
            ("criticism", 2),
            ("philosophy", 2),
            ("tradition", 2),
            ("subject", 1),
        ];

        for (word, count) in sample_words {
            session.words.insert(word.to_string(), count);
        }

        let placed = session.to_placed_words();
        assert_eq!(placed.len(), sample_words.len());

        // Check that at least some words are horizontal and some are vertical
        let has_horizontal = placed.iter().any(|p| !p.is_vertical);
        let has_vertical = placed.iter().any(|p| p.is_vertical);
        assert!(has_horizontal, "Must have horizontal words");
        assert!(has_vertical, "Must have vertical words");

        // The top word should be large
        let top_word = placed.iter().find(|p| p.text == "canon").unwrap();
        let small_word = placed.iter().find(|p| p.text == "subject").unwrap();
        assert!(top_word.font_size_px > small_word.font_size_px);
    }

    #[test]
    fn test_generate_cloud_svg() {
        let mut session = WordCloudSession::new("1234".to_string(), 100);
        session.words.insert("tecnologia".to_string(), 5);
        session.words.insert("innovazione".to_string(), 2);

        let svg = session.generate_svg(1600, 1000);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("tecnologia"));
        assert!(svg.contains("innovazione"));
    }
}
