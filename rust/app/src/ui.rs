//! egui GUI for the Rust sudoku app. Behavior spec: `../SPEC-UI.md`
//! (extracted from `sudoku/ui.rkt`, wayfinder ticket #16).
//!
//! Custom-painted board inside the main window — no popup viewports
//! (they hang on Wayland, research ticket #9).

use eframe::egui;
use sudoku_core::{Difficulty, Grid, PEERS, analyze, format_analysis, generate_random};

pub const CELL: f32 = 48.0;
pub const MARGIN: f32 = 12.0;
pub const BOARD: f32 = 9.0 * CELL + 2.0 * MARGIN; // 456

const GIVEN: egui::Color32 = egui::Color32::from_rgb(20, 40, 160);
const USER: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
const ERROR: egui::Color32 = egui::Color32::from_rgb(200, 30, 30);
const HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(255, 244, 180);
const SELECTED: egui::Color32 = egui::Color32::from_rgb(255, 224, 102);
const PENCIL: egui::Color32 = egui::Color32::from_rgb(120, 120, 120);
const THIN_LINE: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
const BLACK: egui::Color32 = egui::Color32::BLACK;

/// Candidate heat overlay palette (SPEC-UI.md, map #20): one discrete color
/// per candidate count 1..=9 (few = green, many = red), 0 = error red.
pub fn heat_color(count: usize) -> egui::Color32 {
    match count {
        0 => egui::Color32::from_rgb(200, 30, 30),
        1 => egui::Color32::from_rgb(150, 233, 150),
        2 => egui::Color32::from_rgb(171, 233, 150),
        3 => egui::Color32::from_rgb(191, 233, 150),
        4 => egui::Color32::from_rgb(212, 233, 150),
        5 => egui::Color32::from_rgb(233, 233, 150),
        6 => egui::Color32::from_rgb(233, 212, 150),
        7 => egui::Color32::from_rgb(233, 191, 150),
        8 => egui::Color32::from_rgb(233, 171, 150),
        _ => egui::Color32::from_rgb(233, 150, 150),
    }
}

/// True when cell `i` holds a value that also appears in one of its peers.
pub fn is_conflict(g: &Grid, i: usize) -> bool {
    let v = g.get(i);
    v > 0 && PEERS[i].iter().any(|&p| p != i && g.get(p) == v)
}

/// Hint target: first empty cell, else first cell differing from the
/// solution — mirrors the Racket Hint button.
pub fn hint_target(current: &Grid, solution: &Grid) -> Option<usize> {
    (0..81)
        .find(|&i| current.get(i) == 0)
        .or_else(|| (0..81).find(|&i| current.get(i) != solution.get(i)))
}

fn difficulty_name(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Easy => "easy",
        Difficulty::Medium => "medium",
        Difficulty::Hard => "hard",
    }
}

pub struct SudokuApp {
    puzzle: Grid,
    current: Grid,
    solution: Option<Grid>,
    selected: Option<usize>,
    show_pencils: bool,
    show_heat: bool,
    show_analysis: bool,
    status: String,
    analysis: String,
    analysis_dirty: bool,
}

impl SudokuApp {
    pub fn new() -> Self {
        let mut app = Self {
            puzzle: Grid::empty(),
            current: Grid::empty(),
            solution: None,
            selected: None,
            show_pencils: false,
            show_heat: false,
            show_analysis: false,
            status: "Generate a puzzle to begin".to_string(),
            analysis: String::new(),
            analysis_dirty: true,
        };
        app.new_puzzle(Difficulty::Medium);
        app
    }

    fn on_change(&mut self) {
        self.analysis_dirty = true;
    }

    fn new_puzzle(&mut self, d: Difficulty) {
        let (puzzle, solution) = generate_random(d);
        self.status = format!(
            "New {} puzzle ({} givens)",
            difficulty_name(d),
            puzzle.givens()
        );
        self.puzzle = puzzle;
        self.current = puzzle;
        self.solution = Some(solution);
        self.selected = None;
        self.on_change();
    }

    fn solve(&mut self) {
        if let Some(sol) = self.solution {
            self.current = sol;
            self.status = "Solved.".to_string();
            self.on_change();
        }
    }

    fn hint(&mut self) {
        if let Some(sol) = self.solution
            && let Some(t) = hint_target(&self.current, &sol)
        {
            if self.puzzle.get(t) == 0 {
                self.current.set(t, sol.get(t));
                self.on_change();
            }
            self.status = format!("Hint: cell {} = {}", t + 1, sol.get(t));
        }
    }

    fn check(&mut self) {
        self.status = if !self.current.is_valid() {
            "There are conflicting entries.".to_string()
        } else if self.solution == Some(self.current) {
            "Solved correctly!".to_string()
        } else {
            "No conflicts so far.".to_string()
        };
    }

