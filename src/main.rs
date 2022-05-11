use line_drawing::{Bresenham, XiaolinWu};
use nannou::image::{DynamicImage, GenericImage, GenericImageView, Pixel, RgbaImage};
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

enum Mode {
    Move,
    Paint,
}

struct GlobalState {
    scale: f32,
    brush_size: f32,
    mode: Mode,
    last_mouse: Option<Vec2>,
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
        // let mut rng = rand::thread_rng();
        // let mut img = RgbaImage::new(256, 256);
        let mut img = RgbaImage::new(256, 256);
        for (_, _, pixel) in img.enumerate_pixels_mut() {
            // pixel.0 = [rng.gen(), rng.gen(), 255, 255];
            pixel.0 = [255, 255, 255, 255];
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
        scale,
        brush_size,
        brush_size_labels,
        move_mode_button,
        paint_mode_button,
        modes,
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
        global_state: GlobalState {
            scale: 1.75,
            brush_size: 1.0,
            mode: Mode::Move,
            last_mouse: None,
        },
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
                    model.global_state.last_mouse = None;
                    state.offset = translate_mouse_center(app, state.rect);
                }
                ui::RawWindowEvent::CursorMoved { .. } => match model.global_state.mode {
                    Mode::Move => {
                        if state.selected {
                            state.rect = Rect::from_xy_wh(
                                Point2::new(
                                    app.mouse.position().x as _,
                                    app.mouse.position().y as _,
                                ) - state.offset,
                                state.rect.wh(),
                            );
                        }
                    }
                    Mode::Paint => {
                        if state.rect.contains(app.mouse.position()) && state.selected {
                            let mousef = (app.mouse.position() - state.rect.xy())
                                / model.global_state.scale
                                + Vec2::new(state.pixels.width() as _, state.pixels.height() as _)
                                    / 2.0;
                            let mousef =
                                Vec2::new(mousef.x, state.pixels.height() as f32 - mousef.y);

                            let mouse = Vec2::new(
                                mousef.x.round().min(255.0) as _,
                                mousef.y.round().min(255.0) as _,
                            );
                            // state.pixels.put_pixel(
                            //     mouse.0,
                            //     mouse.1,
                            //     nannou::image::Rgba::<u8>::from_channels(0, 0, 0, 255),
                            // );

                            match model.global_state.last_mouse {
                                Some(m) => {
                                    let size = model.global_state.brush_size.round() as i32;
                                    let rad = (model.global_state.brush_size / 2.0).round() as i32;

                                    for (x, y) in Bresenham::<i32>::new(
                                        (m.x as _, m.y as _),
                                        (mouse.x as _, mouse.y as _),
                                    ) {
                                        for i in -rad * 2..rad * 2 {
                                            for j in -rad * 2..rad * 2 {
                                                let dist = mousef.distance(Vec2::new(
                                                    (i + x) as _,
                                                    (y + j) as _,
                                                ));

                                                let opac = ((255.0)
                                                    * (-1.0
                                                        / (model.global_state.brush_size
                                                            * model.global_state.brush_size)
                                                        * (dist * dist) * 2.0
                                                        + 1.0))
                                                    .max(0.0);
                                                let mut pix = state
                                                    .pixels
                                                    .get_pixel((x + i) as u32, (y + j) as u32);
                                                pix.blend(
                                                    &nannou::image::Rgba::<u8>::from_channels(
                                                        0, 0, 0, opac as u8,
                                                    ),
                                                );

                                                state.pixels.put_pixel(
                                                    (x + i) as u32,
                                                    (y + j) as u32,
                                                    pix,
                                                );
                                            }
                                        }
                                    }

                                    // for ((x, y), value) in XiaolinWu::<f32, i32>::new(
                                    //     (m.x, m.y),
                                    //     (mouse.0 as _, mouse.1 as _),
                                    // ) {
                                    //     for i in -size / 2..size / 2 {
                                    //         for j in -size / 2..size / 2 {
                                    //             state.pixels.put_pixel(
                                    //                 (x + i) as u32,
                                    //                 (y + j) as u32,
                                    //                 nannou::image::Rgba::<u8>::from_channels(
                                    //                     0,
                                    //                     0,
                                    //                     0,
                                    //                     (255.0 * value) as _,
                                    //                 ),
                                    //             )
                                    //         }
                                    //     }
                                    // }
                                }
                                None => (),
                            }

                            model.global_state.last_mouse = Some(mousef);
                            // for angle in (0.0 .. 2.0 * f32::PI()) {

                            // }

                            // let mut angle = 0.0;

                            // for r in 0..(model.global_state.brush_size as u32) {
                            //     let r = r as f32;
                            //     while angle < 2.0 * f32::PI() {
                            //         state.pixels.put_pixel(
                            //             ((mouse.0 as f32 + (r * angle.cos()).round()) as u32)
                            //                 .min(255),
                            //             ((mouse.1 as f32 + (r * angle.sin()).round()) as u32)
                            //                 .min(255),
                            //             nannou::image::Rgba::<u8>::from_channels(0, 0, 0, 0),
                            //         );
                            //         angle += 0.00002;
                            //     }
                            //     angle = 0.0;
                            // }
                        }
                    }
                },
                _ => (),
            },
            WindowType::Workbench(_, _) => {}
        }
        window.ui.handle_raw_event(app, event);
        Some(0)
    });

    // match &event {
    //     ui::RawWindowEvent::CursorMoved { .. } => {
    //         model.global_state.last_mouse = app.mouse.position();
    //     }
    //     _ => (),
    // }
}

