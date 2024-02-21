use std::error::Error;
use std::f64;

use approx;
use nalgebra as na;
use nalgebra::Point;


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

        // Get the position of point b, the elbow, if it lay on the ik plane, assuming every other angle is 0
        self.b = self.get_b_2d();

        // now, using the position of the elbow, calculate the position of the end effector, c
        self.c = self.get_c_2d();
        // Now we rotate the end about the ab axis by the angle h2, to get the current position of the end effector, taking into account
        // only v1, v2, and h2.

        // Perform the rotation about the AB axis, representing the rotation of h2
        let normal_ab = na::Unit::new_normalize(*&self.b);
        let mut rot = na::Rotation3::from_axis_angle(&normal_ab, self.h2.to_radians());
        self.c = rot * self.c;

        // Perform the rotation about the i axis, representing the rotation of v1.
        let mut rot = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), self.h1.to_radians());

        self.b = rot * self.b;
        self.c = rot * self.c;
    }

    fn get_b_2d(&self) -> na::Vector3<f64> {
        // Calculate the position of point b, the elbow, if it lay on the ik plane
        // This is simple since we can assume it is only affected by v1, we assume h1 is 0.

        // Firstly we get i
        let i = self.u * self.v1.to_radians().sin();
        // j is 0
        let j = 0.0;
        let k = self.u * self.v1.to_radians().cos();

        na::Vector3::new(i, j, k)
    }
    fn get_c_2d(&self) -> na::Vector3<f64> {
        // Calculate the position of point c, the end effector, if it lay on the ij plane
        // More complicated than the elbow since it is affected by both v1 and v2.


        // Firstly we get i, which will be the same process as for the elbow, but we then add the elbow's i to it.
        let i = self.b.x + self.l * self.v2.to_radians().sin();
        // j is 0
        let j = 0.0;
        // k is the same process as for i, but we then add the elbow's k to it.
        let k = self.b.z + self.l * self.v2.to_radians().cos();

        na::Vector3::new(i, j, k)
        // this is the value of c if everything is 0, except for v1 and v2.
    }

    pub fn update_servo_values(&mut self) {
        self.sav = (self.v1) as u16;
        self.sbv = (self.v2) as u16;
        self.sah = (self.h1) as u16;
        self.sbh = (self.h2) as u16;
        self.sch = (self.h3) as u16;
    }

    pub fn update_angle_values(&mut self) {
        self.v1 = self.sav as f64;
        self.v2 = self.sbv as f64;
        self.h1 = self.sah as f64;
        self.h2 = self.sbh as f64;
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

    fn go_to_middle(&mut self) {
        self.sav = 90;
        self.sbv = 90;
        self.sah = 90;
        self.sbh = 90;
        self.sch = 90;
    }

    pub fn update(&mut self) {
        self.update_angle_values();
        self.calculate_values();
    }
}