    fn recompute_analysis(&mut self) {
        let state = if !self.current.is_valid() {
            "conflicting entries present"
        } else if self.solution == Some(self.current) {
            "solved correctly"
        } else {
            "no conflicts so far"
        };
        self.analysis = format!(
            "Live analysis of the current board ({state})\n{}",
            format_analysis(&analyze(&self.current))
        );
        self.analysis_dirty = false;
    }

    fn handle_click(&mut self, pos: egui::Vec2) {
        let c = (pos.x / CELL) as i32;
        let r = (pos.y / CELL) as i32;
        if (0..=8).contains(&r) && (0..=8).contains(&c) {
            self.selected = Some(r as usize * 9 + c as usize);
        }
    }

    fn handle_char(&mut self, ch: char) {
        if let Some(i) = self.selected
            && self.puzzle.get(i) == 0
        {
            match ch {
                '1'..='9' => {
                    self.current.set(i, ch as u8 - b'0');
                    self.on_change();
                }
                '0' => {
                    self.current.set(i, 0);
                    self.on_change();
                }
                _ => {}
            }
        }
    }

    fn move_selection(&mut self, dr: i32, dc: i32) {
        let i = self.selected.unwrap_or(0);
        let r = (i / 9) as i32;
        let c = (i % 9) as i32;
        let nr = (r + dr).clamp(0, 8);
        let nc = (c + dc).clamp(0, 8);
        self.selected = Some(nr as usize * 9 + nc as usize);
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for ev in &i.events {
                if let egui::Event::Text(t) = ev {
                    for ch in t.chars() {
                        self.handle_char(ch);
                    }
                }
            }
            for (key, dr, dc) in [
                (egui::Key::ArrowUp, -1, 0),
                (egui::Key::ArrowDown, 1, 0),
                (egui::Key::ArrowLeft, 0, -1),
                (egui::Key::ArrowRight, 0, 1),
            ] {
                if i.key_pressed(key) {
                    self.move_selection(dr, dc);
                }
            }
            // digit keys as a fallback when no Text event is emitted
            // (setting the same value twice is idempotent)
            for (n, key) in (0..=9).zip([
                egui::Key::Num0,
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
                egui::Key::Num7,
                egui::Key::Num8,
                egui::Key::Num9,
            ]) {
                if i.key_pressed(key) {
                    self.handle_char((b'0' + n as u8) as char);
                }
            }
            if (i.key_pressed(egui::Key::Backspace) || i.key_pressed(egui::Key::Delete))
                && let Some(idx) = self.selected
                && self.puzzle.get(idx) == 0
            {
                self.current.set(idx, 0);
                self.on_change();
            }
        });
    }

    fn cell_rect(origin: egui::Pos2, i: usize) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(
                origin.x + MARGIN + (i % 9) as f32 * CELL,
                origin.y + MARGIN + (i / 9) as f32 * CELL,
            ),
            egui::vec2(CELL, CELL),
        )
    }

    fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        let origin = rect.min;
        painter.rect_filled(rect, 0.0, egui::Color32::WHITE);

        // highlights (under grid lines): peers first, then the heat overlay
        // on empty cells, then the selected cell on top (SPEC-UI.md, map #20)
        if let Some(s) = self.selected {
            for &p in &PEERS[s] {
                painter.rect_filled(Self::cell_rect(origin, p), 0.0, HIGHLIGHT);
            }
        }
        if self.show_heat {
            for i in 0..81 {
                if self.current.get(i) == 0 {
                    let n = self.current.candidates(i).len();
                    painter.rect_filled(Self::cell_rect(origin, i), 0.0, heat_color(n));
                }
            }
        }
        if let Some(s) = self.selected {
            painter.rect_filled(Self::cell_rect(origin, s), 0.0, SELECTED);
        }

        // grid lines: thick black every 3rd line, thin gray otherwise
        let board_edge = MARGIN + 9.0 * CELL;
        for k in 0..=9 {
            let pos = MARGIN + k as f32 * CELL;
            let thick = k % 3 == 0;
            let stroke = egui::Stroke::new(
                if thick { 2.0 } else { 1.0 },
                if thick { BLACK } else { THIN_LINE },
            );
            painter.line_segment(
                [
                    egui::pos2(origin.x + pos, origin.y + MARGIN),
                    egui::pos2(origin.x + pos, origin.y + board_edge),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(origin.x + MARGIN, origin.y + pos),
                    egui::pos2(origin.x + board_edge, origin.y + pos),
                ],
                stroke,
            );
        }
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(origin.x + MARGIN, origin.y + MARGIN),
                egui::vec2(9.0 * CELL, 9.0 * CELL),
            ),
            0.0,
            egui::Stroke::new(2.0, BLACK),
            egui::StrokeKind::Middle,
        );

        // cells
        let big = egui::FontId::new(22.0, egui::FontFamily::Proportional);
        let small = egui::FontId::new(10.0, egui::FontFamily::Proportional);
        for i in 0..81 {
            let cell = Self::cell_rect(origin, i);
            let pv = self.puzzle.get(i);
            let cv = self.current.get(i);
            if pv > 0 {
                painter.text(
                    cell.center(),
                    egui::Align2::CENTER_CENTER,
                    pv.to_string(),
                    big.clone(),
                    GIVEN,
                );
            } else if cv > 0 {
                let color = if is_conflict(&self.current, i) {
                    ERROR
                } else {
                    USER
                };
                painter.text(
                    cell.center(),
                    egui::Align2::CENTER_CENTER,
                    cv.to_string(),
                    big.clone(),
                    color,
                );
            } else if self.show_pencils {
                let cs = self.current.candidates(i);
                let mini = CELL / 3.0;
                for (k, v) in cs.iter().enumerate() {
                    let center = egui::pos2(
                        cell.min.x + (k % 3) as f32 * mini + mini / 2.0,
                        cell.min.y + (k / 3) as f32 * mini + mini / 2.0,
                    );
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        v.to_string(),
                        small.clone(),
                        PENCIL,
                    );
                }
            }
        }
    }
}

