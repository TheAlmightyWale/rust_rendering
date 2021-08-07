mod buffer_primitives;
mod state;
mod texture;
mod window_target;

use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use state::Sphere;
use state::State;

use futures::executor::block_on;

static MIN_Z: f32 = 1.0;

fn run() -> windows::Result<()> {
    windows::initialize_sta()?; //Single thread application

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("software-raytracer");

    //Initialize actual activity here
    let mut state = block_on(State::new(&window));
    //finish game state intialize

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                ray_trace(&mut state); // trace once for debugging
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

fn ray_trace(state: &mut State) {
    //Get bounds of drawing sruface
    let viewport_width = state.texture.size.width as f32;
    let viewport_height = state.texture.size.height as f32;
    let size = cgmath::Vector2::<f32> {
        x: viewport_width,
        y: viewport_height,
    };
    let origin = cgmath::Vector3::new(0.0, 0.0, 0.0);

    for y in 0..state.texture.size.width {
        for x in 0..state.texture.size.height {
            //Centering x and y gives us a camera view centered at 0,0,0, rather than having the far left of the view starting at 0,0,0
            let centered_x = x as f32 - (viewport_width / 2.0);
            let centered_y = y as f32 - (viewport_height / 2.0);
            let direction = canvas_to_viewport(centered_x, centered_y, size);
            let color = trace_ray(&origin, &direction, 1.0, f32::INFINITY, state);
            state.set_pixel(x, y, &color);
        }
    }
}

fn canvas_to_viewport(x: f32, y: f32, size: cgmath::Vector2<f32>) -> cgmath::Vector3<f32> {
    cgmath::Vector3::<f32> {
        x: (x / size.x),
        y: (y / size.y),
        z: MIN_Z,
    }
}

//min and max distance are measured as the parameter t in the vector equation P = Q + t(V - Q), where V and Q are 2 points
fn trace_ray(
    origin: &cgmath::Vector3<f32>,
    direction: &cgmath::Vector3<f32>,
    min_distance: f32,
    max_distance: f32,
    world: &State,
) -> [u8; 4] {
    let mut closest_t = f32::INFINITY;
    let mut closest_sphere: Option<&Sphere> = None;

    for sphere in world.spheres.iter() {
        let determinants = intersect_ray_sphere(origin, direction, sphere);
        if (min_distance..max_distance).contains(&determinants.0) && determinants.0 < closest_t {
            closest_t = determinants.0;
            closest_sphere = Some(&sphere);
        }

        if (min_distance..max_distance).contains(&determinants.1) && determinants.1 < closest_t {
            closest_t = determinants.1;
            closest_sphere = Some(&sphere);
        }
    }

    match closest_sphere {
        Some(sphere) => sphere.color,
        None => [0, 0, 0, 255],
    }
}

//returns the determinants of the quadratic equation, f32::INFINITY(no intersection), both determinants equal (tangent), two solutions (intersection)
fn intersect_ray_sphere(
    origin: &cgmath::Vector3<f32>,
    direction: &cgmath::Vector3<f32>,
    sphere: &Sphere,
) -> (f32, f32) {
    let r = sphere.radius;
    let origin_sphere = origin - sphere.center;

    //Quadratic equation
    let a = cgmath::dot(*direction, *direction);
    let b = 2.0 * cgmath::dot(origin_sphere, *direction);
    let c = cgmath::dot(origin_sphere, origin_sphere) - r * r;

    let discriminant = b * b - 4.0 * a * c;
    match discriminant {
        d if d < 0.0 => (f32::INFINITY, f32::INFINITY),
        __ => {
            let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b - discriminant.sqrt()) / (2.0 * a);
            (t1, t2)
        }
    }
}

fn main() {
    let result = run();

    if let Err(error) = result {
        error.code().unwrap();
    }
}
