use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use nannou::{
    color::{IntoLinSrgba, LinSrgb, LinSrgba},
    draw::{properties::ColorScalar, theme::Color},
    lyon::geom::{
        euclid::{Point2D, Size2D, UnknownUnit, Vector2D},
        Rect, Size,
    },
    prelude::{Vec2, Vec3},
    state::Mouse,
};

pub trait StateView {
    type StateType: State + Default;
}

pub trait State: Any {
    fn as_any(&self) -> &dyn Any;
    // type State;
    // fn create();
}

pub struct Ui {
    elements: Vec<(Box<dyn View>, Rc<RefCell<dyn State>>)>,
    ui_func: fn(&mut Ui),
    index: usize,
}

impl Ui {
    pub fn new(ui_func: fn(&mut Ui)) -> Ui {
        Ui {
            elements: vec![],
            ui_func,
            index: 0,
        }
    }

    pub fn add_element<V>(&mut self, element: V)
    where
        V: View + StateView + 'static,
    {
        if self.elements.capacity() > 0 {
            let state = self.elements[self.index].1.clone();
            self.elements[self.index].0 = Box::new(element);
            self.elements[self.index].0.set_state(state);
            self.index += 1;
        } else {
            self.elements.push((
                Box::new(element),
                Rc::new(RefCell::new(<V as StateView>::StateType::default())),
            ));
        }
    }

    pub fn update(&mut self) {
        // if self.elements.len() > 0 {
        //     let views: Vec<_> = self.elements.iter().map(|f| f.1.as_ref()).collect();
        //     self.elements.clear();
        // }
        self.index = 0;
        (self.ui_func)(self);
    }

    pub fn draw_to_frame(&self, app: &nannou::App, frame: &nannou::Frame) {
        let draw = app.draw();
        draw.xy(Vec2::new(0.0, 100.0));
        for (element, state) in self.elements.iter() {
            element.draw(app, &draw);
        }

        draw.to_frame(app, &frame).unwrap();
    }

    pub fn window_event(&mut self, app: &nannou::App, event: &nannou::winit::event::WindowEvent) {
        match event {
            nannou::winit::event::WindowEvent::CursorMoved { position, .. } => {
                for (element, _) in self.elements.iter_mut() {
                    element.on_mouse_move(app, &app.mouse);
                }
            }
            nannou::winit::event::WindowEvent::MouseInput { state, .. } => {
                let position = app.mouse.position();
                let position = Point2D::new(position.x as _, position.y as _);
                for (element, _) in self.elements.iter_mut() {
                    if element.get_rect().contains(position) {
                        match state {
                            nannou::event::ElementState::Pressed => {
                                element.on_mouse_press(app, &app.mouse)
                            }
                            nannou::event::ElementState::Released => {
                                element.on_mouse_release(app, &app.mouse)
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    pub fn translate_mouse(app: &nannou::App, view: &impl View) -> Vector2D<i32, UnknownUnit> {
        let rect = view.get_rect();
        -(rect.origin - Point2D::new(app.mouse.x as i32, app.mouse.y as i32))
    }

    pub fn translate_mouse_center(
        app: &nannou::App,
        view: &impl View,
    ) -> Vector2D<i32, UnknownUnit> {
        let rect = view.get_rect();
        let pos = -(rect.origin - Point2D::new(app.mouse.x as i32, app.mouse.y as i32));
        Vector2D::new(pos.x - rect.size.width / 2, pos.y - rect.size.height / 2)
    }
}

pub trait View {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw);

    fn on_mouse_enter(&mut self, app: &nannou::App, mouse: &Mouse) {}
    fn on_mouse_exit(&mut self, app: &nannou::App, mouse: &Mouse) {}
    fn on_mouse_move(&mut self, app: &nannou::App, mouse: &Mouse) {}
    fn on_mouse_drag(&mut self, app: &nannou::App, mouse: &Mouse) {}
    fn on_mouse_click(&mut self, app: &nannou::App, mouse: &Mouse) {}
    fn on_mouse_press(&mut self, app: &nannou::App, mouse: &Mouse) {}
    fn on_mouse_release(&mut self, app: &nannou::App, mouse: &Mouse) {}

    fn get_rect(&self) -> Rect<i32> {
        Default::default()
    }

    fn set_state(&mut self, state: Rc<RefCell<dyn State>>) {}
}

pub struct Panel {
    state: Rc<RefCell<<Self as StateView>::StateType>>,
    background: LinSrgba,
}

impl Panel {
    pub fn new() -> Panel {
        Panel {
            state: Rc::new(Default::default()),
            background: LinSrgba::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    fn frame(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.state.borrow_mut().rect = Rect {
            origin: Point2D::new(x, y),
            size: Size2D::new(width, height),
        };
        self
    }

    fn background<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.background = color.into_lin_srgba();
        self
    }
}

impl View for Panel {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw) {
        let win = app.window_rect();

        draw.rect()
            .xy(Vec2::new(
                self.state.borrow().rect.origin.x as _,
                self.state.borrow().rect.origin.y as _,
            ))
            .w_h(
                self.state.borrow().rect.size.width as _,
                self.state.borrow().rect.size.height as _,
            )
            .color(self.background);
    }

    fn on_mouse_move(&mut self, app: &nannou::App, mouse: &Mouse) {
        let select = { self.state.borrow().selected };
        if select {
            let pos = app.mouse.position();
            let offset = self.state.borrow().offset.unwrap();
            self.state.borrow_mut().rect.origin = Point2D::new(pos.x as i32, pos.y as i32)
                .add_size(&-Size2D::new(offset.0, offset.1));
        }
    }

    fn on_mouse_press(&mut self, app: &nannou::App, mouse: &Mouse) {
        if mouse.buttons.left().is_down() {
            self.state.borrow_mut().selected = true;
            self.state.borrow_mut().offset = Some(Ui::translate_mouse_center(app, self).to_tuple())
        }
    }

    fn on_mouse_release(&mut self, app: &nannou::App, mouse: &Mouse) {
        self.state.borrow_mut().selected = false;
        self.state.borrow_mut().offset = None;
    }

    fn get_rect(&self) -> Rect<i32> {
        nannou::lyon::geom::euclid::Rect {
            origin: self.state.borrow().rect.origin
                - Vector2D::new(
                    self.state.borrow().rect.size.width / 2,
                    self.state.borrow().rect.size.height / 2,
                ),
            size: self.state.borrow().rect.size,
        }
    }

    fn set_state(&mut self, state: Rc<RefCell<dyn State>>) {
        self.state = try_downcast_rc_refcell_wrapper(state).unwrap();
    }
}

pub struct PanelState {
    pub rect: Rect<i32>,
    pub offset: Option<(i32, i32)>,
    pub selected: bool,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            rect: Rect::new(Point2D::new(0, 0), Size2D::new(100, 100)),
            offset: None,
            selected: false,
        }
    }
}

impl State for PanelState {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl StateView for Panel {
    type StateType = PanelState;
}

fn try_downcast_rc_refcell_wrapper<T: State>(
    rc: Rc<RefCell<dyn State>>,
) -> Result<Rc<RefCell<T>>, ()> {
    Ok(unsafe {
        fn _sanity_check(rc: Rc<RefCell<impl State>>) -> Rc<RefCell<dyn State>> {
            rc // Unsize coercion passes.
        }

        Rc::from_raw(Rc::into_raw(rc) as *const RefCell<T>)
    })
}
