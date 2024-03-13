use std::error::Error;
use std::f64;

use approx;
use nalgebra as na;
use nalgebra::Point;


pub struct Arm {
    u: f64,
    l: f64,
    shoulder_servo: f64,
    elbow_servo: f64,
    top_servo: f64,
    upper_arm_servo: f64,
    lower_arm_servo: f64,
    a: na::Vector3<f64>,
    b: na::Vector3<f64>,
    c: na::Vector3<f64>,
}

impl Arm {
    pub fn new(u: f64, l: f64) -> Arm {
        let mut arm = Arm {
            u,
            l,
            shoulder_servo: 40.0,
            elbow_servo: 0.0,
            top_servo: 95.0,
            upper_arm_servo: 90.0,
            lower_arm_servo: 90.0,
            a: na::Vector3::new(0.0, 0.0, 0.0), // Shoulder (origin)
            b: na::Vector3::new(0.0, 0.0, 0.0), // Elbow
            c: na::Vector3::new(0.0, 0.0, 0.0), // End effector
        };
        arm.calculate_values();
        arm
    }

    fn calculate_values(&mut self) {
        /*
        Last recorded home position:
        Top: 95
        Shoulder: 40
        Upper_Rotation: 90
        Elbow: 0
        Lower_Rotation: N/A
         */

        // Initial State with all angles at 0
        let mut dummy_elbow = na::Vector3::new(0.0, 0.0, self.u);

        // perform the rotation of the elbow, representing the elbow servo, v2
        // This is done be translating the end effector to imagine the elbow is at to the origin,
        // rotating it about the i axis, and then moving it back to its original position.
        let mut dummy_end = na::Vector3::new(0.0, 0.0, self.l); // the end effector, minus the length of the lower arm
        let mut rot = na::Rotation3::from_axis_angle(&na::Vector3::y_axis(), self.elbow_servo.to_radians()); // the negative is because the servo is rotating clockwise
        dummy_end = rot * dummy_end;
        // now bring back to the correct position
        dummy_end = na::Vector3::new(dummy_end.x, dummy_end.y, dummy_end.z + self.u);

        // Represent the rotation of the upper arm, h2, by rotating about the k axis
        rot = na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), -self.upper_arm_servo.to_radians()); // negative for clockwise rotation
        dummy_end = rot * dummy_end;

        // perform a rotation about the j axis, representing the rotation of the shoulder servo, v1
        rot = na::Rotation3::from_axis_angle(&na::Vector3::y_axis(), self.shoulder_servo.to_radians()); // Counter clockwise so positive
        dummy_end = rot * dummy_end;
        dummy_elbow = rot * dummy_elbow;

        // // perform the rotation about the i, representing the rotation of the top servo, h1
        // rot = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), self.top_servo.to_radians());
        // dummy_end = rot * dummy_end;
        // dummy_elbow = rot * dummy_elbow;
        //
        // // Now we have the position of the end effector, c, and the elbow, b, in the 3d space
        self.b = dummy_elbow;
        self.c = dummy_end;

        // Now perform the rotation of the
        // // Get the position of point b, the elbow, if it lay on the ik plane, assuming every other angle is 0
        // self.b = self.get_b_2d();
        //
        // // now, using the position of the elbow, calculate the position of the end effector, c
        // self.c = self.get_c_2d();
        // // Now we rotate the end about the ab axis by the angle h2, to get the current position of the end effector, taking into account
        // // only v1, v2, and h2.
        //
        // // Perform the rotation about the AB axis, representing the rotation of h2
        // let normal_ab = na::Unit::new_normalize(*&self.b);
        // let mut rot = na::Rotation3::from_axis_angle(&normal_ab, self.h2.to_radians());
        // self.c = rot * self.c;
        //
        // // Perform the rotation about the i axis, representing the rotation of v1.
        // let mut rot = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), self.h1.to_radians());
        //
        // self.b = rot * self.b;
        // self.c = rot * self.c;
    }

    fn get_b_2d(&self) -> na::Vector3<f64> {
        // Calculate the position of point b, the elbow, if it lay on the ik plane
        // This is simple since we can assume it is only affected by v1, we assume h1 is 0.

        // Firstly we get i
        let i = self.u * self.shoulder_servo.to_radians().sin();
        // j is 0
        let j = 0.0;
        let k = self.u * self.shoulder_servo.to_radians().cos();

        na::Vector3::new(i, j, k)
    }
    fn get_c_2d(&self) -> na::Vector3<f64> {
        // Calculate the position of point c, the end effector, if it lay on the ij plane
        // More complicated than the elbow since it is affected by both v1 and v2.


        // Firstly we get i, which will be the same process as for the elbow, but we then add the elbow's i to it.
        let i = self.b.x + self.l * self.elbow_servo.to_radians().sin();
        // j is 0
        let j = 0.0;
        // k is the same process as for i, but we then add the elbow's k to it.
        let k = self.b.z + self.l * self.elbow_servo.to_radians().cos();

        na::Vector3::new(i, j, k)
        // this is the value of c if everything is 0, except for v1 and v2.
    }

    pub fn get_ijk(&self) -> (f64, f64, f64) {
        (self.c.x, self.c.y, self.c.z)
    }

    pub fn get_elbow(&self) -> (f64, f64, f64) {
        (self.b.x, self.b.y, self.b.z)
    }
    pub fn settable_arm_lengths(&mut self) -> (&mut f64, &mut f64) {
        (&mut self.u, &mut self.l)
    }


    // Function for servo a vertical
    pub fn servo_a_vert(&mut self) -> &mut f64 {
        &mut self.shoulder_servo
    }

    // Function for servo b vertical
    pub fn servo_b_vert(&mut self) -> &mut f64 {
        &mut self.elbow_servo
    }

    // Function for servo a horizontal
    pub fn servo_a_horiz(&mut self) -> &mut f64 {
        &mut self.top_servo
    }

    // Function for servo b horizontal
    pub fn servo_b_horiz(&mut self) -> &mut f64 {
        &mut self.upper_arm_servo
    }

    // Function for servo c horizontal
    pub fn servo_c_horiz(&mut self) -> &mut f64 {
        &mut self.lower_arm_servo
    }

    fn go_to_middle(&mut self) {
        self.shoulder_servo = 90.0;
        self.elbow_servo = 90.0;
        self.top_servo = 90.0;
        self.upper_arm_servo = 90.0;
        self.lower_arm_servo = 90.0;

    }

    pub fn update(&mut self) {
        self.calculate_values();
    }

    pub fn angle_from_point(&mut self, i: f64, j: f64, k: f64) -> (f64, f64, f64){
        //TODO: Implement this function
        (0.0, 0.0, 0.0)
    }
}


