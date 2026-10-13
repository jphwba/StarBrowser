use std::debug_assert;
use std::sync::Arc;

use glyphon::cosmic_text::skrifa::raw::tables::layout;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::CursorIcon::Text;
use winit::window::{Window, WindowId};
mod html;
mod client;
use crate::client::Protocol;
use glyphon::{TextArea, Cache, Color, Buffer, Metrics, Resolution, FontSystem, Shaping, SwashCache, TextBounds, TextRenderer, Viewport, Attrs, TextAtlas};

const TAB_STRIP_BG: [u8; 3] = [0xC9, 0xD5, 0xE8];
const TAB_BG: [u8; 3] = [0xF3, 0xF6, 0xFB];
const TOOLBAR_BG: [u8; 3] = [0xF3, 0xF6, 0xFB];
const BORDER: [u8; 3] = [0x86, 0x97, 0xB5];

const BUTTON_BG: [u8; 3] = [0xE3, 0xE9, 0xF3];
const ADDR_BG: [u8; 3] = [0xFF, 0xFF, 0xFF];
const ADDR_FOCUS_BORDER: [u8; 3] = [0x4D, 0x90, 0xFE];
const PAGE_BG: [u8; 3] = [0xFF, 0xFF, 0xFF];
// check distances between control and description
const MAIN_PAGE_TEXT: &str = "Welcome to Star Browser by jphwba\nType a web address into the search box. Make sure its in the format of https/http://example.com/page.\nShortcuts:\n CTRL+L     focus the address box\n CTRL+R      Reload\n ALT+LEFT       back\n ALT+RIGHT        forward\n Up/Down/PgUp/PgDn/mwheel  scroll";

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    colour: [f32; 3],
}

fn vertices_as_bytes(v: &[Vertex]) -> &[u8] {
    unsafe {std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v))}
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {c/12.92} else {((c+0.055)/1.055).powf(2.4)}
}

const QUAD_SHADER: &str = r#"
struct VsIn { @location(0) pos: vec2<f32>, @location(1) colour: vec3<f32> };
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) colour: vec3<f32> };

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = vec4<f32>(in.pos, 0.0, 1.0);
    out.colour = in.colour;
    return out;
}
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.colour, 1.0);
}"#;

const MAX_QUADS: usize = 64;

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

struct Ui {
    tab_strip: Rect,
    tab: Rect,
    toolbar: Rect,
    back: Rect,
    forward: Rect,
    reload: Rect,
    addr: Rect,
    content_top: f32,
}

fn layout_ui(width: f32, s: f32) -> Ui {
    let r = |x: f32, y: f32, w: f32, h: f32| Rect {x:x * s, y: y * s, w: w*s, h: h*s};
    Ui {
        tab_strip: r(0.0, 0.0, width / s, 32.0),
        tab: r(8.0, 6.0, 210.0, 26.0),
        toolbar: r(0.0, 32.0, width/s, 38.0),
        back: r(8.0, 38.0, 28.0, 26.0),
        forward: r(40.0, 38.0, 28.0, 26.0),
        reload: r(72.0, 38.0, 28.0, 26.0),
        addr: r(108.0, 38.0, (width/s -120.0).max(60.0), 26.0),
        content_top: 70.0*s,
    }
}

fn normalise_url(input: &str) -> String {
    let t = input.trim();
    if t.contains("://") {
        return t.to_string();
    }
    let local = t.starts_with("localhost") || t.starts_with(|c: char| c.is_ascii_digit());
    format!("{}://{}", if local {"http"} else {"https"}, t)
}

fn fetch_page(url: &str) -> Result<String, String> {
    let mut client = client::Client::new(Protocol::HTTP1_1, true);
    client.connect_to(url.to_string());


    let no_scheme = url.trim_start_matches("https://").trim_start_matches("http://");
    let host = no_scheme.split('/').next().unwrap_or(no_scheme).to_string();
    let path = match no_scheme.find('/') {
        Some(i) => no_scheme[i..].to_string(),
        None => "/".to_string(),
    };

    let res = client.send_request(client::Req {
        method: String::from("GET"),
        requesttarget: path,
        protocol: Protocol::HTTP1_1,
        headers: vec![
            client::Header::new(String::from("Host"), host),
            client::Header::new(String::from("User-Agent"), String::from("Star Browser")),
            client::Header::new(String::from("Accept"), String::from("text/html,*/*")),
            client::Header::new(String::from("Connection"), String::from("close")),
        ],
        body: None,
    });

    match res {
        Some(r) => {
            let status = r.status_code.unwrap_or(0);
            if (200..400).contains(&status) {
                Ok(r.body.unwrap_or_default())
            } else {
                Err(format!("HTTP {} {}", status, r.reason.unwrap_or_default()))
            }
        }
        None => Err("Couldnt connect to the server".to_string()),
    }   
}

