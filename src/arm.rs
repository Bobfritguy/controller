use std::f64;

use approx;
use nalgebra as na;

pub struct Arm {
    u: f64,
    l: f64,
    v1: f64,
    v2: f64,
    h1: f64,
    h2: f64,
    h3: f64,
    sav: u16,
    sah: u16,
    sbv: u16,
    sbh: u16,
    sch: u16,
    a: na::Vector3<f64>,
    b: na::Vector3<f64>,
    c: na::Vector3<f64>,
}

impl Arm {
    pub fn new(u: f64, l: f64) -> Arm {
        let mut arm = Arm {
            u,
            l,
            v1: 90.0,
            v2: 90.0,
            h1: 90.0,
            h2: 90.0,
            h3: 90.0,
            sav: 90,
            sah: 90,
            sbv: 90,
            sbh: 90,
            sch: 90,
            a: na::Vector3::new(0.0, 0.0, 0.0),
            b: na::Vector3::new(u, 0.0, 0.0),
            c: na::Vector3::new((u + l), 0.0, 0.0),
        };
        arm.calculate_values();
        arm.update_servo_values();
        arm
    }

    fn calculate_values(&mut self) {
        // generic initial state

        self.b = Arm::get_b_2d(self.u, self.v1);
        self.c = Arm::get_c_2d(self.l, self.v2, self.v1, self.b);

        // Perform the rotation about the i axis
        let mut rot = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), self.h1.to_radians());

        self.b = rot * self.b;
        self.c = rot * self.c;

        // Perform the rotation about the AB axis
        let normal_ab = na::Unit::new_normalize(*&self.b);
        let mut rot = na::Rotation3::from_axis_angle(&normal_ab, self.h2.to_radians());

        self.c = rot * self.c;


    }

    fn get_c_2d(l: f64, v2: f64, v1: f64, b: na::Vector3<f64>) -> na::Vector3<f64> {
        let i = l * (v1 + v2).to_radians().cos() + b.x;
        let j = l * (v1 + v2).to_radians().sin() + b.y;
        let k = 0.0;
        na::Vector3::new(i, j, k)
    }

    fn get_b_2d(u: f64, v1: f64) -> na::Vector3<f64> {
        let i = u * v1.to_radians().cos();
        let j = u * v1.to_radians().sin();
        let k = 0.0;

        na::Vector3::new(i, j, k)
    }

    pub fn update_servo_values(&mut self) {
        self.sav = (self.v1 + 90.0) as u16;
        self.sbv = (self.v2 + 90.0) as u16;
        self.sah = (self.h1 + 90.0) as u16;
        self.sbh = (self.h2 + 90.0) as u16;
        self.sch = (self.h3 + 90.0) as u16;
    }

    pub fn update_angle_values(&mut self) {
        self.v1 = self.sav as f64 - 90.0;
        self.v2 = self.sbv as f64 - 90.0;
        self.h1 = self.sah as f64 - 90.0;
        self.h2 = self.sbh as f64 - 90.0;
    }

    pub fn get_servo_values(&self) -> (u16, u16, u16, u16) {
        (self.sav, self.sbv, self.sah, self.sbh)
    }

    pub fn to_string(&self) -> String {
        format!("A: {:?}\nB: {:?}\nC: {:?}\nServo values: ({}, {}, {}, {})", self.a, self.b, self.c, self.sav, self.sbv, self.sah, self.sbh)
    }

    pub fn get_ijk(&self) -> (f64, f64, f64) {
        (self.c.x, self.c.y, self.c.z)
    }

    pub fn settable_arm_lengths(&mut self) -> (&mut f64, &mut f64) {
        (&mut self.u, &mut self.l)
    }


    // Function for servo a vertical
    pub fn servo_a_vert(&mut self) -> &mut u16 {
        &mut self.sav
    }

    // Function for servo b vertical
    pub fn servo_b_vert(&mut self) -> &mut u16 {
        &mut self.sbv
    }

    // Function for servo a horizontal
    pub fn servo_a_horiz(&mut self) -> &mut u16 {
        &mut self.sah
    }

    // Function for servo b horizontal
    pub fn servo_b_horiz(&mut self) -> &mut u16 {
        &mut self.sbh
    }

    // Function for servo c horizontal
    pub fn servo_c_horiz(&mut self) -> &mut u16 {
        &mut self.sch
    }

    pub fn update(&mut self) {
        self.update_angle_values();
        self.calculate_values();
    }
}


