use crate::properties::Color;
use crate::properties::BG_COLOR;
use crate::state::State;
use cgmath::Vector2;
use std::mem;

pub fn draw_line(
    start: cgmath::Vector2<f32>,
    end: cgmath::Vector2<f32>,
    color: Color<u8>,
    state: &mut State,
) {
    //Need to swap draw algorithm over to handle completely vertical and completely horizontal lines
    let run = end.x - start.x;
    let rise = end.y - start.y;
    let mut starting_point = start;
    let mut end_point = end;

    if (run.abs() > rise.abs()) {
        //Line is horizontal-ish, ensure start is "left-most" x
        if start.x > end.x {
            mem::swap(&mut starting_point, &mut end_point);
        }

        let gradient = rise / run;
        let mut current_height = starting_point.y;

        //TODO: Create f32 version of set pixel which checks and converts for you
        for x in starting_point.x as u32..end_point.x as u32 {
            state.set_pixel(x, current_height as u32, &color);
            current_height = current_height + gradient;
        }
    } else {
        //Vertical-ish line, ensure start is lowest y
        if (start.y > end.y) {
            mem::swap(&mut starting_point, &mut end_point);
        }

        let gradient = run / rise;
        let mut current_width = starting_point.x;

        for y in starting_point.y as u32..end_point.y as u32 {
            state.set_pixel(current_width as u32, y, &color);
            current_width = current_width + gradient;
        }
    }
}

pub fn clear_screen(state: &mut State) {
    for y in 0..state.texture.size.height {
        for x in 0..state.texture.size.width {
            state.set_pixel(x, y, &BG_COLOR);
        }
    }
}
