#[derive(Clone, Debug, serde::Deserialize)]
pub struct Quad {
    pub corners: [[f64; 3]; 4],
    pub colors: [[f32; 4]; 4],
    pub uv: [[f32; 2]; 4],
    pub inner_color: [f32; 4],
    pub inner: f32,
    pub shape: u32,
}
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Line {
    pub ends: [[f64; 3]; 2],
    pub colors: [[f32; 4]; 2],
}
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Effect {
    pub style: usize,
    pub params: [f32; 8],
    pub accent: [f32; 4],
    pub range: f32,
    pub opacity: f32,
}
#[derive(Default, Debug, serde::Deserialize)]
pub struct Frame {
    pub quads: Vec<Quad>,
    pub lines: Vec<Line>,
    pub order: Vec<[usize; 2]>,
    pub effect: Option<Effect>,
    pub depth: bool,
}
thread_local! {static PENDING:RefCell<Option<Frame>>=const{RefCell::new(None)};}
pub fn begin_frame(json: &str) {
    PENDING.with_borrow_mut(|f| *f = serde_json::from_str(json).ok());
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub clip: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
    pub inner_color: [f32; 4],
    pub params: [f32; 4],
}
pub fn vertices(frame: &Frame, projection: Mat4, camera: DVec3, size: [u32; 2]) -> Vec<Vertex> {
    let project = |p: [f64; 3], color: [f32; 4]| Vertex {
        clip: (projection * (DVec3::from_array(p) - camera).as_vec3().extend(1.)).to_array(),
        color,
        uv: [0.; 2],
        inner_color: color,
        params: [0.; 4],
    };
    let mut out = vec![];
    for &[kind, index] in &frame.order {
        if kind == 0 {
            let q = &frame.quads[index];
            let v: [Vertex; 4] = std::array::from_fn(|i| {
                let mut v = project(q.corners[i], q.colors[i]);
                v.uv = q.uv[i];
                v.inner_color = q.inner_color;
                v.params = [q.shape as f32, q.inner, 0., 0.];
                v
            });
            for i in [0, 1, 2, 0, 2, 3] {
                out.push(v[i]);
            }
            continue;
        }
        let line = &frame.lines[index];
        let mut v = std::array::from_fn::<_, 2, _>(|i| project(line.ends[i], line.colors[i]));
        let mut a = Vec4::from_array(v[0].clip);
        let mut b = Vec4::from_array(v[1].clip);
        for plane in 0..2 {
            let distance = |p: Vec4| if plane == 0 { p.w - 0.00001 } else { p.w - p.z };
            let da = distance(a);
            let db = distance(b);
            if da < 0. && db < 0. {
                a = Vec4::ZERO;
                b = Vec4::ZERO;
                break;
            }
            if (da < 0.) != (db < 0.) {
                let t = da / (da - db);
                let c =
                    std::array::from_fn(|i| v[0].color[i] + (v[1].color[i] - v[0].color[i]) * t);
                let p = a.lerp(b, t);
                if da < 0. {
                    a = p;
                    v[0].color = c;
                } else {
                    b = p;
                    v[1].color = c;
                }
            }
        }
        if a.w <= 0. || b.w <= 0. {
            continue;
        }
        let screen = Vec2::new(size[0] as f32, size[1] as f32);
        let d = (b.truncate().truncate() / b.w - a.truncate().truncate() / a.w) * screen;
        if d.length_squared() == 0. {
            continue;
        }
        let d = d.normalize();
        let n = Vec2::new(-d.y, d.x) / screen;
        let mut q = [v[0], v[0], v[1], v[1]];
        for (i, (p, sign)) in [(a, 1.), (a, -1.), (b, -1.), (b, 1.)]
            .into_iter()
            .enumerate()
        {
            q[i].clip = (p + Vec4::new(n.x * sign * p.w, n.y * sign * p.w, 0., 0.)).to_array();
        }
        for i in [0, 1, 2, 0, 2, 3] {
            out.push(q[i]);
        }
    }
    if frame.effect.is_some() && !frame.depth {
        for v in &mut out {
            v.clip[1] = -v.clip[1];
        }
    }
    out
}
