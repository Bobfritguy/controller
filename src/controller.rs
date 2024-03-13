
use std::borrow::Borrow;
use crate::plot::{generate_plot};
use crate::network;
use crate::arm;
use std::net::UdpSocket;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};
use eframe::egui;
use eframe::egui::{Ui, Separator, ComboBox, Slider, Sense, vec2, WidgetType, WidgetInfo, DragValue, Response, pos2, lerp, Widget, Image, ColorImage, TextureOptions};
use plotters::prelude::*;

use plotters::drawing::IntoDrawingArea;
use plotters::element::Rectangle;
use crate::models::{SharedState, Mode};
const MOVE_SCALE: f32 = 0.01;
const SCROLL_SCALE: f32 = 0.001;

pub struct Controller {
    ip_addr_string: String,
    is_ip_addr: bool,
    send_to: String,
    udp_socket: UdpSocket,
    servo_top_range: RangeInclusive<f64>,
    servo_shoulder_range: RangeInclusive<f64>,
    servo_upper_range: RangeInclusive<f64>,
    servo_elbow_range: RangeInclusive<f64>,
    servo_lower_range: RangeInclusive<f64>,
    send_vec: Vec<u8>,
    receive_vec: Vec<u8>,
    received_values: Vec<f64>,
    mode: Mode,
    send: bool,
    flag: bool,
    shared_state: Arc<Mutex<SharedState>>,
    arm: arm::Arm,
    target_i: f64,
    target_j: f64,
    target_k: f64,
    plot_yaw: f64,
    plot_scale: f64,
}

impl Controller {
    pub fn new(shared_state: Arc<Mutex<SharedState>>) -> Self {
        let udp_socket = UdpSocket::bind("0.0.0.0:8080").expect("Failed to bind socket");
        udp_socket.set_nonblocking(true).expect("Failed to set nonblocking");


        Controller {
            ip_addr_string: "0.0.0.0:1234".to_owned(),
            is_ip_addr: true,
            send_to: "0.0.0.0:1234".to_owned(),
            udp_socket,
            servo_top_range: 0.0..=180.0,
            servo_shoulder_range: 0.0..=180.0,
            servo_upper_range: 0.0..=180.0,
            servo_elbow_range: 0.0..=180.0,
            servo_lower_range: 0.0..=180.0,
            send_vec: Vec::new(),
            receive_vec: vec![0; 11],
            received_values: vec![0.0; 5],
            mode: Mode::Stopped,
            send: true,
            flag: true,
            shared_state,
            arm: arm::Arm::new(1.0, 1.0),
            target_i: 1.0,
            target_j: 1.0,
            target_k: 1.0,
            plot_yaw: 0.5,
            plot_scale: 0.55,
        }
    }

    pub fn render_ui(&mut self, ui: &mut Ui) {
        // Heading
        ui.heading("Limb Controller");

        // IP Address input and validation
        ui.horizontal(|ui| {
            let name_label = ui.label("IP Address: ");
            ui.text_edit_singleline(&mut self.ip_addr_string).labelled_by(name_label.id);
            Controller::mdns_button(ui, &mut self.ip_addr_string, &self.shared_state);
            self.is_ip_addr = network::Network::check_ip_string(&self.ip_addr_string);
            ui.label(if self.is_ip_addr { "Valid" } else { "Invalid" });
            if self.is_ip_addr {
                if ui.button("Apply").clicked() {
                    self.send_to = self.ip_addr_string.clone();
                }
            }
        });

        // Mode selection
        ui.add(Separator::default());
        ComboBox::from_label("Choose Mode")
            .selected_text(format!("{:?}", self.mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.mode, Mode::Sending, "Sending");
                ui.selectable_value(&mut self.mode, Mode::Receiving, "Receiving");
                ui.selectable_value(&mut self.mode, Mode::Stopped, "Stopped");
                ui.selectable_value(&mut self.mode, Mode::Settings, "Settings");
            });

