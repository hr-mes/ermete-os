use crate::ecs::components::{Position, TargetPosition, Velocity};
use crate::ecs::world::{SharedEcsWorld, World};
use anyhow::Result;

/// Standard time step for 1000Hz compositor animation tick (1 millisecond = 0.001 seconds).
pub const DT_1000HZ: f32 = 0.001;

/// Positional threshold below which micro-oscillations cease and target is snapped (pixels).
pub const SLEEP_DISTANCE_THRESHOLD: f32 = 0.001;

/// Velocity threshold below which movement stops (pixels per second).
pub const SLEEP_VELOCITY_THRESHOLD: f32 = 0.001;

/// 1000Hz Mass-Spring-Damper Physics System.
///
/// Iterates over all entities possessing `Position`, `Velocity`, and `TargetPosition` components.
/// Executes under a single write lock acquisition to achieve zero lock contention.
///
/// Damped Harmonic Motion Physics Equations:
///   F_spring  = -k * (x - target_x)
///   F_damping = -c * vx
///   F_total   = F_spring + F_damping
///   a = F_total / m
///
/// Updates `Velocity` and `Position` in-place using semi-implicit Euler integration per 1ms tick.
pub fn spring_physics_system(world: &SharedEcsWorld, dt: f32) -> Result<()> {
    let mut world_guard = world.write()?;
    spring_physics_system_batch(&mut world_guard, dt);
    Ok(())
}

/// Core batch physics computation function.
/// Takes mutable reference to `World` to batch component references in contiguous memory without repeated locks.
pub fn spring_physics_system_batch(world: &mut World, dt: f32) {
    // Safety cap on dt to prevent integration divergence during micro-stalls
    let dt_clamped = dt.min(0.005);

    world.for_each_mut_3::<Position, Velocity, TargetPosition, _>(
        |_entity, pos, vel, target| {
            let dx = pos.x - target.x;
            let dy = pos.y - target.y;
            let distance_sq = dx * dx + dy * dy;
            let speed_sq = vel.vx * vel.vx + vel.vy * vel.vy;

            // Sleeping optimization: snap to target position if within threshold and practically stationary
            if distance_sq < SLEEP_DISTANCE_THRESHOLD * SLEEP_DISTANCE_THRESHOLD
                && speed_sq < SLEEP_VELOCITY_THRESHOLD * SLEEP_VELOCITY_THRESHOLD
            {
                pos.x = target.x;
                pos.y = target.y;
                vel.vx = 0.0;
                vel.vy = 0.0;
                return;
            }

            // Damped harmonic motion force calculation:
            // F_x = -k * (x - target_x) - c * vx
            // F_y = -k * (y - target_y) - c * vy
            let fx = -target.stiffness * dx - target.damping * vel.vx;
            let fy = -target.stiffness * dy - target.damping * vel.vy;

            let mass = target.mass.max(0.001);
            let ax = fx / mass;
            let ay = fy / mass;

            // Semi-implicit Euler integration for 1000Hz precision
            vel.vx += ax * dt_clamped;
            vel.vy += ay * dt_clamped;

            pos.x += vel.vx * dt_clamped;
            pos.y += vel.vy * dt_clamped;
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::{Position, TargetPosition, Velocity};
    use crate::ecs::world::World;

    #[test]
    fn test_spring_physics_system_batch_convergence() {
        let mut world = World::new();
        let entity = world.spawn();

        world.add_component(entity, Position::new(0.0, 0.0));
        world.add_component(entity, Velocity::new(0.0, 0.0));
        world.add_component(entity, TargetPosition::new(1920.0, 1080.0));

        // Simulate 1000 ticks at 1000Hz (1 second of physics integration)
        for _ in 0..1000 {
            spring_physics_system_batch(&mut world, DT_1000HZ);
        }

        let pos = world.get_component::<Position>(entity).unwrap();
        let vel = world.get_component::<Velocity>(entity).unwrap();

        assert!((pos.x - 1920.0).abs() < 0.1, "Position X should converge to target");
        assert!((pos.y - 1080.0).abs() < 0.1, "Position Y should converge to target");
        assert_eq!(vel.vx, 0.0, "Velocity VX should snap to 0");
        assert_eq!(vel.vy, 0.0, "Velocity VY should snap to 0");
    }
}