// fn line(p: &mut DynamicImage, p1: Vec2, p2: Vec2) {
//     let dx = (p2.x - p1.x).abs();
//     let sx = if p1.x < p2.x { 1.0 } else { -1.0 };
//     let dy = -(p2.y - p1.y).abs();
//     let sy = if p1.y < p2.y { 1.0 } else { -1.0 };
//     let mut error = dx + dy;

//     let (mut x0, x1) = (p1.x, p2.x);
//     let (mut y0, y1) = (p1.y, p2.y);

//     loop {
//         p.put_pixel(
//             x0 as _,
//             y0 as _,
//             nannou::image::Rgba::<u8>::from_channels(0, 0, 0, 255),
//         );

//         if x0 == x1 && y0 == y1 {
//             break;
//         }

//         let e2 = 2.0 * error;
//         if e2 >= dy {
//             if x0 == x1 {
//                 break;
//             }
//             error += dy;
//             x0 += sx;
//         }

//         if e2 <= dx {
//             if y0 == y1 {
//                 break;
//             }
//             error += dx;
//             y0 += sy;
//         }
//     }
// }

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

                if let Some(value) = slider(model.global_state.brush_size, 1.0, 100.0)
                    .down(10.0)
                    .label("Brush Size")
                    .set(ids.brush_size, ui)
                {
                    model.global_state.brush_size = value;
                }

                widget::Text::new(format!("{}", model.global_state.brush_size).as_str())
                    .right_from(ids.brush_size, 10.0)
                    .set(ids.brush_size_labels, ui);

                for _click in widget::Button::new()
                    .down_from(ids.brush_size, 10.0)
                    .label("Move")
                    .set(ids.move_mode_button, ui)
                {
                    model.global_state.mode = Mode::Move;
                }

                for _click in widget::Button::new()
                    .label("Paint")
                    .set(ids.paint_mode_button, ui)
                {
                    model.global_state.mode = Mode::Paint;
                }

                // widget::Tabs::new(&[(ids.move_mode_button, "Move"), (
                //     ids.paint_mode_button,
                //     "Paint",
                // )]);
                // .set(ids.modes, ui);
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

                draw.ellipse()
                    .no_fill()
                    .stroke(LinSrgb::new(0.0, 0.0, 0.0))
                    .stroke_weight(1.0)
                    .xy(app.mouse.position())
                    .w_h(
                        model.global_state.brush_size * model.global_state.scale,
                        model.global_state.brush_size * model.global_state.scale,
                    );
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
