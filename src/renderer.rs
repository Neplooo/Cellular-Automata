// Note that this renderer was about 85% Coded through A.I. Assistance.
// This renderer was not the goal of this project, and thus, I did not waste time
// Trying to troubleshoot it. Much of the base-level code also came from the Vello
// Developers. Originally, this was a part of the Simple Drawings Template.

// Much Love, Alberto

use anyhow::Result;
use array2d::Array2D;
use std::sync::{Arc, mpsc::Receiver};
use vello::kurbo::{Affine, Rect};
use vello::peniko::Color;
use vello::peniko::color::palette;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions, Scene};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use vello::wgpu::{self, CurrentSurfaceTexture};

const TILE_SIZE: f64 = 20.0;
const INITIAL_TILE_SIZE: f64 = TILE_SIZE * 1.5;
const MIN_TILE_SIZE: f64 = 2.0;
const MAX_TILE_SIZE: f64 = 100.0;
const ZOOM_FACTOR: f64 = 1.1;
const MAX_INITIAL_WINDOW_USAGE: f64 = 0.75;

/// The cellular automaton ruleset whose state is being rendered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ruleset {
    Conway,
    BriansBrain,
}

#[derive(Debug)]
enum RenderState {
    /// `RenderSurface` and `Window` for active rendering.
    Active {
        surface: Box<RenderSurface<'static>>,
        valid_surface: bool,
        window: Arc<Window>,
    },
    /// Cache a window so that it can be reused when the app is resumed after being suspended.
    Suspended(Option<Arc<Window>>),
}

struct SimpleVelloApp {
    /// The Vello `RenderContext` which is a global context that lasts for the
    /// lifetime of the application
    context: RenderContext,

    /// An array of renderers, one per wgpu device
    renderers: Vec<Option<Renderer>>,

    /// State for our example where we store the winit Window and the wgpu Surface
    state: RenderState,

    /// A vello Scene which is a data structure which allows one to build up a
    /// description a scene to be drawn (with paths, fills, images, text, etc)
    /// which is then passed to a renderer for rendering
    scene: Scene,

    /// The grid of cells to render
    grid: Array2D<i32>,
    /// Logical width and height of one cell, controlled with the mouse wheel.
    tile_size: f64,
    /// Pixel offset from the centered grid position, controlled by dragging.
    pan_offset: (f64, f64),
    is_panning: bool,
    last_cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    /// The ruleset that determines how cell states are visualized.
    ruleset: Ruleset,
    /// Grid snapshots produced by the simulator.
    updates: Receiver<Array2D<i32>>,
}

