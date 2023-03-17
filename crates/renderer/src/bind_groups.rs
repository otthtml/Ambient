use wgpu::BindGroup;

pub struct BindGroups<'a> {
    pub globals: &'a BindGroup,
    pub entities: &'a BindGroup,
    pub mesh_data: &'a BindGroup,
    // Subset of `mesh_data`
    pub mesh_meta: &'a BindGroup,
}