#[allow(dead_code)]
struct State {
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    issurfaceconfigured: bool,
    window: Arc<Window>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    viewport: Viewport,
    quad_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    address: String,
    address_focused: bool,
    page_text: String,
    title: String,
    history: Vec<String>,
    history_pos: usize,
    scroll: f32,
    max_scroll: f32,
    mouse: PhysicalPosition<f64>,
    modifiers: ModifiersState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: Default::default(),
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::defaults(),
            trace: wgpu::Trace::Off,
            memory_hints: Default::default(),
        }).await.unwrap();

        let surfacecap = surface.get_capabilities(&adapter);
        let surfaceformat = surfacecap
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surfacecap.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surfaceformat,
            width: size.width,
            height: size.height,
            present_mode: surfacecap.present_modes[0],
            alpha_mode: surfacecap.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surfaceformat);
        let text_renderer = TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(&device, &cache);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chome shader"),
            source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
        });
        let attrs = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];
        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chrome pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surfaceformat,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chrome verticies"),
            size: (MAX_QUADS * 6 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });




        Self {
            surface,
            window,
            adapter,
            device,
            queue,
            config,
            issurfaceconfigured: false,
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            viewport,
            quad_pipeline,
            quad_buffer,
            address: String::new(),
            address_focused: true,
            page_text: MAIN_PAGE_TEXT.to_string(),
            title: "New Tab".to_string(),
            history: Vec::new(),
            history_pos: 0,
            scroll: 0.0,
            max_scroll: 0.0,
            mouse: PhysicalPosition::new(0.0, 0.0),
            modifiers: ModifiersState::empty(),
        }
    }

    fn scale(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    fn load(&mut self, url: &str) {
        match fetch_page(url) {
            Ok(body) => {
                let looks_like_html = {
                    let lower = body.to_ascii_lowercase();
                    lower.contains("<html") || lower.contains("<body") || lower.trim_start().starts_with('<')
                };
                if looks_like_html {
                    let doc = html::parse(&body);
                    let text = html::extract_visible_text(&doc);
                    self.title = html::find_title(&doc).unwrap_or_else(|| url.to_string());
                    self.page_text = if text.is_empty() { "(This page has no visible text)".to_string()} else {text};
                } else {
                    self.title = url.to_string();
                    self.page_text = body;
                }
            }
            Err(e) => {
                self.title = "Error".to_string();
                self.page_text = format!("This page cant be loaded\n\n{}\n\n{}", url, e);
            }
        }
        self.window.set_title(&format!("{} - Star Browser", self.title));
        self.scroll = 0.0;
    }

    fn navigate(&mut self, input: &str) {
        let url = normalise_url(input);
        if self.history.is_empty() {
            self.history.push(url.clone());
            self.history_pos = 0;
        } else if self.history[self.history_pos] != url {
            self.history.truncate(self.history_pos + 1);
            self.history.push(url.clone());
            self.history_pos += 1;
        }
        self.address = url.clone();
        self.address_focused = false;
        self.load(&url);
    }

    fn go_history(&mut self, delta: isize) {
        let target = self.history_pos as isize + delta;
        if target < 0 || target as usize >= self.history.len() {
            return;
        }
        self.history_pos = target as usize;
        let url = self.history[self.history_pos].clone();
        self.address = url.clone();
        self.load(&url);
    }

    fn reload(&mut self) {
        if let Some(url) = self.history.get(self.history_pos).cloned() {
            self.load(&url);
        }
    }

    fn on_click(&mut self) {
        let ui = layout_ui(self.config.width as f32, self.scale());
        let (x, y) = (self.mouse.x as f32, self.mouse.y as f32);
        if ui.back.contains(x, y ) {
            self.go_history(-1);
        } else if ui.forward.contains(x, y) {
            self.go_history(1);
        } else if ui.reload.contains(x, y) {
            self.reload();
        } else if ui.addr.contains(x, y) {
            self.address_focused = true;
        } else {
            self.address_focused = false;
        }
    }

    fn on_key(&mut self, code: KeyCode, text: Option<&str>) {
        let ctrl = self.modifiers.control_key();
        let alt = self.modifiers.alt_key();

        if ctrl {
            match code {
                KeyCode::KeyL => {
                    self.address_focused = false;
                    return;
                }
                KeyCode::KeyR => {
                    self.reload();
                    return;
                }
                _ => {}
            }
        }
        if alt {
            match code {
                KeyCode::ArrowLeft => return self.go_history(-1),
                KeyCode::ArrowRight => return self.go_history(1),
            _ => {}
            }
        }

        let line = 40.0 * self.scale();
        match code {
            KeyCode::Enter | KeyCode::NumpadEnter if self.address_focused => {
                let target = self.address.clone();
                if !target.trim().is_empty() {
                    self.navigate(&target);
                }
            }
            KeyCode::Escape => {
                self.address_focused = false;
                self.address = self.history.get(self.history_pos).cloned().unwrap_or_default();
            }
            KeyCode::Backspace if self.address_focused => {
                self.address.pop();
            }
            KeyCode::ArrowDown => self.scroll = (self.scroll + line).min(self.max_scroll),
            KeyCode::ArrowUp => self.scroll = (self.scroll - line).max(0.0),
            KeyCode::PageDown => self.scroll = (self.scroll + line * 12.0).min(self.max_scroll),
            KeyCode::PageUp => self.scroll = (self.scroll - line * 12.0).max(0.0),
            KeyCode::Home if !self.address_focused => self.scroll = 0.0,
            KeyCode::End if !self.address_focused => self.scroll = self.max_scroll,
            _ => {
                if self.address_focused && !ctrl && !alt {
                    if let Some(t) = text {
                        for c in t.chars().filter(|c| !c.is_control()) {
                            self.address.push(c);
                        }
                    }
                }
            }

        }
    }

    fn text_buffer(&mut self, text: &str, size: f32, width: Option<f32>) -> Buffer {
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(size, size * 1.4));
        buffer.set_size(&mut self.font_system, width, None);
        buffer.set_text(&mut self.font_system, text, &Attrs::new(), Shaping::Basic, None);
        buffer.shape_until_scroll(&mut self.font_system, true);
        buffer
    }

    pub fn render(&mut self) {
        self.window.request_redraw();
        if !self.issurfaceconfigured {
            return;
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(out) | wgpu::CurrentSurfaceTexture::Suboptimal(out) => out,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                return;
            }
            _ => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: Some("Render Encoder")});
        let (w, h) = (self.config.width as f32, self.config.height as f32);
        let s = self.scale();
        let ui = layout_ui(w, s);

        let mut verts: Vec<Vertex> = Vec::new();
        let mut quad = |r: Rect, c: [u8; 3]| {
            let col = [srgb_to_linear(c[0]), srgb_to_linear(c[1]), srgb_to_linear(c[2])];
            let px = |x: f32| x / w * 2.0 - 1.0;
            let py = |y: f32| 1.0 - y / h * 2.0;
            let (x0, y0, x1, y1) = (px(r.x), py(r.y), px(r.x + r.w), py(r.y + r.h));
            for p in [[x0, y0], [x1, y0], [x0, y1], [x1, y0], [x1, y1], [x0, y1]] {
                verts.push(Vertex { position: p, colour: col });
            }
        };
        let inset = |r: Rect, d: f32| Rect {x: r.x + d, y: r.y + d, w: r.w -2.0 * d, h: r.h - 2.0 * d};

        quad(ui.tab_strip, TAB_STRIP_BG);
        quad(ui.tab, BORDER);
        quad(Rect {x: ui.tab.x + s, y: ui.tab.y + s, w: ui.tab.w - 2.0 * s, h: ui.tab.h}, TAB_BG);
        quad(ui.toolbar, TOOLBAR_BG);
        quad(Rect { x: 0.0, y: ui.toolbar.y + ui.toolbar.h -s , w, h: s}, BORDER);
        for b in [ui.back, ui.forward, ui.reload] {
            quad(b, BORDER);
            quad(inset(b,s), BUTTON_BG);
        }
        quad(ui.addr, if self.address_focused { ADDR_FOCUS_BORDER } else {BORDER});
        quad(inset(ui.addr, s), ADDR_BG);
        let quad_count = verts.len() / 6;
        debug_assert!(quad_count <= MAX_QUADS);
        self.queue.write_buffer(&self.quad_buffer, 0, vertices_as_bytes(&verts));
        let vertex_count = verts.len() as u32;

        let ui_size = 14.0 * s;
        let addr_text = if self.address_focused { format!("{}|", self.address)} else {self.address.clone()};
        let addr_buf = self.text_buffer(&addr_text, ui_size, None);
        let title_buf = self.text_buffer(&self.title.clone(), 13.0 * s, None);
        let back_buf = self.text_buffer("<", 16.0 * s, None);
        let fwd_buf = self.text_buffer(">", 16.0 * s, None);
        let reload_buf = self.text_buffer("R", 14.0 * s, None);
        let page_text = self.page_text.clone();
        let page_buf = self.text_buffer(&page_text, 16.0 * s, Some((w - 40.0 * s).max(50.0)));

        let line_h = 16.0 * s * 1.4;
        let content_h = page_buf.layout_runs().count() as f32 * line_h;
        self.max_scroll = (content_h - (h - ui.content_top) + 30.0 * s).max(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        let full = |l: f32, t: f32, r: f32, b: f32| TextBounds { left: l as i32, top: t as i32, right: r as i32, bottom: b as i32};
        let black = Color::rgba(0, 0, 0, 255);
        let areas = [
            TextArea { buffer: &addr_buf, left: ui.addr.x + 6.0 * s, top: ui.addr.y + 5.0 * s, scale: 1.0,
            bounds: full(ui.addr.x, ui.addr.y, ui.addr.x + ui.addr.w, ui.addr.y + ui.addr.h), default_color: black, custom_glyphs: &[] },
            TextArea { buffer: &title_buf, left: ui.tab.x + 10.0 * s, top: ui.tab.y + 6.0 * s, scale: 1.0,
                bounds: full(ui.tab.x, ui.tab.y, ui.tab.x + ui.tab.w - 6.0 * s, ui.tab.y + ui.tab.h), default_color: black, custom_glyphs: &[] },
            TextArea { buffer: &back_buf, left: ui.back.x + 9.0 * s, top: ui.back.y + 3.0 * s, scale: 1.0,
                bounds: full(ui.back.x, ui.back.y, ui.back.x + ui.back.w, ui.back.y + ui.back.h), default_color: black, custom_glyphs: &[] },
            TextArea { buffer: &fwd_buf, left: ui.forward.x + 9.0 * s, top: ui.forward.y + 3.0 * s, scale: 1.0,
                bounds: full(ui.forward.x, ui.forward.y, ui.forward.x + ui.forward.w, ui.forward.y + ui.forward.h), default_color: black, custom_glyphs: &[] },
            TextArea { buffer: &reload_buf, left: ui.reload.x + 9.0 * s, top: ui.reload.y + 5.0 * s, scale: 1.0,
                bounds: full(ui.reload.x, ui.reload.y, ui.reload.x + ui.reload.w, ui.reload.y + ui.reload.h), default_color: black, custom_glyphs: &[] },
            TextArea { buffer: &page_buf, left: 20.0 * s, top: ui.content_top + 12.0 * s - self.scroll, scale: 1.0,
                bounds: full(0.0, ui.content_top, w, h), default_color: black, custom_glyphs: &[] },
        ];

        let _ = self.text_renderer.prepare(
            &self.device, &self.queue, &mut self.font_system, &mut self.text_atlas, &self.viewport, areas, &mut self.swash_cache,
        );

        {
            let page = |c: [u8; 3]| wgpu::Color { r: srgb_to_linear(c[0]) as f64, g: srgb_to_linear(c[1]) as f64, b: srgb_to_linear(c[2]) as f64, a: 1.0};
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(page(PAGE_BG)), store: wgpu::StoreOp::Store},
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.quad_pipeline);
            pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
            pass.draw(0..vertex_count, 0..1);
            let _ = self.text_renderer.render(&self.text_atlas, &self.viewport, &mut pass);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn update(&mut self) {
        if !self.issurfaceconfigured {
            self.surface.configure(&self.device, &self.config);
            self.issurfaceconfigured = true;
            self.viewport.update(&self.queue, Resolution { width: self.config.width, height: self.config.height});
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.issurfaceconfigured = false;
        }
    }
}

