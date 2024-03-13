use plotters::prelude::*;
use plotters::backend::{BGRXPixel, RGBPixel};
use std::borrow::BorrowMut;
use std::borrow::Borrow;
use eframe::egui::Color32;
use eframe::egui::ecolor::HexColor::Hex6;
use image::Rgba;
use crate::arm::Arm;
const OPACITY: f64 = 0.25;
const STROKE_WIDTH: u32 = 4;

pub fn generate_plot(buf: &mut Vec<u8>, w: u32, h: u32, arm: &Arm, yaw: f64, scale: f64) -> Result<(), Box<dyn std::error::Error>> {
    let area =
        BitMapBackend::<RGBPixel>::with_buffer_and_format(buf, (w , h))?
            .into_drawing_area();

    area.fill(&RGBColor(27, 27, 27).mix(1.1))?;

    let x_axis = (0.0..3.0).step(0.1);
    let y_axis = (-3.0..3.0).step(0.1);
    let z_axis = (-3.0..3.0).step(0.1);


    let mut chart = ChartBuilder::on(&area)
        .caption("Arm Locations".to_string(), ("times", 20, &WHITE))
        .build_cartesian_3d(x_axis.clone(), y_axis, z_axis)?;

    chart.with_projection(|mut pb| {
        pb.yaw = yaw;
        pb.pitch = 0.1;
        pb.scale = scale;
        pb.into_matrix()
    });

    chart
        .configure_axes()
        .light_grid_style(WHITE.mix(OPACITY))
        .max_light_lines(2)
        .label_style(TextStyle::from(("sans-serif", 15).into_font()).color(&WHITE))
        .draw()?;


    // Mark the Origin point with a circle
    chart
        .draw_series(std::iter::once(Circle::new(
            (0.0,0.0,0.0), // Position of the circle (end effector point)
            7, // Radius of the circle
            *&YELLOW.filled(), // Style of the circle
        )))?;

    chart
        .draw_series(LineSeries::new(
            vec![(0.0, 0.0, 0.0), arm.get_elbow()], // Points from Origin to Elbow
            *&YELLOW.stroke_width(STROKE_WIDTH), // Line color
        ))?
        .label("Lower Arm")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], YELLOW));


    chart
        .draw_series(LineSeries::new(
            vec![arm.get_elbow(), arm.get_ijk()], // Points from Elbow to End Effector)
            *&RED.stroke_width(STROKE_WIDTH), // Line color
        ))?
        .label("Upper Arm")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    // Mark the elbow point with a circle
    chart
        .draw_series(std::iter::once(Circle::new(
            arm.get_elbow(), // Position of the circle (elbow point)
            5, // Radius of the circle
            *&RED.filled(), // Style of the circle
        )))?;

    // Mark the IJK (End Effector) point with a circle
    chart
        .draw_series(std::iter::once(Circle::new(
            arm.get_ijk(), // Position of the circle (end effector point)
            5, // Radius of the circle
            *&BLUE.filled(), // Style of the circle
        )))?;

    chart.configure_series_labels().border_style(WHITE.mix(OPACITY)).label_font(("times", 12, &WHITE)).draw()?;

    area.present().expect("Unable to write result to memory buffer");
    Ok(())
}