impl eframe::App for SudokuApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default().show(root, |ui| {
            ui.vertical_centered(|ui| {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(BOARD, BOARD), egui::Sense::click());
                if response.clicked()
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    self.handle_click(pos - rect.min);
                }
                self.handle_keys(&ctx);
                let painter = ui.painter_at(rect);
                self.paint(&painter, rect);
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("New Easy").clicked() {
                    self.new_puzzle(Difficulty::Easy);
                }
                if ui.button("New Medium").clicked() {
                    self.new_puzzle(Difficulty::Medium);
                }
                if ui.button("New Hard").clicked() {
                    self.new_puzzle(Difficulty::Hard);
                }
                if ui.button("Solve").clicked() {
                    self.solve();
                }
                if ui.button("Hint").clicked() {
                    self.hint();
                }
                if ui.button("Check").clicked() {
                    self.check();
                }
                if ui.button("Analyze").clicked() {
                    self.show_analysis = !self.show_analysis;
                    self.analysis_dirty = true;
                }
                ui.checkbox(&mut self.show_pencils, "Pencils");
                ui.checkbox(&mut self.show_heat, "Heat");
            });
            ui.add_space(4.0);
            ui.label(&self.status);
            if self.show_analysis {
                if self.analysis_dirty {
                    self.recompute_analysis();
                }
                ui.add_space(4.0);
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.analysis).monospace().size(12.0));
                    });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_detection_matches_peers() {
        let mut g = Grid::empty();
        g.set(0, 5);
        g.set(1, 5); // same row as 0
        g.set(9, 5); // same column as 0
        g.set(20, 5); // same box as 0
        assert!(is_conflict(&g, 0));
        assert!(is_conflict(&g, 1));
        assert!(is_conflict(&g, 9));
        assert!(is_conflict(&g, 20));
        g.set(40, 5); // unrelated cell (different row/col/box)
        assert!(!is_conflict(&g, 40));
        g.set(1, 0);
        g.set(9, 0);
        g.set(20, 0);
        assert!(!is_conflict(&g, 0));
    }

    #[test]
    fn hint_target_prefers_first_empty_then_first_mismatch() {
        let sol = Grid::from_puzzle_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        let mut cur = sol;
        cur.set(0, 0);
        cur.set(1, 9); // wrong vs solution (solution[1] = 3)
        // cell 0 is empty -> target 0 even though cell 1 also mismatches
        assert_eq!(hint_target(&cur, &sol), Some(0));
        cur.set(0, sol.get(0));
        assert_eq!(hint_target(&cur, &sol), Some(1));
        assert_eq!(hint_target(&sol, &sol), None);
    }

    #[test]
    fn heat_color_matches_the_9_step_palette() {
        let expected = [
            (200, 30, 30),   // 0 candidates: error red
            (150, 233, 150), // 1
            (171, 233, 150), // 2
            (191, 233, 150), // 3
            (212, 233, 150), // 4
            (233, 233, 150), // 5
            (233, 212, 150), // 6
            (233, 191, 150), // 7
            (233, 171, 150), // 8
            (233, 150, 150), // 9
        ];
        for (count, (r, g, b)) in expected.iter().enumerate() {
            assert_eq!(
                heat_color(count),
                egui::Color32::from_rgb(*r, *g, *b),
                "count {count}"
            );
        }
    }

    #[test]
    fn heat_counts_come_from_the_current_board() {
        // empty grid: every cell has 9 candidates; fill a peer of cell 0
        let mut g = Grid::empty();
        assert_eq!(g.candidates(0).len(), 9);
        g.set(1, 5);
        assert_eq!(g.candidates(0).len(), 8);
        // a dead cell (no candidates) is possible once all 9 digits are
        // taken by peers — here just check the count shrinks, not the color
        let dead = {
            let mut d = Grid::empty();
            for (k, p) in PEERS[0].iter().take(9).enumerate() {
                d.set(*p, (k + 1) as u8);
            }
            d.candidates(0).len()
        };
        assert_eq!(dead, 0);
    }
}