        // Mode-specific UI
        match self.mode {
            Mode::Sending => {
                self.render_sending_mode_ui(ui);
                self.render_receive_ui();
                self.render_arm_status_ui(ui);
                self.render_plot(ui);
            }
            Mode::Receiving => {
                ui.label("Stopped");
            },
            Mode::Stopped => {
                ui.label("Stopped");
            }
            Mode::Settings => {
                self.render_settings(ui);
            }
        }
    }

    fn render_sending_mode_ui(&mut self, ui: &mut Ui) {
        ui.label(format!("Sending Data to {}", &self.send_to));
        if ui.button("Reset").clicked() {
            *self.arm.servo_a_horiz() = (self.servo_top_range.end() + self.servo_top_range.start()) / 2.0;
            *self.arm.servo_a_vert() = (self.servo_shoulder_range.end() + self.servo_shoulder_range.start()) / 2.0;
            *self.arm.servo_b_horiz() = (self.servo_upper_range.end() + self.servo_upper_range.start()) / 2.0;
            *self.arm.servo_b_vert() = *self.servo_elbow_range.start();
            *self.arm.servo_c_horiz() = (self.servo_lower_range.end() + self.servo_lower_range.start()) / 2.0;
            self.flag = true;
        }
        // Servo control sliders
        Controller::render_servo_control(ui, &self.servo_top_range, self.arm.servo_a_horiz(), "Top Servo", &mut self.flag);
        Controller::render_servo_control(ui, &self.servo_shoulder_range, self.arm.servo_a_vert(), "Shoulder Servo", &mut self.flag);
        Controller::render_servo_control(ui, &self.servo_upper_range, self.arm.servo_b_horiz(), "Upper Servo", &mut self.flag);
        Controller::render_servo_control(ui, &self.servo_elbow_range, self.arm.servo_b_vert(), "Elbow Servo", &mut self.flag);
        Controller::render_servo_control(ui, &self.servo_lower_range, self.arm.servo_c_horiz(), "Lower Servo", &mut self.flag);
        self.arm.update();
        ui.label(format!("Packet {:?}", &self.send_vec));

        ui.horizontal(|ui| {
            ui.add(Controller::toggle(&mut self.send));
            ui.add(Separator::default());
            ui.label(if self.send { "Auto Send" } else { "Manual Send" });
        });

        if !self.send {
            if ui.button("Send").clicked() {
                self.send_data().expect("Failed to send data");
            }
        } else if self.flag {
            self.send_data().expect("Failed to send data");
        }

        // Receive data
        match self.udp_socket.recv_from(&mut self.receive_vec) {
            Ok(_) => {}
            Err(_) => {}
        };
        for (i, chunk) in self.receive_vec.chunks_exact(2).enumerate() { // Divides the received data into 2 byte chunks, and iterates over them
            self.received_values[i] = chunk.try_into()
                .map(|bytes: [u8; 2]| u16::from_be_bytes(bytes) as f64) // Convert bytes to u16, then to f64
                .unwrap_or(-1.0); // Default to -1.0 if conversion fails, indicating a problem converting from bytes to u16
            if self.received_values == vec![
                *self.arm.servo_a_horiz(),
                *self.arm.servo_a_vert(),
                *self.arm.servo_b_horiz(),
                *self.arm.servo_b_vert()
            ] {
                self.flag = false;
            }
        }
        ui.label(format!("Received {:?}, flag set to: {}", &self.received_values, &self.flag));
    }

    fn render_servo_control(ui: &mut Ui, range: &RangeInclusive<f64>, angle: &mut f64, label: &str, flag: &mut bool) {
        ui.horizontal(|ui| {
            // Label for the servo
            ui.label(format!("{} Position:", label));

            // Slider for the servo
            Controller::flag_setting_slider(
                ui,
                angle,
                range.clone(),
                "°",
                flag,
            );
            if ui.button("-").clicked() {
                *angle = (*angle - 1.0).max(*range.start());
                *flag = true;
            }
            if ui.button("+").clicked() {
                *angle = (*angle + 1.0).min(*range.end());
                *flag = true;
            }
        });
        ui.end_row(); // End the current row and prepare for the next
    }

    fn flag_setting_slider(
        ui: &mut Ui,
        value: &mut f64,
        range: RangeInclusive<f64>,
        suffix: &str,
        flag: &mut bool,
    ) {
        let slider_response = ui.add(Slider::new(value, range).suffix(suffix));
        if slider_response.changed() {
            *flag = true;
        }
    }

    // Code for egui toggle switch.
    fn toggle_ui(ui: &mut Ui, on: &mut bool) -> Response {
        let desired_size = ui.spacing().interact_size.y * vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, *on, ""));

        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool(response.id, *on);
            let visuals = ui.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }
        response
    }

    // A wrapper that allows the more idiomatic usage pattern: `ui.add(toggle(&mut my_bool))`
    /// iOS-style toggle switch.
    ///
    /// ## Example:
    /// ``` ignore
    /// ui.add(toggle(&mut my_bool));
    /// ```
    fn toggle(on: &mut bool) -> impl Widget + '_ {
        move |ui: &mut Ui| Controller::toggle_ui(ui, on)
    }

    fn send_data(&mut self) -> Result<(), std::io::Error> {
        // Clear the previous data
        self.send_vec.clear();

        // Add the control signal type
        self.send_vec.push(0);

        // Helper function to convert f64 to u16 and append it to the vector
        let append_f64_as_u16 = |vec: &mut Vec<u8>, value: &mut f64| {
            // Ensure the value is within the valid range for u16
            if *value >= 0.0 && *value <= u16::MAX as f64 {
                // Safe to unwrap because we've already checked the range
                let bytes = (*value as u16).to_be_bytes();
                vec.extend_from_slice(&bytes);
            } else {
                vec.extend_from_slice(&0u16.to_be_bytes());
            }
        };

        // Append servo values to send_vec
        append_f64_as_u16(&mut self.send_vec, self.arm.servo_a_horiz());
        append_f64_as_u16(&mut self.send_vec, self.arm.servo_a_vert());
        append_f64_as_u16(&mut self.send_vec, self.arm.servo_b_horiz());
        append_f64_as_u16(&mut self.send_vec, self.arm.servo_b_vert());
        append_f64_as_u16(&mut self.send_vec, self.arm.servo_c_horiz());

        // Send the data
        self.udp_socket.send_to(&self.send_vec, &self.send_to)?;

        Ok(())
    }

    fn render_arm_status_ui(&mut self, ui: &mut Ui) {
        ui.add(Separator::default());
        ui.heading("Arm Status");
        let (mut i, mut j, mut k) = self.arm.get_ijk();
        ui.horizontal(|ui| {
            ui.label("i:");
            ui.add(DragValue::new(&mut i).speed(0.0).max_decimals(2));
            ui.add(Separator::default());
            ui.label("j:");
            ui.add(DragValue::new(&mut j).speed(0.0).max_decimals(2));
            ui.add(Separator::default());
            ui.label("k:");
            ui.add(DragValue::new(&mut k).speed(0.0).max_decimals(2));
        });

        ui.horizontal(|ui| {
            ui.label("target i:");
            ui.add(DragValue::new(&mut self.target_i).speed(0.1).max_decimals(2));
            ui.add(Separator::default());
            ui.label("target j:");
            ui.add(DragValue::new(&mut self.target_j).speed(0.1).max_decimals(2));
            ui.add(Separator::default());
            ui.label("target k:");
            ui.add(DragValue::new(&mut self.target_k).speed(0.1).max_decimals(2));
        });
        if ui.button("Apply").clicked() {
            //self.arm.calculate_inverse_kinematics(self.target_i, self.target_j, self.target_k, 0.0001);
        }
        // Controller::plot_arm(ui, 64.0);
    }

    fn render_settings(&mut self, ui: &mut Ui) {
        ui.heading("Settings");
        ui.label("Arm Lengths:");
        ui.horizontal(|ui| {
            ui.label("Upper Length:");
            ui.add(DragValue::new(self.arm.settable_arm_lengths().0).speed(0.05).max_decimals(3).suffix(" m").clamp_range(0.0..=100.0));
            ui.add(Separator::default());
            ui.label("Lower Length:");
            ui.add(DragValue::new(self.arm.settable_arm_lengths().1).speed(0.05).max_decimals(3).suffix(" m").clamp_range(0.0..=100.0));
        });
        let mut top_limit = *self.servo_top_range.end();
        let mut shoulder_limit = *self.servo_shoulder_range.end();
        let mut upper_limit = *self.servo_upper_range.end();
        let mut elbow_limit = *self.servo_elbow_range.end();
        let mut lower_limit = *self.servo_lower_range.end();
        let mut top_limit_lower = *self.servo_top_range.start();
        let mut shoulder_limit_lower = *self.servo_shoulder_range.start();
        let mut upper_limit_lower = *self.servo_upper_range.start();
        let mut elbow_limit_lower = *self.servo_elbow_range.start();
        let mut lower_limit_lower = *self.servo_lower_range.start();
        ui.label("\nArm Angle Upper Limits:");
        ui.horizontal(|ui| {
            // Temporary variables to hold the current upper limits

            ui.add(Separator::default());
            ui.label("Servo Top:");
            if ui.add(DragValue::new(&mut top_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_top_range = top_limit_lower..=top_limit;
            }
            ui.label("Servo Shoulder:");
            if ui.add(DragValue::new(&mut shoulder_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_shoulder_range = shoulder_limit_lower..=shoulder_limit;
            }
            ui.label("Servo Upper:");
            if ui.add(DragValue::new(&mut upper_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_upper_range = upper_limit_lower..=upper_limit;
            }
            ui.label("Servo Elbow:");
            if ui.add(DragValue::new(&mut elbow_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_elbow_range = elbow_limit_lower..=elbow_limit;
            }
            ui.label("Servo Lower:");
            if ui.add(DragValue::new(&mut lower_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_lower_range = lower_limit_lower..=lower_limit;
            }
        });
        ui.add(Separator::default());
        ui.label("Servo Top Upper Limit:");
        ui.horizontal(|ui| {
            // Temporary variables to hold the current upper limits
            ui.add(Separator::default());
            ui.label("Servo Top:");
            if ui.add(DragValue::new(&mut top_limit_lower).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_top_range = top_limit_lower..=top_limit;
            }
            ui.label("Servo Shoulder:");
            if ui.add(DragValue::new(&mut shoulder_limit_lower).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_shoulder_range = shoulder_limit_lower..=shoulder_limit;
            }
            ui.label("Servo Upper:");
            if ui.add(DragValue::new(&mut upper_limit_lower).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_upper_range = upper_limit_lower..=upper_limit;
            }
            ui.label("Servo Elbow:");
            if ui.add(DragValue::new(&mut elbow_limit_lower).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_elbow_range = elbow_limit_lower..=elbow_limit;
            }
            ui.label("Servo Lower:");
            if ui.add(DragValue::new(&mut lower_limit_lower).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_lower_range = lower_limit_lower..=lower_limit;
            }
        });
        ui.add(Separator::default());
        let mdns_label = ui.label("mDNS Service Address: (NON-FUNCTIONAL SETTING)");
        let mut shared_state_lock = self.shared_state.lock().unwrap();
        let mut mdns_address = shared_state_lock.service.clone();
        ui.text_edit_singleline(&mut mdns_address).labelled_by(mdns_label.id);
        if self.ip_addr_string != mdns_address{
            shared_state_lock.set_service(mdns_address)
        }
    }

    fn render_receive_ui(&mut self) {
        // Receive data
        match self.udp_socket.recv_from(&mut self.receive_vec) {
            Ok(_) => {}
            Err(_) => {}
        };
        for (i, chunk) in self.receive_vec.chunks_exact(2).enumerate() {
            let adc_value = u16::from_be_bytes(chunk.try_into().unwrap());
            // Process these !TODO
        }

    }

    fn mdns_button(ui: &mut Ui, sock: &mut String, shared_state: &Arc<Mutex<SharedState>>) {
        // Acquire the lock and immediately scope it to limit its duration
        let first_ip_option = {
            let shared_state_lock = shared_state.lock().unwrap();
            shared_state_lock.discovered_ips.first().cloned() // Clone the first IP if it exists
        };

        // Use the first IP option outside the lock scope
        match first_ip_option {
            Some(mdns_ip) => {
                if ui.button("mDNS").clicked() {
                    *sock = mdns_ip.to_string();
                }
            }
            None => {}
        }
    }


    fn render_plot(&mut self, ui: &mut Ui) {
        let w = 640;
        let h = 640;
        // Generate the buffer
        let mut buf: Vec<u8> = vec![0u8; w * h * 3];
        let image_data = generate_plot(&mut buf, w as u32, h as u32, &self.arm, self.plot_yaw, self.plot_scale);

        // Do the above but handle with a match
        match image_data {
            Ok(_) => {
                let image = ColorImage::from_rgb([w, h], &mut buf);

                // Thanks to https://github.com/emilk/egui/discussions/3431
                // you must keep the handle, if the handle is destroyed so the texture will be destroyed as well
                let handle = ui.ctx().load_texture("Arm Positions", image.clone(), TextureOptions::default());
                let sized_image = egui::load::SizedTexture::new(handle.id(), vec2(*&image.size[0] as f32, *&image.size[1] as f32));
                let image = Image::from_texture(sized_image);
                ui.add(image);
                ui.add(Slider::new(&mut self.plot_yaw, 0.0..=5.0).text("Yaw"));
                ui.add(Slider::new(&mut self.plot_scale, 0.0..=2.0).text("Scale"));
            }
            Err(e) => {
                ui.label(format!("Unable to get Image Data: {}", e));
            }
        }
    }
}




