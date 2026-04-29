/// Convert HSV (h: 0-360, s: 0-1, v: 0-1) to RGB [u8; 3].
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else              { (c, 0.0, x) };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}

/// Convert RGB [u8; 3] to HSV (h: 0-360, s: 0-1, v: 0-1).
pub fn rgb_to_hsv(rgb: [u8; 3]) -> (f64, f64, f64) {
    let r = rgb[0] as f64 / 255.0;
    let g = rgb[1] as f64 / 255.0;
    let b = rgb[2] as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Parse "#RRGGBB" or "RRGGBB" hex string into [u8; 3].
pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Format [u8; 3] as "#RRGGBB".
pub fn to_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_hsv_to_rgb() { assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), [255, 0, 0]); }

    #[test]
    fn green_hsv_to_rgb() { assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), [0, 255, 0]); }

    #[test]
    fn blue_hsv_to_rgb() { assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), [0, 0, 255]); }

    #[test]
    fn black_hsv_to_rgb() { assert_eq!(hsv_to_rgb(0.0, 0.0, 0.0), [0, 0, 0]); }

    #[test]
    fn white_hsv_to_rgb() { assert_eq!(hsv_to_rgb(0.0, 0.0, 1.0), [255, 255, 255]); }

    #[test]
    fn rgb_to_hsv_red() {
        let (h, s, v) = rgb_to_hsv([255, 0, 0]);
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn roundtrip_rgb_hsv() {
        let original = [169, 220, 118];
        let (h, s, v) = rgb_to_hsv(original);
        let back = hsv_to_rgb(h, s, v);
        for i in 0..3 {
            let diff = (original[i] as i16 - back[i] as i16).abs();
            assert!(diff <= 2, "channel {i}: original={} back={}", original[i], back[i]);
        }
    }

    #[test]
    fn parse_hex_with_hash() { assert_eq!(parse_hex("#FF6188"), Some([255, 97, 136])); }

    #[test]
    fn parse_hex_without_hash() { assert_eq!(parse_hex("a9dc76"), Some([169, 220, 118])); }

    #[test]
    fn parse_hex_invalid() {
        assert_eq!(parse_hex("#GGGGGG"), None);
        assert_eq!(parse_hex("#FFF"), None);
    }

    #[test]
    fn to_hex_roundtrip() {
        let color = [255, 97, 136];
        assert_eq!(parse_hex(&to_hex(color)), Some(color));
    }
}
