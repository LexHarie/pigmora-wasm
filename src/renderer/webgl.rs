use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

use super::Rect;

pub struct WebGlRenderer {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vao: WebGlVertexArrayObject,
    #[allow(dead_code)]
    vertex_buffer: WebGlBuffer,
    #[allow(dead_code)]
    index_buffer: WebGlBuffer,
    uniform_resolution: Option<WebGlUniformLocation>,
    uniform_origin: Option<WebGlUniformLocation>,
    uniform_size: Option<WebGlUniformLocation>,
    uniform_color: Option<WebGlUniformLocation>,
}

impl WebGlRenderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("Missing document"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas element not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context_options = js_sys::Object::new();
        js_sys::Reflect::set(&context_options, &JsValue::from_str("alpha"), &JsValue::FALSE)?;
        js_sys::Reflect::set(
            &context_options,
            &JsValue::from_str("antialias"),
            &JsValue::FALSE,
        )?;
        js_sys::Reflect::set(
            &context_options,
            &JsValue::from_str("depth"),
            &JsValue::FALSE,
        )?;
        js_sys::Reflect::set(
            &context_options,
            &JsValue::from_str("stencil"),
            &JsValue::FALSE,
        )?;
        js_sys::Reflect::set(
            &context_options,
            &JsValue::from_str("preserveDrawingBuffer"),
            &JsValue::FALSE,
        )?;
        js_sys::Reflect::set(
            &context_options,
            &JsValue::from_str("powerPreference"),
            &JsValue::from_str("high-performance"),
        )?;

        let gl = canvas
            .get_context_with_context_options("webgl2", &context_options)?
            .ok_or_else(|| JsValue::from_str("WebGL2 not supported"))?
            .dyn_into::<WebGl2RenderingContext>()?;

        let program = Self::create_program(&gl)?;
        let vertex_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("Failed to create vertex buffer"))?;
        let index_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("Failed to create index buffer"))?;

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| JsValue::from_str("Failed to create vertex array"))?;

        gl.bind_vertex_array(Some(&vao));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        let vertices: [f32; 8] = [
            0.0, 0.0, // top-left
            1.0, 0.0, // top-right
            0.0, 1.0, // bottom-left
            1.0, 1.0, // bottom-right
        ];
        let vertex_array = js_sys::Float32Array::from(vertices.as_ref());
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertex_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        let indices: [u16; 4] = [0, 1, 3, 2];
        let index_array = js_sys::Uint16Array::from(indices.as_ref());
        gl.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&index_buffer),
        );
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.bind_vertex_array(None);
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);

        gl.use_program(Some(&program));
        let uniform_resolution = gl.get_uniform_location(&program, "u_resolution");
        let uniform_origin = gl.get_uniform_location(&program, "u_origin");
        let uniform_size = gl.get_uniform_location(&program, "u_size");
        let uniform_color = gl.get_uniform_location(&program, "u_color");

        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        gl.disable(WebGl2RenderingContext::CULL_FACE);
        gl.clear_color(0.06, 0.07, 0.08, 1.0);

        Ok(Self {
            gl,
            program,
            vao,
            vertex_buffer,
            index_buffer,
            uniform_resolution,
            uniform_origin,
            uniform_size,
            uniform_color,
        })
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.gl
            .viewport(0, 0, width as i32, height as i32);
    }

    pub fn render_scene(&self, width: u32, height: u32, rects: &[Rect], selected: Option<Rect>) {
        self.gl
            .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        if width == 0 || height == 0 {
            return;
        }

        self.gl.use_program(Some(&self.program));
        self.gl.bind_vertex_array(Some(&self.vao));

        self.set_resolution(width, height);

        for (index, rect) in rects.iter().enumerate() {
            if !rect.is_valid() {
                continue;
            }
            self.set_rect_uniforms(rect);
            let tint = (index % 4) as f32 * 0.04;
            self.set_color(0.86 - tint, 0.42 + tint, 0.25 + tint, 1.0);
            self.gl
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
        }

        if let Some(rect) = selected {
            if rect.is_valid() {
                self.draw_selection_outline(&rect);
            }
        }
        self.gl.bind_vertex_array(None);
    }

    fn set_resolution(&self, width: u32, height: u32) {
        if let Some(resolution_loc) = &self.uniform_resolution {
            self.gl.uniform2f(
                Some(resolution_loc),
                width as f32,
                height as f32,
            );
        }
    }

    fn set_rect_uniforms(&self, rect: &Rect) {
        if let Some(origin_loc) = &self.uniform_origin {
            self.gl
                .uniform2f(Some(origin_loc), rect.x, rect.y);
        }

        if let Some(size_loc) = &self.uniform_size {
            self.gl
                .uniform2f(Some(size_loc), rect.width, rect.height);
        }
    }

    fn set_color(&self, r: f32, g: f32, b: f32, a: f32) {
        if let Some(color_loc) = &self.uniform_color {
            self.gl
                .uniform4f(Some(color_loc), r, g, b, a);
        }
    }

    fn draw_selection_outline(&self, rect: &Rect) {
        self.set_rect_uniforms(rect);
        self.set_color(0.98, 0.94, 0.9, 1.0);
        self.gl.line_width(1.0);
        self.gl.draw_elements_with_i32(
            WebGl2RenderingContext::LINE_LOOP,
            4,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        let handle_size = 24.0;
        let handle_half = handle_size * 0.5;
        let handles = [
            Rect {
                x: rect.x - handle_half,
                y: rect.y - handle_half,
                width: handle_size,
                height: handle_size,
            },
            Rect {
                x: rect.x + rect.width - handle_half,
                y: rect.y - handle_half,
                width: handle_size,
                height: handle_size,
            },
            Rect {
                x: rect.x + rect.width - handle_half,
                y: rect.y + rect.height - handle_half,
                width: handle_size,
                height: handle_size,
            },
            Rect {
                x: rect.x - handle_half,
                y: rect.y + rect.height - handle_half,
                width: handle_size,
                height: handle_size,
            },
        ];

        self.set_color(0.98, 0.96, 0.93, 1.0);
        for handle in &handles {
            self.set_rect_uniforms(handle);
            self.gl
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn create_program(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_source = "#version 300 es\nin vec2 a_position;\nuniform vec2 u_origin;\nuniform vec2 u_size;\nuniform vec2 u_resolution;\nvoid main() {\n  vec2 position = u_origin + (a_position * u_size);\n  vec2 zeroToOne = position / u_resolution;\n  vec2 zeroToTwo = zeroToOne * 2.0;\n  vec2 clip = zeroToTwo - 1.0;\n  gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);\n}\n";

        let fragment_source = "#version 300 es\nprecision mediump float;\nuniform vec4 u_color;\nout vec4 out_color;\nvoid main() {\n  out_color = u_color;\n}\n";

        let vertex_shader = Self::compile_shader(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_source,
        )?;
        let fragment_shader = Self::compile_shader(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_source,
        )?;

        let program = gl
            .create_program()
            .ok_or_else(|| JsValue::from_str("Failed to create program"))?;
        gl.bind_attrib_location(&program, 0, "a_position");
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        if gl
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            let log = gl
                .get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown program link error".to_string());
            Err(JsValue::from_str(&log))
        }
    }

    fn compile_shader(
        gl: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<web_sys::WebGlShader, JsValue> {
        let shader = gl
            .create_shader(shader_type)
            .ok_or_else(|| JsValue::from_str("Failed to create shader"))?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if gl
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            let log = gl
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown shader compile error".to_string());
            Err(JsValue::from_str(&log))
        }
    }
}
