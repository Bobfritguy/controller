use crate::network;
use eframe::egui;
use crate::arm;
use std::net::UdpSocket;
use std::io::ErrorKind;
use std::ops::{Range, RangeInclusive};
use std::sync::{Arc, Mutex};
use eframe::egui::Ui;
use crate::models::{SharedState, Mode};

pub struct Controller {
    ip_addr_string: String,
    is_ip_addr: bool,
    send_to: String,
    udp_socket: UdpSocket,
    servo_top_range: RangeInclusive<u16>,
    servo_shoulder_range: RangeInclusive<u16>,
    servo_upper_range: RangeInclusive<u16>,
    servo_elbow_range: RangeInclusive<u16>,
    servo_lower_range: RangeInclusive<u16>,
    send_vec: Vec<u8>,
    receive_vec: Vec<u8>,
    mode: Mode,
    send: bool,
    flag: bool,
    shared_state: Arc<Mutex<SharedState>>,
    arm: arm::Arm,
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
            servo_top_range: 0..=180,
            servo_shoulder_range: 0..=180,
            servo_upper_range: 0..=180,
            servo_elbow_range: 0..=180,
            servo_lower_range: 0..=180,
            send_vec: Vec::new(),
            receive_vec: Vec::new(),
            mode: Mode::Stopped,
            send: true,
            flag: true,
            shared_state,
            arm: arm::Arm::new(1.0, 1.0),
        }
    }

    pub fn render_ui(&mut self, ui: &mut egui::Ui) {
        // Heading
        ui.heading("Limb Controller");

        // IP Address input and validation
        ui.horizontal(|ui| {
            let name_label = ui.label("IP Address: ");
            ui.text_edit_singleline(&mut self.ip_addr_string).labelled_by(name_label.id);
            Controller::mdns_button(ui, &mut self.ip_addr_string, &self.shared_state);
            self.is_ip_addr = network::check_ip_string(&self.ip_addr_string);
            ui.label(if self.is_ip_addr { "Valid" } else { "Invalid" });
            if self.is_ip_addr {
                if ui.button("Apply").clicked() {
                    self.send_to = self.ip_addr_string.clone();
                }
            }
        });

        // Mode selection
        ui.add(egui::Separator::default());
        egui::ComboBox::from_label("Choose Mode")
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
                self.render_arm_status_ui(ui);
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
            ui.add(egui::Separator::default());
            ui.label(if self.send { "Auto Send" } else { "Manual Send" });
        });

        if !self.send {
            if ui.button("Send").clicked() {
                self.send_data().expect("Failed to send data");
            }
        } else if self.flag{
            self.send_data().expect("Failed to send data");
            self.flag = false;
        }
    }

    fn render_servo_control(ui: &mut egui::Ui, range: &RangeInclusive<u16>, angle: &mut u16, label: &str, flag: &mut bool) {
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
        });
        ui.end_row(); // End the current row and prepare for the next
    }

    fn flag_setting_slider(
        ui: &mut egui::Ui,
        value: &mut u16,
        range: std::ops::RangeInclusive<u16>,
        suffix: &str,
        flag: &mut bool,
    ) {
        let slider_response = ui.add(egui::Slider::new(value, range).suffix(suffix));
        if slider_response.changed() {
            *flag = true; // Set the flag when the slider is used.
        }
    }

    // Code for egui toggle switch.
    fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
        let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool(response.id, *on);
            let visuals = ui.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
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
    fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| Controller::toggle_ui(ui, on)
    }

    fn send_data(&mut self) -> Result<(), std::io::Error> {
        // Send the data
        self.send_vec.clear();
        self.send_vec.push(0);
        self.send_vec.push(self.arm.servo_a_horiz().to_be_bytes()[0]);
        self.send_vec.push(self.arm.servo_a_horiz().to_be_bytes()[1]);
        self.send_vec.push(self.arm.servo_a_vert().to_be_bytes()[0]);
        self.send_vec.push(self.arm.servo_a_vert().to_be_bytes()[1]);
        self.send_vec.push(self.arm.servo_b_horiz().to_be_bytes()[0]);
        self.send_vec.push(self.arm.servo_b_horiz().to_be_bytes()[1]);
        self.send_vec.push(self.arm.servo_b_vert().to_be_bytes()[0]);
        self.send_vec.push(self.arm.servo_b_vert().to_be_bytes()[1]);
        self.send_vec.push(self.arm.servo_c_horiz().to_be_bytes()[0]);
        self.send_vec.push(self.arm.servo_c_horiz().to_be_bytes()[1]);
        self.udp_socket.send_to(&self.send_vec, &self.send_to)?;

        Ok(())
    }
    fn render_arm_status_ui(&self, ui: &mut Ui) {
        ui.add(egui::Separator::default());
        ui.heading("Arm Status");
        let (mut i, mut j, mut k) = self.arm.get_ijk();
        ui.horizontal(|ui| {
            ui.label("i:");
            ui.add(egui::DragValue::new(&mut i).speed(0.0).max_decimals(2));
            ui.add(egui::Separator::default());
            ui.label("j:");
            ui.add(egui::DragValue::new(&mut j).speed(0.0).max_decimals(2));
            ui.add(egui::Separator::default());
            ui.label("k:");
            ui.add(egui::DragValue::new(&mut k).speed(0.0).max_decimals(2));
        });
    }

    fn render_settings(&mut self, ui: &mut Ui) {
        ui.heading("Settings");
        ui.label("Arm Lengths:");
        ui.horizontal(|ui| {
            ui.label("Upper Length:");
            ui.add(egui::DragValue::new(self.arm.settable_arm_lengths().0).speed(0.05).max_decimals(3).suffix(" cm").clamp_range(0.0..=100.0));
            ui.add(egui::Separator::default());
            ui.label("Lower Length:");
            ui.add(egui::DragValue::new(self.arm.settable_arm_lengths().1).speed(0.05).max_decimals(3).suffix(" cm").clamp_range(0.0..=100.0));
        });
        ui.label("\nArm Angle Limits:");
        ui.horizontal(|ui| {
            // Temporary variables to hold the current upper limits
            let mut top_limit = *self.servo_top_range.end();
            let mut shoulder_limit = *self.servo_shoulder_range.end();
            let mut upper_limit = *self.servo_upper_range.end();
            let mut elbow_limit = *self.servo_elbow_range.end();
            let mut lower_limit = *self.servo_lower_range.end();
            ui.add(egui::Separator::default());
            ui.label("Servo Top:");
            if ui.add(egui::DragValue::new(&mut top_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_top_range = 0..=top_limit;
            }
            ui.label("Servo Shoulder:");
            if ui.add(egui::DragValue::new(&mut shoulder_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_shoulder_range = 0..=shoulder_limit;
            }
            ui.label("Servo Upper:");
            if ui.add(egui::DragValue::new(&mut upper_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_upper_range = 0..=upper_limit;
            }
            ui.label("Servo Elbow:");
            if ui.add(egui::DragValue::new(&mut elbow_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_elbow_range = 0..=elbow_limit;
            }
            ui.label("Servo Lower:");
            if ui.add(egui::DragValue::new(&mut lower_limit).speed(1).suffix("°").clamp_range(0..=270)).changed() {
                self.servo_lower_range = 0..=lower_limit;
            }
        });
    }

    fn mdns_button(ui: &mut Ui, ip: &mut String, shared_state: &Arc<Mutex<SharedState>>) {
        let mut shared_state_lock = shared_state.lock().unwrap();
        match shared_state_lock.discovered_ips.first()
            .map(|ip| ip.to_string()){
            Some(mdns_ip) => {
                if ui.button("mDNS").clicked(){
                    *ip = mdns_ip
                }
            }
            None => {}
        }

    }

}


