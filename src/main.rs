#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::io::ErrorKind;
use eframe::egui;
use std::net::UdpSocket;


const SERVO_ANGLE: u32 = 180;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1920.0, 1080.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Robotic Limb Controller",
        options,
        Box::new(|_cc| {
            Box::<Controller>::default()
        }),
    )
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Mode {
    Sending,
    Receiving,
    Stopped,
}


struct Controller {
    ip_addr_string: String,
    is_ip_addr: bool,
    send_to: String,
    udp_socket: UdpSocket,
    servo_shoulder_pos: u32,
    servo_top_pos: u32,
    stepper_pos: i32,
    send_vec: Vec<u8>,
    receive_vec: Vec<u8>,
    mode: Mode,
    send: bool,
}

impl Default for Controller {
    fn default() -> Self {
        // Create and configure the UdpSocket
        let udp_socket = UdpSocket::bind("0.0.0.0:8080").expect("Failed to bind socket");
        udp_socket.set_nonblocking(true).expect("Failed to set nonblocking");

        // Create the Controller with the configured UdpSocket
        Self {
            ip_addr_string: "0.0.0.0:1234".to_owned(),
            is_ip_addr: true,
            send_to: "0.0.0.0:1234".to_owned(),
            udp_socket,
            servo_shoulder_pos: 0,
            servo_top_pos: 0,
            stepper_pos: 0,
            send_vec: Vec::new(),
            receive_vec: Vec::new(),
            mode: Mode::Stopped,
            send: false,
        }

    }
}



impl eframe::App for Controller {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(
            ctx,
            |ui| {
                ui.heading("Limb Controller");
                ui.horizontal(|ui| {
                    // Create a string from the local IP and Port of the socket
                    let name_label = ui.label("IP Address: ");
                    ui.text_edit_singleline(&mut self.ip_addr_string)
                        .labelled_by(name_label.id);

                    self.is_ip_addr = check_ip_string(&self.ip_addr_string);

                    ui.label(if self.is_ip_addr { "Valid" } else { "Invalid" });
                    if self.is_ip_addr {
                        if ui.button("Apply").clicked()  {
                            self.send_to = self.ip_addr_string.clone();
                        }
                    }

                });

                ui.add(egui::Separator::default());


                egui::ComboBox::from_label("Choose Mode")
                    .selected_text(format!("{0:?}", self.mode))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(40.0);
                        ui.selectable_value(&mut self.mode, Mode::Sending, "Sending");
                        ui.selectable_value(&mut self.mode, Mode::Receiving, "Receiving");
                        ui.selectable_value(&mut self.mode, Mode::Stopped, "Stopped");
                    });


                match self.mode {
                    Mode::Sending => {
                        ui.label(format!("Destination Address: {} \n", self.send_to));
                        ui.horizontal(|ui| {
                            ui.add(toggle(&mut self.send));
                            ui.add(egui::Separator::default());
                            ui.label(if self.send { "Sending" } else { "Not Sending" });
                        });

                        if self.send {
                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut self.servo_shoulder_pos, 0..=SERVO_ANGLE).text("Servo Shoulder Position"));
                                if ui.button("Move back a degree").clicked() {
                                    if self.servo_shoulder_pos > 0 {
                                        self.servo_shoulder_pos -= 1;
                                    }
                                }
                                if ui.button("Move forward a degree").clicked() {
                                    if self.servo_shoulder_pos < SERVO_ANGLE {
                                        self.servo_shoulder_pos += 1;
                                    }
                                }

                            });

                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut self.servo_top_pos, 0..=SERVO_ANGLE).text("Servo Top Position"));
                                if ui.button("Move back a degree").clicked() {
                                    if self.servo_top_pos > 0 {
                                        self.servo_top_pos -= 1;
                                    }
                                }
                                if ui.button("Move forward a degree").clicked() {
                                    if self.servo_top_pos < SERVO_ANGLE {
                                        self.servo_top_pos += 1;
                                    }
                                }

                            });

                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.stepper_pos).speed(1.0).clamp_range(-1000..=1000).prefix("Stepper Position: "));
                                if ui.button("Step Counter-Clockwise").clicked() {
                                    self.stepper_pos -= 1;
                                }
                                if ui.button("Step Clockwise").clicked() {
                                    self.stepper_pos += 1;
                                }
                            });
                            // Send the data
                            self.send_vec.clear();
                            self.send_vec.push(0);
                            self.send_vec.push((self.servo_shoulder_pos >> 8) as u8);
                            self.send_vec.push(self.servo_shoulder_pos as u8);
                            self.send_vec.push((self.servo_top_pos >> 8) as u8);
                            self.send_vec.push(self.servo_top_pos as u8);
                            self.send_vec.push((self.stepper_pos >> 24) as u8);
                            self.send_vec.push((self.stepper_pos >> 16) as u8);
                            self.send_vec.push((self.stepper_pos >> 8) as u8);
                            self.send_vec.push(self.stepper_pos as u8);

                            self.udp_socket.send_to(&self.send_vec, &self.send_to).expect("Failed to send data");

                            let mut buf = [0u8; 1024]; // You can use a fixed-size array as buffer

                            match self.udp_socket.recv_from(&mut buf) {192.168.
                                Ok((amt, _src)) => {
                                    // Received data
                                    self.receive_vec.clear();
                                    self.receive_vec.extend_from_slice(&buf[0..amt]);
                                },
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                    // Would block

                                },
                                Err(e) => {
                                    // An actual error occurred
                                    // Handle this error as appropriate in your application context
                                    ui.label(format!("An error occurred: {}", e));
                                }
                            }

                            if self.receive_vec.len() > 0 {
                                // make a string of all the elements of the vec
                                // Combine the two received bytes into a u16
                                // let vec_num = [self.receive_vec[0] as u16];
                                let vec_string = self.receive_vec.iter().map(|b| format!("{}", b)).collect::<Vec<_>>().join(" ");
                                ui.label(format!("Received {} from {}", vec_string, self.send_to));
                            } else {
                                ui.label(format!("Waiting for data from {}", self.send_to));
                            }

                        }

                    },
                    Mode::Receiving => {
                        //TODO: Receive data
                    },
                    Mode::Stopped => {
                        ui.label("Stopped");
                    },
                }
            }
        );
    }
}


fn check_ip_string(ip_string: &String) -> bool{
    let default: String = "0.0.0.0:1234".to_string();
    let ip_max_length = "000.000.000.000:0000".to_string().len();
    let ip_min_length = default.len();
    if (ip_string.len() > ip_max_length) || (ip_string.len() < ip_min_length) {
        return false;
    }

    // Split into IP and Port
    let ip_port: Vec<&str> = ip_string.split(":").collect();
    if ip_port.len() != 2 {
        return false;
    }
    let ip = ip_port[0];
    let port = ip_port[1];

    // Convert the port to an int
    if port.len() != 4 {
        return false;
    }

    let _: u16 = match port.parse() {
        Ok(n) => {
            if n < 1{
                return false;
            }
            n
        },
        Err(_) => return false,
    };

    // Split the IP into octets
    let octets: Vec<&str> = ip.split(".").collect();
    if octets.len() != 4 {
        return false;
    }

    // Convert the octets to u8
    for octet in octets {
        match octet.parse::<u8>() {
            Ok(_) =>  continue,
            Err(_) => return false,
        };
    }

    true
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
pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}
