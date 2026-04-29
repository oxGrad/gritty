use crate::state::{Frame, Project};

pub fn export_ansi_frame(frame: &Frame, width: u32, height: u32) -> String {
    let mut out = String::new();
    for row in 0..height as usize {
        for col in 0..width as usize {
            let cell = &frame.cells[row * width as usize + col];
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}",
                cell.fg[0], cell.fg[1], cell.fg[2],
                cell.bg[0], cell.bg[1], cell.bg[2],
                cell.ch
            ));
        }
        out.push_str("\x1b[0m\n");
    }
    out.push_str("\x1b[0m");
    out
}

pub fn export_ansi_all(project: &Project) -> String {
    project.frames
        .iter()
        .map(|f| export_ansi_frame(f, project.width, project.height))
        .collect::<Vec<_>>()
        .join("\x1b[2J\x1b[H")
}

pub fn export_json(project: &Project) -> String {
    serde_json::to_string_pretty(project).expect("project serialization never fails")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Cell, Frame, Project};

    fn make_frame_with_cell(ch: char, fg: [u8; 3], bg: [u8; 3]) -> Frame {
        let mut frame = Frame::new(2, 1);
        frame.cells[0] = Cell { fg, bg, ch };
        frame
    }

    #[test]
    fn ansi_frame_contains_escape_sequences() {
        let frame = make_frame_with_cell('█', [255, 97, 136], [34, 31, 34]);
        let out = export_ansi_frame(&frame, 2, 1);
        assert!(out.contains("\x1b[38;2;255;97;136m"));
        assert!(out.contains("\x1b[48;2;34;31;34m"));
        assert!(out.contains('█'));
        assert!(out.ends_with("\x1b[0m"));
    }

    #[test]
    fn ansi_frame_row_count_matches_height() {
        let frame = Frame::new(5, 3);
        let out = export_ansi_frame(&frame, 5, 3);
        assert_eq!(out.matches('\n').count(), 3);
    }

    #[test]
    fn ansi_all_inserts_clear_between_frames() {
        let mut project = Project::default();
        project.add_frame();
        let out = export_ansi_all(&project);
        assert!(out.contains("\x1b[2J\x1b[H"));
    }

    #[test]
    fn json_export_is_valid_json() {
        let project = Project::default();
        let json = export_json(&project);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["width"], 40);
        assert_eq!(parsed["height"], 20);
        assert!(parsed["frames"].is_array());
    }

    #[test]
    fn json_export_roundtrips_cell_data() {
        let mut project = Project::default();
        project.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '▄');
        let json = export_json(&project);
        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.frames[0].cells[0].ch, '▄');
        assert_eq!(parsed.frames[0].cells[0].fg, [255, 0, 0]);
    }
}
