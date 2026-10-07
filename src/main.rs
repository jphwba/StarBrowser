use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use wgpu;
mod client;
use crate::client::Protocol;
use glyphon::{TextArea, Cache, Color, Buffer, Metrics, Resolution, FontSystem, Shaping, SwashCache, TextBounds, TextRenderer, Viewport, Attrs, TextAtlas};

pub fn rgbatocolour(r: u8, g: u8, b: u8, a: u8) -> wgpu::Color {
    wgpu::Color {
        r: (r as f64) / 255.0,
        g: (g as f64) / 255.0,
        b: (b as f64) / 255.0,
        a: (a as f64) / 255.0,
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
    display_text: String,
    viewport: Viewport,
}

impl State {
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

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if !self.display_text.is_empty() {
            let scale = 24.0;
            let mut buffer = Buffer::new(
                &mut self.font_system,
                Metrics::new(scale, scale * 1.4),
            );
            buffer.set_size(
                &mut self.font_system,
                Some(self.config.width as f32),
                Some(self.config.height as f32),
            );
            buffer.set_text(
                &mut self.font_system,
                &self.display_text,
                &Attrs::new(),
                Shaping::Basic,
                None,
            );
            buffer.shape_until_scroll(&mut self.font_system, true);
            let text_area = TextArea {
                buffer: &buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.config.width as i32,
                    bottom: self.config.height as i32,
                },
                default_color: Color::rgba(0,0,0,255),
                custom_glyphs: &[],
            };

            let _ = self.text_renderer.prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                [text_area],
                &mut self.swash_cache,
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(rgbatocolour(0, 255, 0, 100)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        

            if !self.display_text.is_empty() {
                let _ = self
                    .text_renderer
                    .render(&self.text_atlas,&self.viewport,&mut render_pass);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub async fn new(window: Arc<Window>, display_text: String) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            },)
            .await
            .unwrap();

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
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );
        let viewport = Viewport::new(&device, &cache);

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
            text_renderer,
            display_text,
            text_atlas,
            viewport,
        }
    }

    pub fn update(&mut self) {
        if !self.issurfaceconfigured {
            self.surface.configure(&self.device, &self.config);
            self.issurfaceconfigured = true;
            self.viewport
                .update(&self.queue, Resolution { width: self.config.width, height: self.config.height });
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
    display_text: String,
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut windowattrib = Window::default_attributes().with_title("Star Browser");

        if self.window_options.use_transparent {
            windowattrib = windowattrib.with_transparent(true);
        }

        let window = Arc::new(event_loop.create_window(windowattrib).unwrap());
        let text = std::mem::take(&mut self.display_text);
        self.state = Some(pollster::block_on(State::new(window, text)));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    )
    {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                state.render();
            }
            WindowEvent::KeyboardInput { 
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => match (code, key_state) {
                (KeyCode::Escape, ElementState::Pressed) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let target_url = args.get(1).cloned().unwrap_or_else(|| "http://httpbin.org/get".to_string());

    let display_text = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> String {
        let mut client = client::Client::new(Protocol::HTTP1_1, true);
        
        client.connect_to(target_url.clone());
        
    let host_only = target_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or(&target_url);

    let res = client.send_request(client::Req {
        method: String::from("GET"),
        requesttarget: String::from("/get"),
        protocol: client::Protocol::HTTP1_1,
        headers: vec![
            client::Header::new(String::from("Host"), host_only.to_string()),
            client::Header::new(String::from("User-Agent"), String::from("Star Browser")),
            client::Header::new(String::from("Connection"), String::from("close")),
        ],
        body: None,
    });

        match res {
            Some(response) => {
                let status = response.status_code.unwrap_or(0);
                let reason = response.reason.as_deref().unwrap_or("OK");
                let body = response.body.as_deref().unwrap_or("[Empty Body]");

                format!(
                    "HTTP/1.1 {} {}\n\n{}",
                    status, reason, body
                )
            }
            None => "No response".to_string(),
        }
    }))
    .unwrap_or_else(|_| "Request error".to_string());

    let mut app = App {
        display_text,
        ..Default::default()
    };

    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    _ = event_loop.run_app(&mut app);
}
