"""Independent Wings renderer, original shader core plus scene-depth silhouette clipping."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 t=(HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('ModuleBreadcrumbs','ModuleWings').replace('BreadcrumbRenderer','WingsRenderer').replace('Breadcrumbs','Wings')
 def once(a,b):
  nonlocal t
  assert t.count(a)==1,(a,t.count(a));t=t.replace(a,b)
 once('    layout: wgpu::BindGroupLayout,','    layout: wgpu::BindGroupLayout,\n    mesh_layout:wgpu::BindGroupLayout,')
 old='''        let empty = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wings mesh"),
            bind_group_layouts: &[],'''
 new='''        let mesh_layout=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:Some("Wings scene depth"),entries:&[wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Texture{sample_type:wgpu::TextureSampleType::Depth,view_dimension:wgpu::TextureViewDimension::D2,multisampled:false},count:None}]});
        let empty = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wings mesh"),
            bind_group_layouts: &[Some(&mesh_layout)],'''
 once(old,new);once('include_str!("breadcrumbs.wgsl")','include_str!("wings.wgsl")')
 once('            layout,','            layout,\n            mesh_layout,')
 once('        target: &wgpu::TextureView,\n        game_time: f32,','        target: &wgpu::TextureView,\n        depth:&wgpu::TextureView,\n        game_time: f32,')
 once('        let pass = |encoder:', '        let mesh_group=device.create_bind_group(&wgpu::BindGroupDescriptor{label:Some("Wings scene depth"),layout:&self.mesh_layout,entries:&[wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(depth)}]});\n        let pass = |encoder:')
 once('pass(encoder, target, &self.plain, None, true, false);','pass(encoder, target, &self.plain, Some(&mesh_group), true, false);')
 once('pass(encoder, &targets.views[0], &self.capture, None, true, true);','pass(encoder, &targets.views[0], &self.capture, Some(&mesh_group), true, true);')
 return t
