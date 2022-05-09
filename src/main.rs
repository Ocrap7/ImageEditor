use nannou::image::{DynamicImage, RgbaImage};
use nannou::prelude::Rect;
use nannou::prelude::*;
use nannou_conrod as ui;
use nannou_conrod::prelude::*;
use rand::Rng;
use std::collections::HashMap;

struct Window {
    pub id: WindowId,
    pub ui: Ui,
    pub widget_ids: WindowType,
}

trait Init<T> {
    fn new(app: &App, title: &str) -> Self;
}

impl Init<EditorIds> for Window {
    fn new(app: &App, title: &str) -> Window {
        let w_id = app
            .new_window()
            .title(title)
            .raw_event(raw_window_event)
            .view(view)
            .build()
            .unwrap();

        let mut ui = ui::builder(app).window(w_id).build().unwrap();
        let generator = ui.widget_id_generator();

        Window {
            id: w_id,
            widget_ids: WindowType::Editor(EditorIds::new(generator), Default::default()),
            ui,
        }
    }
}

impl Init<WorkbenchIds> for Window {
    fn new(app: &App, title: &str) -> Window {
        let w_id = app
            .new_window()
            .title(title)
            .raw_event(raw_window_event)
            .view(view)
            .build()
            .unwrap();

        let mut ui = ui::builder(app).window(w_id).build().unwrap();
        let generator = ui.widget_id_generator();

        Window {
            id: w_id,
            widget_ids: WindowType::Workbench(WorkbenchIds::new(generator), Default::default()),
            ui,
        }
    }
}

fn main() {
    env_logger::init();

    nannou::app(model).update(update).run();
}

struct Model {
    windows: HashMap<WindowId, Window>,
    global_state: GlobalState,
}

struct GlobalState {
    scale: f32,
}

widget_ids! {
    struct EditorIds {
    }
}

struct EditorState {
    offset: Point2,
    selected: bool,
    pixels: DynamicImage,

    rect: Rect<f32>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let mut img = RgbaImage::new(256, 256);
        for (_, _, pixel) in img.enumerate_pixels_mut() {
            pixel.0 = [rng.gen(), rng.gen(), 255, 255];
        }
        Self {
            offset: Point2::new(0.0, 0.0),
            selected: false,
            pixels: DynamicImage::ImageRgba8(img),
            rect: nannou::prelude::Rect::from_x_y_w_h(0.0, 0.0, 256.0, 256.0),
        }
    }
}

widget_ids! {
    struct WorkbenchIds {
        scale
    }
}

struct WorkBenchState {}

impl Default for WorkBenchState {
    fn default() -> Self {
        Self {}
    }
}

enum WindowType {
    Editor(EditorIds, EditorState),
    Workbench(WorkbenchIds, WorkBenchState),
}

fn model(app: &App) -> Model {
    // Set the loop mode to wait for events, an energy-efficient option for pure-GUI apps.
    app.set_loop_mode(LoopMode::Wait);

    let editor_window = <Window as Init<EditorIds>>::new(app, "Editor");
    let workbench_window = <Window as Init<WorkbenchIds>>::new(app, "Workbench");

    let mut map = HashMap::default();
    map.insert(editor_window.id, editor_window);
    map.insert(workbench_window.id, workbench_window);

    Model {
        windows: map,
        global_state: GlobalState { scale: 1.0 },
    }
}

fn raw_window_event(app: &App, model: &mut Model, event: &ui::RawWindowEvent, id: WindowId) {
    model.windows.get_mut(&id).map(|window| {
        match &mut window.widget_ids {
            WindowType::Editor(_, state) => match &event {
                ui::RawWindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::PixelDelta(d) => {
                        model.global_state.scale = (model.global_state.scale
                            + d.y as f32 / 10.0 * model.global_state.scale)
                            .clamp(1.0, 100.0);
                    }
                    MouseScrollDelta::LineDelta(_, y) => {
                        model.global_state.scale = (model.global_state.scale
                            + *y as f32 / 10.0 * model.global_state.scale)
                            .clamp(1.0, 100.0);
                    }
                },
                ui::RawWindowEvent::MouseInput {
                    button: nannou::event::MouseButton::Left,
                    state: bstate,
                    ..
                } => {
                    state.selected = match bstate {
                        nannou::event::ElementState::Pressed => true,
                        nannou::event::ElementState::Released => false,
                    };
                    state.offset = translate_mouse_center(app, state.rect);
                }
                ui::RawWindowEvent::CursorMoved { .. } => {
                    if state.selected {
                        state.rect = Rect::from_xy_wh(
                            Point2::new(app.mouse.position().x as _, app.mouse.position().y as _)
                                - state.offset,
                            state.rect.wh(),
                        );
                    }
                }
                _ => (),
            },
            WindowType::Workbench(_, _) => {}
        }
        window.ui.handle_raw_event(app, event);
        Some(0)
    });
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Calling `set_widgets` allows us to instantiate some widgets.
    for window in model.windows.values_mut() {
        let ui = &mut window.ui.set_widgets();
        match &mut window.widget_ids {
            WindowType::Editor(_, state) => {
                state.rect = Rect::from_xy_wh(
                    state.rect.xy(),
                    Point2::new(
                        state.pixels.as_rgba8().unwrap().width() as f32 * model.global_state.scale,
                        state.pixels.as_rgba8().unwrap().height() as f32 * model.global_state.scale,
                    ),
                );
            }
            WindowType::Workbench(ids, _) => {
                fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
                    widget::Slider::new(val, min, max)
                        .w_h(200.0, 30.0)
                        .label_font_size(15)
                        .rgb(0.3, 0.3, 0.3)
                        .label_rgb(1.0, 1.0, 1.0)
                        .border(0.0)
                }

                if let Some(value) = slider(model.global_state.scale, 1.0, 100.0)
                    .top_left_with_margin(20.0)
                    .label("Scale")
                    .set(ids.scale, ui)
                {
                    model.global_state.scale = value;
                }
            }
        }
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    // frame.win
    model.windows.get(&frame.window_id()).map(|window| {
        match &window.widget_ids {
            WindowType::Editor(_, state) => {
                let draw = app.draw();
                draw.background().rgb(0.15, 0.15, 0.15);

                let sampler = wgpu::SamplerBuilder::new()
                    .address_mode(wgpu::AddressMode::ClampToEdge)
                    .mag_filter(wgpu::FilterMode::Nearest)
                    .into_descriptor();

                let draw = draw.sampler(sampler);

                let canvas = wgpu::Texture::from_image(app, &state.pixels);
                draw.texture(&canvas)
                    .wh(state.rect.wh())
                    .xy(state.rect.xy());
                // println!("View Editor {:?}", state.rect);

                // Write the result of our drawing to the window's frame.
                draw.to_frame(app, &frame).unwrap();

                // Draw the state of the `Ui` to the frame.
                window.ui.draw_to_frame(app, &frame).unwrap();
            }
            WindowType::Workbench(_, _) => {
                let draw = app.draw();
                draw.background().rgb(0.15, 0.15, 0.15);
                draw.to_frame(app, &frame).unwrap();
                // println!("View workbench");

                // println!("View Workbench");

                window.ui.draw_to_frame(app, &frame).unwrap();
            }
        }
        Some(0)
    });
}

pub fn translate_mouse_center(app: &nannou::App, rect: Rect<f32>) -> Point2 {
    let pos = -(rect.xy() - Point2::new(app.mouse.x as _, app.mouse.y as _));
    Point2::new(pos.x, pos.y)
}