impl ApplicationHandler for SimpleVelloApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };

        // Get the winit window cached in a previous Suspended event or else create a new window
        let window = cached_window.take().unwrap_or_else(|| {
            create_winit_window(event_loop, self.grid.row_len(), self.grid.column_len())
        });

        // Create a vello Surface
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        // Save the Window and Surface to a state variable
        self.state = RenderState::Active {
            surface: Box::new(surface),
            valid_surface: true,
            window: window.clone(),
        };

        window.request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let mut changed = false;

        // Keep the newest snapshot if the simulator is ahead of rendering.
        while let Ok(grid) = self.updates.try_recv() {
            self.grid = grid;
            changed = true;
        }

        if changed {
            if let RenderState::Active { window, .. } = &self.state {
                window.request_redraw();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let RenderState::Active { window, .. } = &self.state {
            self.state = RenderState::Suspended(Some(window.clone()));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Only process events for our window, and only when we have a surface.
        let (surface, valid_surface, window) = match &mut self.state {
            RenderState::Active {
                surface,
                valid_surface,
                window,
            } if window.id() == window_id => (surface, valid_surface, window),
            _ => return,
        };

        match event {
            // Exit the event loop when a close is requested (e.g. window's close button is pressed)
            WindowEvent::CloseRequested => event_loop.exit(),

            // Resize the surface when the window is resized
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    self.context
                        .resize_surface(surface, size.width, size.height);
                    *valid_surface = true;
                } else {
                    *valid_surface = false;
                }
            }

            // Scroll up to zoom in and down to zoom out. The simulation grid is unchanged.
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_steps = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(position) => position.y / 50.0,
                };

                if scroll_steps != 0.0 {
                    let old_tile_size = self.tile_size;
                    let new_tile_size = (old_tile_size * ZOOM_FACTOR.powf(scroll_steps))
                        .clamp(MIN_TILE_SIZE, MAX_TILE_SIZE);
                    let viewport_width = surface.config.width as f64;
                    let viewport_height = surface.config.height as f64;
                    let grid_width = self.grid.column_len() as f64;
                    let grid_height = self.grid.row_len() as f64;

                    // Keep the current viewport center fixed in grid coordinates while zooming.
                    let old_offset_x =
                        (viewport_width - grid_width * old_tile_size) / 2.0 + self.pan_offset.0;
                    let old_offset_y =
                        (viewport_height - grid_height * old_tile_size) / 2.0 + self.pan_offset.1;
                    let center_cell_x = (viewport_width / 2.0 - old_offset_x) / old_tile_size;
                    let center_cell_y = (viewport_height / 2.0 - old_offset_y) / old_tile_size;
                    let new_offset_x = (viewport_width - grid_width * new_tile_size) / 2.0;
                    let new_offset_y = (viewport_height - grid_height * new_tile_size) / 2.0;

                    self.tile_size = new_tile_size;
                    self.pan_offset.0 =
                        viewport_width / 2.0 - new_offset_x - center_cell_x * new_tile_size;
                    self.pan_offset.1 =
                        viewport_height / 2.0 - new_offset_y - center_cell_y * new_tile_size;
                    window.request_redraw();
                }
            }

            // Hold the left mouse button and drag to move the grid around the viewport.
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.is_panning = state == ElementState::Pressed;
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.is_panning {
                    if let Some(last_position) = self.last_cursor_position {
                        self.pan_offset.0 += position.x - last_position.x;
                        self.pan_offset.1 += position.y - last_position.y;
                        window.request_redraw();
                    }
                }
                self.last_cursor_position = Some(position);
            }

            // This is where all the rendering happens
            WindowEvent::RedrawRequested => {
                if !*valid_surface {
                    return;
                }

                // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
                // the same Scene is reused so that the underlying memory allocation can also be reused.
                self.scene.reset();

                // Re-add the objects to draw to the scene.
                draw_grid(
                    &mut self.scene,
                    &self.grid,
                    self.ruleset,
                    self.tile_size,
                    self.pan_offset,
                    surface.config.width as f64,
                    surface.config.height as f64,
                );

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;

                // Get a handle to the device
                let device_handle = &self.context.devices[surface.dev_id];

                // Render to a texture, which we will later copy into the surface
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
                        &surface.target_view,
                        &vello::RenderParams {
                            base_color: palette::css::BLACK, // Background color
                            width,
                            height,
                            // The grid consists of axis-aligned rectangles; area AA is
                            // substantially cheaper than 16x MSAA here.
                            antialiasing_method: AaConfig::Area,
                        },
                    )
                    .expect("failed to render to surface");

                // Get the surface's texture
                let surface_texture = match surface.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
                    CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Suboptimal(_) => {
                        self.context.configure_surface(surface);
                        window.request_redraw();
                        return;
                    }
                    CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => {
                        window.request_redraw();
                        return;
                    }
                    CurrentSurfaceTexture::Lost => panic!("Surface was lost"),
                    CurrentSurfaceTexture::Validation => {
                        panic!("Validation error getting surface")
                    }
                };

                // Perform the copy
                let mut encoder =
                    device_handle
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surface Blit"),
                        });
                surface.blitter.copy(
                    &device_handle.device,
                    &mut encoder,
                    &surface.target_view,
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
                device_handle.queue.submit([encoder.finish()]);
                // Queue the texture to be presented on the surface
                surface_texture.present();
            }
            _ => {}
        }
    }
}

