use std::cell::RefCell;
use std::rc::Rc;
use gtk4::glib;

#[derive(Clone, Copy, Debug)]
pub struct SpringConfig {
    pub damping_ratio: f64,
    pub stiffness: f64,
    pub mass: f64,
    pub epsilon: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            damping_ratio: 0.9,
            stiffness: 180.0,
            mass: 1.0,
            epsilon: 0.1,
        }
    }
}

#[derive(Clone)]
pub struct SpringAnimator {
    config: SpringConfig,
    current_val: Rc<RefCell<f64>>,
    current_vel: Rc<RefCell<f64>>,
    target_val: Rc<RefCell<f64>>,
    source_id: Rc<RefCell<Option<glib::SourceId>>>,
}

impl SpringAnimator {
    pub fn new(initial_value: f64, config: SpringConfig) -> Self {
        Self {
            config,
            current_val: Rc::new(RefCell::new(initial_value)),
            current_vel: Rc::new(RefCell::new(0.0)),
            target_val: Rc::new(RefCell::new(initial_value)),
            source_id: Rc::new(RefCell::new(None)),
        }
    }

    pub fn value(&self) -> f64 {
        *self.current_val.borrow()
    }

    pub fn set_value_immediate(&self, val: f64) {
        if let Some(src) = self.source_id.borrow_mut().take() {
            src.remove();
        }
        *self.current_val.borrow_mut() = val;
        *self.target_val.borrow_mut() = val;
        *self.current_vel.borrow_mut() = 0.0;
    }

    pub fn animate_to<F>(&self, target: f64, mut on_update: F)
    where
        F: FnMut(f64) + 'static,
    {
        *self.target_val.borrow_mut() = target;

        if let Some(src) = self.source_id.borrow_mut().take() {
            src.remove();
        }

        let current_val = self.current_val.clone();
        let current_vel = self.current_vel.clone();
        let target_val = self.target_val.clone();
        let source_id = self.source_id.clone();
        let cfg = self.config;

        let src = glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            let dt = 0.010; // 10ms frame step
            let x = *current_val.borrow();
            let v = *current_vel.borrow();
            let x_target = *target_val.borrow();

            // Spring differential equation: F = -k*(x - x_target) - c*v
            let c = 2.0 * cfg.damping_ratio * (cfg.stiffness * cfg.mass).sqrt();
            let f = -cfg.stiffness * (x - x_target) - c * v;
            let a = f / cfg.mass;

            let new_v = v + a * dt;
            let new_x = x + new_v * dt;

            *current_val.borrow_mut() = new_x;
            *current_vel.borrow_mut() = new_v;

            on_update(new_x);

            if (new_x - x_target).abs() < cfg.epsilon && new_v.abs() < cfg.epsilon {
                *current_val.borrow_mut() = x_target;
                *current_vel.borrow_mut() = 0.0;
                on_update(x_target);
                *source_id.borrow_mut() = None;
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });

        *self.source_id.borrow_mut() = Some(src);
    }
}
