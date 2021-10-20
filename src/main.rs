mod buffer_primitives;
mod lights;
mod objects;
mod properties;
mod rasterizer;
mod ray_tracer;
mod scene;
mod serialization_defs;
mod state;
mod texture;

use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use futures::executor::block_on;
use properties::Color;
use rasterizer::clear_screen;
use rasterizer::draw_line;
use ray_tracer::ray_trace;
use scene::Scene;
use state::State;
use state::Surface;

use std::fs;

enum RenderType {
    RayTraced,
    Rasterized,
}

fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("software-graphics");

    //Initialize actual activity here
    let mut state = Box::new(block_on(State::new(&window)));
    //finish game state intialize

    let scene_filename = "scene.json";
    let scene_json = fs::read_to_string(scene_filename)
        .unwrap_or_else(|_| panic!("Could not read file {}", scene_filename));
    let scene = Scene::load(&scene_json).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                //Render updates state to be drawn, should probably live in state
                render(RenderType::Rasterized, &mut state.pixel_surface, &scene);
                state.update();
                match state.render() {
                    Ok(_) => {}
                    //Lost swapchain, re-create it
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    Err(wgpu::SwapChainError::OutOfMemory) => {
                        panic!("Out of memory when attempting to Render!")
                    }
                    Err(error) => eprintln!("{:?}", error),
                }
            }
            //Call redraw every time we have finished processing other events
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                state.resize(size);
            }
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                ..
            } => {
                state.resize(*new_inner_size);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { .. },
                ..
            } => {
                print!("Mouse move!");
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, .. },
                ..
            } => {
                if state == ElementState::Pressed {
                    print!("Mouse clicked!");
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                //state.input(input);
            }
            _ => (),
        }
    });
}

fn render(render_mode: RenderType, surface: &mut dyn Surface, scene: &Scene) {
    match render_mode {
        RenderType::RayTraced => ray_trace(&scene, surface),
        RenderType::Rasterized => {
            let p1 = cgmath::Vector2::<f32> { x: 10.0, y: 10.0 };
            let p2 = cgmath::Vector2::<f32> { x: 200.0, y: 100.0 };
            let color = Color::<u8> {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            };
            clear_screen(surface);
            draw_line(p1, p2, color, surface);
            let p3 = cgmath::Vector2::<f32> { x: 15.0, y: 200.0 };
            draw_line(p1, p3, color, surface);
        }
    }
}

fn main() {
    run();
}
