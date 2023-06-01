use crate::screen::objects;

/// Assumptions:
/// only applied to icons
/// the values stored represent percent of transition remaining
pub enum Transition {
    FadeOut(f32),
    SqueezeDown(f32),
}

impl Transition {
    pub fn apply(&self, icon: &mut objects::Icon, queue: &wgpu::Queue) {
        match self {
            Transition::FadeOut(p) => {
                // overwrite the icon's opacity in the tint
                // transitions from opacity of 1.0 to 0.0
                icon.tint[3] = 3.0 * (*p).powf(2.0) - 2.0 * (*p).powf(3.0);
                // rewrite vertex buffer
                for v in &mut icon.vertices {
                    v.color = icon.tint.into();
                }
                queue.write_buffer(&icon.vbuf, 0, bytemuck::cast_slice(&icon.vertices));
            }
            Transition::SqueezeDown(p) => {
                // for this to make sense, the tint should be 0 in the display config
                let mut tmp_vtxs = icon.vertices.clone();
                for v in &mut tmp_vtxs {
                    v.color[3] = 1.0;
                }
                // do the actual scaling here
                // let smoothstepped = (3.0 * (*p).powf(2.0) - 2.0 * (*p).powf(3.0));
                // let modified_height =
                //     smoothstepped * (icon.vertices[1].position[1] - icon.vertices[0].position[1]);
                let modified_height = *p * (icon.vertices[1].position[1] - icon.vertices[0].position[1]);
                tmp_vtxs[1].position[1] = icon.vertices[0].position[1] + modified_height;
                tmp_vtxs[2].position[1] = icon.vertices[0].position[1] + modified_height;
                // tmp_vtxs[1].texture[1] = 1.0 - smoothstepped;
                // tmp_vtxs[2].texture[1] = 1.0 - smoothstepped;
                tmp_vtxs[1].texture[1] = 1.0 - *p;
                tmp_vtxs[2].texture[1] = 1.0 - *p;
                queue.write_buffer(&icon.vbuf, 0, bytemuck::cast_slice(&tmp_vtxs));
            }
        }
    }
}
