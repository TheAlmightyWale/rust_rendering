use crate::properties::Color;
use crate::properties::BG_COLOR;
use crate::state::Surface;
use std::mem;

pub fn draw_line(
    start: cgmath::Vector2<f32>,
    end: cgmath::Vector2<f32>,
    color: Color<u8>,
    surface: &mut dyn Surface,
) {
    //Need to swap draw algorithm over to handle completely vertical and completely horizontal lines
    let run = end.x - start.x;
    let rise = end.y - start.y;
    let mut starting_point = start;
    let mut end_point = end;

    if run.abs() > rise.abs() {
        //Line is horizontal-ish, ensure start is "left-most" x
        if start.x > end.x {
            mem::swap(&mut starting_point, &mut end_point);
        }

        let y_coords = interpolate(
            starting_point.x as usize,
            end_point.x as usize,
            starting_point.y,
            end_point.y,
        );

        //TODO: Create f32 version of set pixel which checks and converts for you
        for (step, x) in (starting_point.x as u32..end_point.x as u32).enumerate() {
            surface.set_pixel(x, y_coords[step] as u32, &color);
        }
    } else {
        //Vertical-ish line, ensure start is lowest y
        if start.y > end.y {
            mem::swap(&mut starting_point, &mut end_point);
        }

        let x_coords = interpolate(
            starting_point.y as usize,
            end_point.y as usize,
            starting_point.x,
            end_point.x,
        );

        for (step, y) in (starting_point.y as u32..end_point.y as u32).enumerate() {
            surface.set_pixel(x_coords[step] as u32, y, &color);
        }
    }
}

pub fn clear_screen(surface: &mut dyn Surface) {
    for y in 0..surface.get_height() {
        for x in 0..surface.get_width() {
            surface.set_pixel(x, y, &BG_COLOR);
        }
    }
}

pub fn interpolate(
    independent_start: usize,
    independent_end: usize,
    dependent_start: f32,
    dependent_end: f32,
) -> Vec<f32> {
    if independent_start == independent_end {
        return vec![dependent_start];
    }

    let mut values = Vec::new();
    let gradient: f32 =
        (dependent_end - dependent_start) / (independent_end as f32 - independent_start as f32);

    let num_steps = independent_end - independent_start;
    for step in 0..num_steps {
        let interpolation_step = dependent_start + (gradient * step as f32);
        values.push(interpolation_step);
    }

    //Make interpolation inclusive of end point
    values.push(dependent_end);

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_single_value() {
        //Point at 2,1 and 2,3
        let values = interpolate(2, 2, 1.0, 3.0);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], 1.0);
    }

    #[test]
    fn interpolate_correct_gradient() {
        //Point at 1,1 and 5,5
        let starting_y = 1.0;
        let end_y = 5.0;

        let values = interpolate(1, 5, starting_y, end_y);
        let expected_gradient = 1.0;
        assert_eq!(values.len(), 5);
        assert_eq!(values[0], starting_y);
        assert_eq!(values[1], starting_y + expected_gradient);
        assert_eq!(values[2], starting_y + (expected_gradient * 2.0));
        assert_eq!(values[3], starting_y + (expected_gradient * 3.0));
        assert_eq!(values[4], end_y);
    }
}
