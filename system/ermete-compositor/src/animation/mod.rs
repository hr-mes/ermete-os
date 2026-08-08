pub mod manager;
pub mod solver;
pub mod spring;

pub use manager::AnimationEngine;
pub use solver::{MassSpringDamperSolver, SpringConfig};
pub use spring::{Spring1D, WindowSpringAnimator};
