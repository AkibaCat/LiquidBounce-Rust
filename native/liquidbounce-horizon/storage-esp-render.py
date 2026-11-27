"""Independent StorageESP derivative of frozen ShaderLayer; original shaders remain untouched."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def plain_source():return (HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('BreadcrumbRenderer','StoragePlainRenderer').replace('ModuleBreadcrumbs','ModuleStorageESP plain')
def source():
 t=(HERE/'breadcrumbs-render.rs').read_text(encoding='utf8').replace('BreadcrumbRenderer','StorageRenderer').replace('ModuleBreadcrumbs','ModuleStorageESP')
 def rep(o,n,count=1):
  nonlocal t
  assert t.count(o)==count,(o,t.count(o));t=t.replace(o,n)
 rep('    pub effect: Option<Effect>,','    pub effect: Option<Effect>,\n    #[serde(default)] pub targets:Vec<(i32,u32)>,')
 rep('    PENDING.with_borrow_mut(|f| *f = serde_json::from_str(json).ok());','    MODELS.with_borrow_mut(Vec::clear);\n    PENDING.with_borrow_mut(|f| *f = serde_json::from_str(json).ok());')
 rep('#[repr(C)]', '''thread_local!{static MODELS:RefCell<Vec<crate::mesher::Vertex>>=const{RefCell::new(Vec::new())};}
pub fn active()->bool{PENDING.with_borrow(|f|f.as_ref().is_some_and(|f|f.effect.is_some()))}
pub fn entity_color(id:i32)->Option<u32>{PENDING.with_borrow(|f|f.as_ref().and_then(|f|f.targets.iter().find(|v|v.0==id).map(|v|v.1)))}
pub fn capture_entity(id:i32,vertices:&[crate::mesher::Vertex]){if let Some(color)=entity_color(id){capture_model(vertices,color);}}
pub fn capture_model(vertices:&[crate::mesher::Vertex],argb:u32){
let rgba=(argb&0xff00ff00)|((argb>>16)&255)|((argb&255)<<16);
MODELS.with_borrow_mut(|out|{for q in vertices.chunks_exact(4){for i in [0,1,2,0,2,3]{let mut v=q[i];v.color=rgba;out.push(v);}}});}
#[repr(C)]#[derive(Clone,Copy,bytemuck::Pod,bytemuck::Zeroable)]struct ModelVertex{clip:[f32;4],uv:[f32;2],color:u32}
#[repr(C)]''')
 rep('const SOURCES: [&str; 11]', 'const SOURCES: [&str; 12]')
 rep('    include_str!("breadcrumbs-shaders/aura.frag"),','    include_str!("breadcrumbs-shaders/aura.frag"),\n    include_str!("storage-esp-sobel.frag"),')
 rep('pipelines: [wgpu::RenderPipeline; 11]','pipelines: [wgpu::RenderPipeline; 12]')
 rep('    capture: wgpu::RenderPipeline,','    capture: wgpu::RenderPipeline,\n    model_pipeline:wgpu::RenderPipeline,\n    model_pending:Option<(wgpu::Buffer,u32)>,')
 rep('pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool) -> Self {','pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear: bool,entity_layout:&wgpu::PipelineLayout) -> Self {')
 rep('        Self {\n            layout,','''        let model_shader=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("StorageESP native model outline"),source:wgpu::ShaderSource::Wgsl(include_str!("storage-esp-model.wgsl").into())});
        let attributes=wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x2,2=>Unorm8x4];
        let model_pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:Some("StorageESP real models"),layout:Some(entity_layout),vertex:wgpu::VertexState{module:&model_shader,entry_point:Some("model"),compilation_options:Default::default(),buffers:&[Some(wgpu::VertexBufferLayout{array_stride:28,step_mode:wgpu::VertexStepMode::Vertex,attributes:&attributes})]},primitive:wgpu::PrimitiveState{cull_mode:None,..Default::default()},depth_stencil:None,multisample:Default::default(),fragment:Some(wgpu::FragmentState{module:&model_shader,entry_point:Some("capture"),compilation_options:Default::default(),targets:&[Some(FORMAT.into())]}),multiview_mask:None,cache:None});
        Self {
            model_pipeline,model_pending:None,
            layout,''')
 rep('        self.pending = None;','        self.pending = None;self.model_pending=None;')
 rep('        if mesh.is_empty() {','''        let models=MODELS.with_borrow_mut(std::mem::take).into_iter().map(|v|{let mut clip=(projection*glam::Vec3::from_array(v.pos).extend(1.)).to_array();clip[1] = -clip[1];ModelVertex{clip,uv:v.uv,color:v.color}}).collect::<Vec<_>>();
        if !models.is_empty(){self.model_pending=Some((device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:Some("StorageESP real model vertices"),contents:bytemuck::cast_slice(&models),usage:wgpu::BufferUsages::VERTEX}),models.len()as u32));}
        if mesh.is_empty() && self.model_pending.is_none() {''')
 rep('        let count = mesh.len() as u32;','        let count = mesh.len() as u32;\n        let mesh=if mesh.is_empty(){vec![Vertex{clip:[0.;4],color:[0.;4]}]}else{mesh};')
 rep('        fov: f32,\n    ) {','        fov: f32,\n        entity_group:&wgpu::BindGroup,\n    ) {')
 old='''        for (i, pipeline) in [0, 1, effect.style + 2].into_iter().enumerate() {'''
 rep(old,'''        if let Some((vertices,count))=self.model_pending.take(){let mut p=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:Some("StorageESP real model silhouettes"),color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&targets.views[0],depth_slice:None,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Load,store:wgpu::StoreOp::Store}})],..Default::default()});p.set_pipeline(&self.model_pipeline);p.set_bind_group(0,entity_group,&[]);p.set_vertex_buffer(0,vertices.slice(..));p.draw(0..count,0..1);}
        if effect.style==9{let group=group(&targets.views[0]);pass(encoder,&targets.views[3],&self.pipelines[11],Some(&group),false,true);}else{
        for (i, pipeline) in [0, 1, effect.style + 2].into_iter().enumerate() {''')
 rep('        let group = group(&targets.views[3]);','        }\n        let group = group(&targets.views[3]);')
 return t