#[derive(Default)]
struct WindowOptions {
    use_transparent: bool,
}

#[derive(Default)]
struct App {
    window_options: WindowOptions,
    state: Option<State>,
    start_url: Option<String>,
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = Window::default_attributes().with_title("Star Browser");
        if self.window_options.use_transparent {
            attrs = attrs.with_transparent(true);
        }
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let mut state = pollster::block_on(State::new(window));
        if let Some(url) = self.start_url.take() {
            state.navigate(&url);
        }
        self.state = Some(state);

    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    )
    {
     let Some(state) = &mut self.state else {return};
     match event {
        WindowEvent::CloseRequested => event_loop.exit(),
        WindowEvent::Resized(size) => state.resize(size.width, size.height),
        WindowEvent::RedrawRequested => {
            state.update();
            state.render();
        }
        WindowEvent::ModifiersChanged(m) => state.modifiers = m.state(),
        WindowEvent::CursorMoved { position, ..} => state.mouse = position,
        WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, ..} => state.on_click(),
        WindowEvent::MouseWheel {delta, ..} => {
            let dy = match delta {
                MouseScrollDelta::LineDelta(_, y) => y * 60.0,
                MouseScrollDelta::PixelDelta(p) => p.y as f32,
            };
            state.scroll = (state.scroll - dy).clamp(0.0, state.max_scroll);
        }
        WindowEvent::KeyboardInput {
            event: KeyEvent { physical_key: PhysicalKey::Code(code), state: ElementState::Pressed, text, .. }, ..
        } => state.on_key(code, text.as_deref()),
        _ => {}
     }   
    }
}

fn main() {
    let mut app = App {start_url: std::env::args().nth(1), ..Default::default()};
    let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    _ = event_loop.run_app(&mut app);
}
