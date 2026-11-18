//! A procedural WebGL 2 field. Vertex and instance IDs generate the geometry;
//! each frame uploads only the camera, clock and four gate weights.

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlContextAttributes, WebGlProgram,
    WebGlUniformLocation,
};

// Entry, four postal stations, exit. Both WebGL and the clickable clock labels
// use these world coordinates and the same camera projection.
const ANCHORS: [[f32; 3]; 6] = [
    [-4.0, -0.20, 0.15],
    [-2.4, 0.14, 0.34],
    [-0.8, 0.28, 0.05],
    [0.8, 0.02, -0.32],
    [2.4, -0.28, -0.26],
    [4.0, -0.20, 0.15],
];

pub(super) struct Camera([[f32; 4]; 4]);

impl Camera {
    pub fn new([yaw, pitch]: [f32; 2], aspect: f32) -> Self {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let f = 2.14;
        let x = f / aspect;
        // Column-major orbit and perspective transform. Transparent filaments
        // add light, so they share a clip depth and need no depth buffer.
        let mut camera = Self([
            [x * cy, f * sp * sy, 0.0, cp * sy],
            [0.0, f * cp, 0.0, -sp],
            [x * sy, -f * sp * cy, 0.0, -cp * cy],
            [0.0, 0.0, 0.0, (9.3 / aspect + 1.4).max(6.35)],
        ]);
        let midpoint = (camera.project(ANCHORS[0]).0 + camera.project(ANCHORS[5]).0) * 0.5;
        for column in &mut camera.0 {
            column[0] -= midpoint * column[3];
        }
        camera
    }

    fn project(&self, [x, y, z]: [f32; 3]) -> (f32, f32) {
        let m = &self.0;
        let row = |r: usize| m[0][r] * x + m[1][r] * y + m[2][r] * z + m[3][r];
        let w = row(3);
        (row(0) / w, row(1) / w)
    }

    pub fn label_position(&self, station: usize) -> (f32, f32) {
        let mut anchor = ANCHORS[station + 1];
        anchor[1] += 1.13;
        let (x, y) = self.project(anchor);
        ((1.0 + x) * 50.0, (1.0 - y) * 50.0)
    }
}

pub(super) struct Renderer {
    gl: Gl,
    program: WebGlProgram,
    viewport: Option<WebGlUniformLocation>,
    pixel_ratio: Option<WebGlUniformLocation>,
    camera: Option<WebGlUniformLocation>,
    transmission: Option<WebGlUniformLocation>,
    aligned: Option<WebGlUniformLocation>,
    clock: Option<WebGlUniformLocation>,
    layer: Option<WebGlUniformLocation>,
    segments: Option<WebGlUniformLocation>,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, JsValue> {
        let attributes = WebGlContextAttributes::new();
        attributes.set_depth(false);
        let gl: Gl = canvas
            .get_context_with_context_options("webgl2", &attributes)?
            .ok_or("WebGL 2 unavailable")?
            .dyn_into()?;
        let program = gl
            .create_program()
            .ok_or("Unable to allocate field program")?;
        if let Err(error) = link_field(&gl, &program) {
            gl.delete_program(Some(&program));
            return Err(error);
        }
        gl.use_program(Some(&program));
        gl.enable(Gl::BLEND);
        gl.blend_func_separate(Gl::ONE, Gl::ONE, Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA);
        let location = |name| gl.get_uniform_location(&program, name);
        gl.uniform3fv_with_f32_array(location("anchors[0]").as_ref(), ANCHORS.as_flattened());
        Ok(Self {
            viewport: location("viewport"),
            pixel_ratio: location("pixel_ratio"),
            camera: location("camera"),
            transmission: location("transmission"),
            aligned: location("aligned"),
            clock: location("clock_s"),
            layer: location("layer"),
            segments: location("segments"),
            gl,
            program,
        })
    }

    pub fn draw(
        &self,
        camera: &Camera,
        gates: &[f32; 4],
        clock: f32,
        density: f32,
        mail: [f32; 2],
    ) {
        let gl = &self.gl;
        let (width, height) = (gl.drawing_buffer_width(), gl.drawing_buffer_height());
        gl.viewport(0, 0, width, height);
        gl.clear(Gl::COLOR_BUFFER_BIT);
        gl.uniform2f(self.viewport.as_ref(), width as f32, height as f32);
        gl.uniform1f(self.pixel_ratio.as_ref(), density);
        gl.uniform_matrix4fv_with_f32_array(self.camera.as_ref(), false, camera.0.as_flattened());
        gl.uniform2f(self.transmission.as_ref(), mail[0], mail[1]);
        gl.uniform4fv_with_f32_array(self.aligned.as_ref(), gates);
        gl.uniform1f(self.clock.as_ref(), clock);
        gl.uniform1i(self.layer.as_ref(), 0);
        gl.draw_arrays(Gl::POINTS, 0, 192);
        // Floor grid, five bundles of eight filaments, four engraved triple hoops.
        for (layer, segments, instances) in [(3, 1, 34), (1, 112, 40), (2, 100, 12)] {
            gl.uniform1i(self.layer.as_ref(), layer);
            gl.uniform1i(self.segments.as_ref(), segments);
            gl.draw_arrays_instanced(Gl::TRIANGLE_STRIP, 0, (segments + 1) * 2, instances);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.gl.use_program(None);
        self.gl.delete_program(Some(&self.program));
    }
}

fn link_field(gl: &Gl, program: &WebGlProgram) -> Result<(), JsValue> {
    for (kind, source) in [
        (Gl::VERTEX_SHADER, include_str!("field.vert")),
        (Gl::FRAGMENT_SHADER, include_str!("field.frag")),
    ] {
        let shader = gl
            .create_shader(kind)
            .ok_or("Unable to allocate field shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        if gl
            .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
            .as_bool()
            != Some(true)
        {
            let log = gl.get_shader_info_log(&shader).unwrap_or_default();
            gl.delete_shader(Some(&shader));
            return Err(log.into());
        }
        gl.attach_shader(program, &shader);
        gl.delete_shader(Some(&shader));
    }
    gl.link_program(program);
    if gl.get_program_parameter(program, Gl::LINK_STATUS).as_bool() != Some(true) {
        return Err(gl.get_program_info_log(program).unwrap_or_default().into());
    }
    Ok(())
}
