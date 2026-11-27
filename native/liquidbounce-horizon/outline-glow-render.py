"""Owned common OutlineGlow derivative. Frozen ShaderLayer/StorageESP stay unchanged."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if (HERE/'storage-esp-render.py').is_file() else HERE.parents[1]

def source():
 t=runpy.run_path(str(NATIVE/'storage-esp-render.py'))['source']().replace('StorageRenderer','OwnerRenderer').replace('StorageESP','OutlineGlow').replace('ModuleStorageESP','OutlineGlow')
 def rep(old,new,count=1):
  nonlocal t
  assert t.count(old)==count,(old,t.count(old));t=t.replace(old,new)
 rep('pub fn active()->bool', 'pub(super) fn input(json:&str,models:Vec<crate::mesher::Vertex>){begin_frame(json);MODELS.with_borrow_mut(|out|*out=models);}\npub fn active()->bool')
 rep('    model_pipeline:wgpu::RenderPipeline,','    model_pipeline:wgpu::RenderPipeline,\n    mask_layout:wgpu::BindGroupLayout,\n    mask_pipeline:wgpu::RenderPipeline,')
 rep('const SOURCES: [&str; 12]','const SOURCES: [&str; 14]')
 rep('pipelines: [wgpu::RenderPipeline; 12]','pipelines: [wgpu::RenderPipeline; 14]')
 rep('    include_str!("storage-esp-sobel.frag"),','    include_str!("storage-esp-sobel.frag"),\n    include_str!("outline-glow-blur-h.frag"),\n    include_str!("outline-glow-blur-v.frag"),')
 rep('        Self {\n            model_pipeline,model_pending:None,','''        let mask_shader=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("OutlineGlow original tag mask"),source:wgpu::ShaderSource::Wgsl(include_str!("outline-glow.wgsl").into())});
        let mask_layout=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:Some("OutlineGlow tag mask"),entries:&[
            wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Texture{sample_type:wgpu::TextureSampleType::Float{filterable:false},view_dimension:wgpu::TextureViewDimension::D2,multisampled:false},count:None},
            wgpu::BindGroupLayoutEntry{binding:2,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Buffer{ty:wgpu::BufferBindingType::Uniform,has_dynamic_offset:false,min_binding_size:None},count:None},
        ]});
        let mask_pl=device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:Some("OutlineGlow tag mask"),bind_group_layouts:&[Some(&mask_layout)],..Default::default()});
        let mask_pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:Some("OutlineGlow original mask.frag formula"),layout:Some(&mask_pl),vertex:wgpu::VertexState{module:&mask_shader,entry_point:Some("screen"),compilation_options:Default::default(),buffers:&[]},primitive:Default::default(),depth_stencil:None,multisample:Default::default(),fragment:Some(wgpu::FragmentState{module:&mask_shader,entry_point:Some("mask"),compilation_options:Default::default(),targets:&[Some(FORMAT.into())]}),multiview_mask:None,cache:None});
        Self {
            mask_layout,mask_pipeline,
            model_pipeline,model_pending:None,''')
 rep('    pub fn pending(&self) -> bool {','    pub(super) fn output(&self)->Option<&wgpu::TextureView>{self.targets.as_ref().map(|t|&t.views[3])}\n    pub(super) fn silhouette(&self)->Option<&wgpu::TextureView>{self.targets.as_ref().map(|t|&t.views[0])}\n    pub fn pending(&self) -> bool {')
 rep('        entity_group:&wgpu::BindGroup,\n    ) {','        entity_group:&wgpu::BindGroup,\n        mask:Option<(&wgpu::TextureView,u32,bool)>,\n        capture_only:bool,\n    ) {')
 rep('self.pending.take()','self.pending.as_ref()')
 rep('let Some(effect) = frame.effect else','let Some(effect) = frame.effect.as_ref() else')
 rep('p.draw(0..count, 0..1);','p.draw(0..*count, 0..1);')
 rep('        pass(encoder, &targets.views[0], &self.capture, None, true, true);','''        if let Some((source,tag,untagged))=mask {
            let data=[tag,u32::from(untagged),0,0];
            let uniform=device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:Some("EffectMask"),contents:bytemuck::cast_slice(&data),usage:wgpu::BufferUsages::UNIFORM});
            let group=device.create_bind_group(&wgpu::BindGroupDescriptor{label:Some("OutlineGlow tag mask"),layout:&self.mask_layout,entries:&[wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(source)},wgpu::BindGroupEntry{binding:2,resource:uniform.as_entire_binding()}]});
            pass(encoder,&targets.views[0],&self.mask_pipeline,Some(&group),false,true);
        }else{
        pass(encoder, &targets.views[0], &self.capture, None, true, true);''')
 rep('self.model_pending.take()','self.model_pending.as_ref()')
 rep('p.draw(0..count,0..1);','p.draw(0..*count,0..1);')
 rep('        if effect.style==9{','        }\n        if capture_only{return;}\n        if effect.style==9{')
 rep('if effect.style==9{let group=group(&targets.views[0]);pass(encoder,&targets.views[3],&self.pipelines[11],Some(&group),false,true);}', 'if effect.style==9{for(source,dest,pipeline)in[(0,1,11),(1,2,12),(2,3,13)]{let group=group(&targets.views[source]);pass(encoder,&targets.views[dest],&self.pipelines[pipeline],Some(&group),false,true);}}')
 rep('        let group = group(&targets.views[3]);\n        pass(encoder, target, &self.composite, Some(&group), false, false);','        // Common renderer combines straight-alpha style targets before one scene conversion.')
 return t
