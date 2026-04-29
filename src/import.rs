use crate::state::{Cell, Frame, Project};

pub fn import_json(json: &str) -> Result<Project, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

pub fn import_ansi(text: &str, width: u32, height: u32) -> Project {
    let frame_texts: Vec<&str> = text.split("\x1b[2J").collect();
    let frames: Vec<Frame> = frame_texts
        .iter()
        .map(|t| parse_ansi_frame(t, width, height))
        .collect();
    let frames = if frames.is_empty() {
        vec![Frame::new(width, height)]
    } else {
        frames
    };
    Project { width, height, frames, active_frame: 0 }
}

fn parse_ansi_frame(text: &str, width: u32, height: u32) -> Frame {
    let mut frame = Frame::new(width, height);
    let mut col: u32 = 0;
    let mut row: u32 = 0;
    let mut fg: [u8; 3] = Cell::default().fg;
    let mut bg: [u8; 3] = Cell::default().bg;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i] != b'm' && bytes[i] != b'H' && bytes[i] != b'J' {
                i += 1;
            }
            if i >= bytes.len() { break; }
            let terminator = bytes[i] as char;
            let seq = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
            match terminator {
                'm' => parse_sgr(seq, &mut fg, &mut bg),
                'H' => {
                    let parts: Vec<&str> = seq.split(';').collect();
                    row = parts.first().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1).saturating_sub(1);
                    col = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1).saturating_sub(1);
                }
                _ => {}
            }
            i += 1;
        } else if bytes[i] == b'\n' {
            row += 1;
            col = 0;
            i += 1;
        } else {
            let ch = text[i..].chars().next().unwrap_or(' ');
            if col < width && row < height {
                let idx = (row * width + col) as usize;
                if let Some(cell) = frame.cells.get_mut(idx) {
                    cell.fg = fg;
                    cell.bg = bg;
                    cell.ch = ch;
                }
            }
            i += ch.len_utf8();
            col += 1;
        }
    }
    frame
}

fn parse_sgr(seq: &str, fg: &mut [u8; 3], bg: &mut [u8; 3]) {
    if seq == "0" || seq.is_empty() {
        *fg = Cell::default().fg;
        *bg = Cell::default().bg;
        return;
    }
    let parts: Vec<u8> = seq.split(';').filter_map(|s| s.parse().ok()).collect();
    let mut j = 0;
    while j < parts.len() {
        match parts[j] {
            38 if parts.get(j + 1) == Some(&2) => {
                if let (Some(&r), Some(&g), Some(&b)) =
                    (parts.get(j + 2), parts.get(j + 3), parts.get(j + 4))
                {
                    *fg = [r, g, b];
                    j += 5;
                    continue;
                }
            }
            48 if parts.get(j + 1) == Some(&2) => {
                if let (Some(&r), Some(&g), Some(&b)) =
                    (parts.get(j + 2), parts.get(j + 3), parts.get(j + 4))
                {
                    *bg = [r, g, b];
                    j += 5;
                    continue;
                }
            }
            0 => {
                *fg = Cell::default().fg;
                *bg = Cell::default().bg;
            }
            _ => {}
        }
        j += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{export_ansi_frame, export_json};
    use crate::state::Project;

    #[test]
    fn json_roundtrip() {
        let mut project = Project::default();
        project.paint_cell(2, 1, [255, 97, 136], [34, 31, 34], '▀');
        let json = export_json(&project);
        let imported = import_json(&json).unwrap();
        assert_eq!(imported.frames[0].cells[2 + 40].ch, '▀');
        assert_eq!(imported.frames[0].cells[2 + 40].fg, [255, 97, 136]);
    }

    #[test]
    fn json_import_invalid_returns_err() {
        assert!(import_json("not json").is_err());
    }

    #[test]
    fn ansi_roundtrip_single_frame() {
        let mut project = Project::default();
        project.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '█');
        let ansi = export_ansi_frame(&project.frames[0], project.width, project.height);
        let imported = import_ansi(&ansi, project.width, project.height);
        assert_eq!(imported.frames.len(), 1);
        assert_eq!(imported.frames[0].cells[0].ch, '█');
        assert_eq!(imported.frames[0].cells[0].fg, [255, 0, 0]);
    }

    #[test]
    fn ansi_import_multi_frame() {
        let mut project = Project::default();
        project.add_frame();
        let text = format!(
            "{}\x1b[2J{}",
            export_ansi_frame(&project.frames[0], project.width, project.height),
            export_ansi_frame(&project.frames[1], project.width, project.height),
        );
        let imported = import_ansi(&text, project.width, project.height);
        assert_eq!(imported.frames.len(), 2);
    }

    #[test]
    fn ansi_import_empty_string_gives_blank_frame() {
        let imported = import_ansi("", 10, 5);
        assert_eq!(imported.frames.len(), 1);
        assert_eq!(imported.frames[0].cells.len(), 50);
    }
}
