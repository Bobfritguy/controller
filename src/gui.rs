use eframe::egui::{self, CentralPanel, Context};
use std::sync::{Arc, Mutex};
use crate::controller::Controller;
use crate::models::SharedState;


pub struct Gui {
    controller: Controller, // Assuming Controller has necessary UI related methods
}

impl Gui {
    pub fn  new(shared_state: Arc<Mutex<SharedState>>) -> Self {

        let controller = Controller::new(shared_state);

        Gui {
            controller,
        }
    }

    // The main update loop for the UI
    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.controller.render_ui(ui);
        });
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.update(ctx);
    }
}