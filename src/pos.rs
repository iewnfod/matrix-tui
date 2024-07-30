use ratatui::crossterm::event::KeyCode;

use crate::app::{App, FocusArea};

// 寻找到在 direction 方向最接近的 rect 并返回其 FocusArea Enum
pub fn get_nearest_focus_area(app: &App, direction: KeyCode) -> Option<FocusArea> {
	if let Some(current_screen) = app.focus_area_positions.get(&app.current_focus) {
		let mut nearest_focus_area: Option<FocusArea> = None;
        let mut min_distance = std::isize::MAX;

		for (focus_area, rect) in app.focus_area_positions.iter() {
			if *focus_area == app.current_focus {
				continue;
			}

			let distance = match direction {
				KeyCode::Left => current_screen.x as isize - rect.x as isize,
				KeyCode::Right => rect.x as isize - current_screen.x as isize,
				KeyCode::Up => current_screen.y as isize - rect.y as isize,
				KeyCode::Down => rect.y as isize - current_screen.y as isize,
				_ => 0
			};

			if distance > 0 && distance < min_distance {
				min_distance = distance;
				nearest_focus_area = Some(focus_area.clone());
			}
		}

		nearest_focus_area
	} else {
		None
	}
}

// 寻找到最靠左，最靠上的 rect 并返回其 FocusArea Enum
pub fn get_top_left_focus_area(app: &App) -> Option<FocusArea> {
	let mut top_left_focus_area: Option<FocusArea> = None;
	let mut min_x = std::u16::MAX;
	let mut min_y = std::u16::MAX;

	for (focus_area, rect) in app.focus_area_positions.iter() {
		if rect.x < min_x {
			min_x = rect.x;
			top_left_focus_area = Some(focus_area.clone());
		}

		if rect.y < min_y {
			min_y = rect.y;
			top_left_focus_area = Some(focus_area.clone());
		}
	}

	top_left_focus_area
}
