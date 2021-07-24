mod buffer_primitives;
mod state;
mod window_target;

use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use state::State;

use futures::executor::block_on;

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
                //state.update();
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

fn main() {
    let result = run();

    if let Err(error) = result {
        error.code().unwrap();
    }
}
