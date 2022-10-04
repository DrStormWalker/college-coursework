/// The trait for a vertex
pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
