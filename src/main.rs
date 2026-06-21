use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu;

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

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

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
            })
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

        Self {
            surface,
            window,
            adapter,
            device,
            queue,
            config,
            issurfaceconfigured: false,
        }
    }

    pub fn update(&mut self) {
        if !self.issurfaceconfigured {
            self.surface.configure(&self.device, &self.config);
            self.issurfaceconfigured = true;
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
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut windowattrib = Window::default_attributes().with_title("Star Browser");

        if self.window_options.use_transparent {
            windowattrib = windowattrib.with_transparent(true);
        }

        let window = Arc::new(event_loop.create_window(windowattrib).unwrap());
        self.state = Some(pollster::block_on(State::new(window)));
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
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App {
        window_options: WindowOptions {
            use_transparent: false,
        },
        ..Default::default()
    };
    _ = event_loop.run_app(&mut app);
}