pub fn run_renderer(
    initial_grid: Array2D<i32>,
    updates: Receiver<Array2D<i32>>,
    ruleset: Ruleset,
) -> Result<()> {
    // Setup a bunch of state:
    let mut app = SimpleVelloApp {
        context: RenderContext::new(),
        renderers: vec![],
        state: RenderState::Suspended(None),
        scene: Scene::new(),
        grid: initial_grid,
        tile_size: INITIAL_TILE_SIZE,
        pan_offset: (0.0, 0.0),
        is_panning: false,
        last_cursor_position: None,
        ruleset,
        updates,
    };

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;
    // The simulator sends snapshots from another thread. Polling ensures the
    // renderer checks the receiver promptly instead of sleeping until another
    // OS window event wakes winit.
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
    Ok(())
}

/// Helper function that creates a Winit window and returns it (wrapped in an Arc for sharing between threads)
fn create_winit_window(event_loop: &ActiveEventLoop, rows: usize, columns: usize) -> Arc<Window> {
    let desired_width = columns as f64 * INITIAL_TILE_SIZE;
    let desired_height = rows as f64 * INITIAL_TILE_SIZE;
    let (width, height) = event_loop
        .primary_monitor()
        .map(|monitor| {
            let screen_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let max_width = screen_size.width as f64 / scale_factor * MAX_INITIAL_WINDOW_USAGE;
            let max_height = screen_size.height as f64 / scale_factor * MAX_INITIAL_WINDOW_USAGE;
            (desired_width.min(max_width), desired_height.min(max_height))
        })
        .unwrap_or((desired_width, desired_height));

    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(true)
        .with_title("Cellular Automata");
    Arc::new(event_loop.create_window(attr).unwrap())
}

/// Helper function that creates a vello `Renderer` for a given `RenderContext` and `RenderSurface`
fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface<'_>) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions::default(),
    )
    .expect("Couldn't create renderer")
}

/**
 * NOTE: This function was coded by me.
 */
fn draw_grid(
    scene: &mut Scene,
    grid: &Array2D<i32>,
    ruleset: Ruleset,
    tile_size: f64,
    pan_offset: (f64, f64),
    viewport_width: f64,
    viewport_height: f64,
) {
    let alive_color_conway = Color::new([0.0, 1.0, 0.0, 1.0]); // Green

    let alive_color_brian = Color::new([1.0, 1.0, 0.0, 1.0]); // Yellow
    let alive_color_brian_dying = Color::new([1.0, 0.5, 0.0, 1.0]); // Orange
    let dead_color_brian = Color::new([1.0, 0.0, 0.0, 0.3]); // Red
    let grid_width = grid.column_len() as f64 * tile_size;
    let grid_height = grid.row_len() as f64 * tile_size;
    let offset_x = (viewport_width - grid_width) / 2.0 + pan_offset.0;
    let offset_y = (viewport_height - grid_height) / 2.0 + pan_offset.1;

    for row in 0..grid.row_len() {
        for col in 0..grid.column_len() {
            let cell_state = *grid.get(row, col).unwrap_or(&0);
            let color = match (ruleset, cell_state) {
                (Ruleset::BriansBrain, 1) => alive_color_brian,
                (Ruleset::BriansBrain, 2) => alive_color_brian_dying,
                (Ruleset::BriansBrain, 3) => dead_color_brian,
                (Ruleset::BriansBrain, _) => continue,
                (_, 1) => alive_color_conway,
                (_, _) => continue,
            };

            let x = offset_x + col as f64 * tile_size;
            let y = offset_y + row as f64 * tile_size;
            let rect = Rect::new(x, y, x + tile_size, y + tile_size);

            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &rect,
            );
        }
    }
}
