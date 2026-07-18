/// PID Controller class

pub struct PID {
    kp: f32,
    kd: f32,
    ki: f32,
    prev_err: f32,
    integral: f32,
}

impl PID {
    pub fn new(kp: f32, kd: f32, ki: f32) -> Self {
        PID {
            kp,
            kd,
            ki,
            prev_err: 0.0,
            integral: 0.0
        }
    }

    /** Runs a simple PID controller
        @breif takes in a setpoint and measured value and returns the computed value off one pass of the pid
        @param dt, time between each pass of the loop
        @param setpoint, where you want to go
        @param measured, where you are */
    pub fn compute(&mut self, dt: f32, setpoint: f32, measured: f32) -> f32 {

        let err = setpoint - measured;

        // Proportional
        let proportional = err * self.kp;

        // Integral
        let integral = self.integral * self.ki;
        self.integral += err * dt; // Add to the integator

        // Derivative
        let de = (err - self.prev_err) / dt;
        self.prev_err = err;
        let derivative = self.kd * de;

        proportional + integral + derivative

    }
}