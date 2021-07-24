fn main() {
    windows::build! {
        Windows::Graphics::SizeInt32,
        Windows::Win32::System::WinRT::{
            CreateDispatcherQueueController, ICompositorDesktopInterop, ICompositorInterop
        },
        Windows::UI::Colors,
        Windows::UI::Composition::Desktop::DesktopWindowTarget,
        Windows::UI::Composition::{
            CompositionColorBrush, CompositionContainerShape, CompositionEllipseGeometry,
            CompositionNineGridBrush, Compositor, ShapeVisual, SpriteVisual, Vector3KeyFrameAnimation,
            VisualCollection, CompositionGraphicsDevice
        },
    };
}
