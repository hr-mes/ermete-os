pub struct CompositorState {}
impl CompositorState {
    pub fn new(_freq: f64) -> Self { Self {} }
}
pub fn render_system(_world: &crate::ecs::SharedEcsWorld, _state: &mut CompositorState) {}
