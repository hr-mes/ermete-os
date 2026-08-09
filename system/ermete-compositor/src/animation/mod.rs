pub mod magic_lamp;
pub mod manager;
pub mod solver;
pub mod spring;
pub mod wobbly;

pub use magic_lamp::{GenieSlice, MagicLampAnimator, MagicLampConfig, MagicLampState};
pub use manager::AnimationEngine;
pub use solver::{MassSpringDamperSolver, SpringConfig};
pub use spring::{Spring1D, WindowSpringAnimator};
pub use wobbly::{WobblyConfig, WobblyNode, WobblyWindowAnimator};

