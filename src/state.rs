use serde::{Deserialize, Serialize};

pub const CELL_W: f64 = 8.0;
pub const CELL_H: f64 = 16.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub ch: char,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { fg: [252, 252, 250], bg: [34, 31, 34], ch: ' ' }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub cells: Vec<Cell>,
}

impl Frame {
    pub fn new(width: u32, height: u32) -> Self {
        Frame {
            cells: vec![Cell::default(); (width * height) as usize],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Frame>,
    pub active_frame: usize,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            width: 40,
            height: 20,
            frames: vec![Frame::new(40, 20)],
            active_frame: 0,
        }
    }
}

impl Project {
    pub fn paint_cell(&mut self, col: u32, row: u32, fg: [u8; 3], bg: [u8; 3], ch: char) {
        if col >= self.width || row >= self.height { return; }
        let idx = (row * self.width + col) as usize;
        if let Some(frame) = self.frames.get_mut(self.active_frame) {
            if let Some(cell) = frame.cells.get_mut(idx) {
                cell.fg = fg;
                cell.bg = bg;
                cell.ch = ch;
            }
        }
    }

    pub fn erase_cell(&mut self, col: u32, row: u32) {
        self.paint_cell(col, row, Cell::default().fg, Cell::default().bg, ' ');
    }

    pub fn add_frame(&mut self) {
        self.frames.push(Frame::new(self.width, self.height));
    }

    pub fn duplicate_frame(&mut self, index: usize) {
        if index < self.frames.len() {
            let frame = self.frames[index].clone();
            self.frames.insert(index + 1, frame);
        }
    }

    pub fn delete_frame(&mut self, index: usize) {
        if self.frames.len() <= 1 { return; }
        self.frames.remove(index);
        if self.active_frame >= self.frames.len() {
            self.active_frame = self.frames.len() - 1;
        }
    }

    pub fn move_frame_up(&mut self, index: usize) {
        if index == 0 { return; }
        self.frames.swap(index, index - 1);
        if self.active_frame == index { self.active_frame -= 1; }
        else if self.active_frame == index - 1 { self.active_frame += 1; }
    }

    pub fn move_frame_down(&mut self, index: usize) {
        if index + 1 >= self.frames.len() { return; }
        self.frames.swap(index, index + 1);
        if self.active_frame == index { self.active_frame += 1; }
        else if self.active_frame == index + 1 { self.active_frame -= 1; }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorTarget {
    Fg,
    Bg,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaybackState {
    pub playing: bool,
    pub delay_ms: u32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState { playing: false, delay_ms: 100 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    pub project: Project,
    pub tool: Tool,
    pub active_glyph: char,
    pub fg_color: [u8; 3],
    pub bg_color: [u8; 3],
    pub color_target: ColorTarget,
    pub playback: PlaybackState,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            project: Project::default(),
            tool: Tool::Brush,
            active_glyph: '█',
            fg_color: [255, 97, 136],
            bg_color: [34, 31, 34],
            color_target: ColorTarget::Fg,
            playback: PlaybackState::default(),
        }
    }
}

pub const GLYPH_GROUPS: &[(&str, &[char])] = &[
    ("Full",   &['█']),
    ("H-Half", &['▀', '▄']),
    ("V-Half", &['▌', '▐']),
    ("Shade",  &['░', '▒', '▓']),
    ("Quad",   &['▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟']),
    ("Space",  &[' ']),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_project_has_one_frame() {
        let p = Project::default();
        assert_eq!(p.frames.len(), 1);
        assert_eq!(p.width, 40);
        assert_eq!(p.height, 20);
    }

    #[test]
    fn paint_cell_updates_correct_index() {
        let mut p = Project::default();
        p.paint_cell(1, 0, [255, 0, 0], [0, 0, 0], '█');
        assert_eq!(p.frames[0].cells[1].ch, '█');
        assert_eq!(p.frames[0].cells[0].ch, ' ');
    }

    #[test]
    fn paint_cell_out_of_bounds_does_not_panic() {
        let mut p = Project::default();
        p.paint_cell(999, 999, [255, 0, 0], [0, 0, 0], '█');
    }

    #[test]
    fn erase_cell_resets_to_default() {
        let mut p = Project::default();
        p.paint_cell(0, 0, [255, 0, 0], [255, 0, 0], '█');
        p.erase_cell(0, 0);
        assert_eq!(p.frames[0].cells[0], Cell::default());
    }

    #[test]
    fn add_frame_increments_count() {
        let mut p = Project::default();
        p.add_frame();
        assert_eq!(p.frames.len(), 2);
    }

    #[test]
    fn duplicate_frame_inserts_after() {
        let mut p = Project::default();
        p.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '█');
        p.duplicate_frame(0);
        assert_eq!(p.frames.len(), 2);
        assert_eq!(p.frames[1].cells[0].ch, '█');
    }

    #[test]
    fn delete_frame_refuses_last() {
        let mut p = Project::default();
        p.delete_frame(0);
        assert_eq!(p.frames.len(), 1);
    }

    #[test]
    fn delete_frame_removes_and_clamps_active() {
        let mut p = Project::default();
        p.add_frame();
        p.active_frame = 1;
        p.delete_frame(1);
        assert_eq!(p.active_frame, 0);
    }

    #[test]
    fn move_frame_down_swaps_correctly() {
        let mut p = Project::default();
        p.add_frame();
        p.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '█');
        p.active_frame = 0;
        let ch_before = p.frames[0].cells[0].ch;
        p.move_frame_down(0);
        assert_eq!(p.frames[1].cells[0].ch, ch_before);
        assert_eq!(p.active_frame, 1);
    }
}
