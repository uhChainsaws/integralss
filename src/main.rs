#![allow(non_snake_case)]

use std::str::FromStr;

use nannou::{prelude::*};
use nannou_egui::{self, egui::{self, Align2}, Egui};
use rand::prelude::*;

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PickMode {
    LOW,
    HIGH,
    RANDOM,
    CUSTOM
}

struct Model {
    f: fn(&f32) -> Option<f32>,
    n: u32,
    a: f32,
    b: f32,
    pick_mode: PickMode,
    custom_pick: f32,
    randomness_seed: String,
    view_zoom: f32,
    show_area: bool,
    draw_helpers: bool,
    // area: f32,
    egui: Egui,
}

fn update(_app: &App, model: &mut Model, update: Update) {
    egui_majjikks(model, update)
}
fn egui_majjikks(model: &mut Model, update: Update){
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    
    egui::Window::new("settings").anchor(Align2::RIGHT_BOTTOM, [-20.,-20.]).show(&ctx, |ui| {
        ui.add(egui::Slider::new(&mut model.n, 1..=2000).text("n"));
        ui.add(egui::Slider::new(&mut model.a, -10.0..=model.b-0.1).text("a"));
        ui.add(egui::Slider::new(&mut model.b, model.a+0.1..=10.0).text("b"));
        let pick_mode = &mut model.pick_mode;
        ui.horizontal(|ui| {
            ui.label("pick_mode");
            ui.selectable_value(pick_mode, PickMode::LOW, "LOW");
            ui.selectable_value(pick_mode, PickMode::HIGH, "HIGH");
            ui.selectable_value(pick_mode, PickMode::RANDOM, "RANDOM");
            ui.selectable_value(pick_mode, PickMode::CUSTOM, "CUSTOM");
        });
        ui.add(egui::Slider::new(&mut model.custom_pick, 0.0..=1.0).text("custom_pick"));
        ui.add(egui::Label::new("randomness_seed"));
        ui.add(egui::TextEdit::singleline(&mut model.randomness_seed));
        ui.add(egui::Slider::new(&mut model.view_zoom, 0.1..=10.0).text("view_width_scale"));
        ui.add(egui::Checkbox::new(&mut model.show_area, "show_area"));
        ui.add(egui::Checkbox::new(&mut model.draw_helpers, "draw_helpers"));
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let INABAKUMORI = rgb(0.7, 0.7, 0.7);
    let draw = app.draw();
    draw.background().color(BLACK);
    let mut rng = StdRng::seed_from_u64(model.randomness_seed.as_bytes().iter().fold(0, |acc, x| acc + *x as u64));

    let X = (0..=model.n).map(|i| model.a + (model.b - model.a) * (i as f32) / (model.n as f32));
    let X_points: Vec<f32> = match model.pick_mode{
        PickMode::LOW => {
            X.clone().collect()
        },
        PickMode::HIGH => {
            X.clone().map(|x| x+1./model.n as f32).collect()
        },
        PickMode::RANDOM => {
            X.clone().map(|x| rng.gen_range(x..(x+1./model.n as f32))).collect()
        },
        PickMode::CUSTOM => {
            X.clone().map(|x| x+model.custom_pick/model.n as f32).collect()
        }
    };
    let Y: Vec<Option<f32>> = X_points.iter().map(model.f).collect();
    let max_y_deviation: f32 = Y.iter().max_by(|x, y| 
        x.unwrap_or(0.).abs().partial_cmp(&y.unwrap_or(0.).abs()).unwrap()
    ).unwrap().unwrap();

    let vw = model.b-model.a;
    let vh = 2.*max_y_deviation.abs();
    let scale_x = app.window_rect().w()/(model.view_zoom*vw);
    let scale_y = app.window_rect().h()/(vh*model.view_zoom);
    let zero_x = (0.-(vw/2.+model.a))*scale_x;
    
    draw.line()
        .start(pt2(-app.window_rect().w()/2., 0.))
        .end(pt2(app.window_rect().w()/2., 0.))
        .weight(2.0)
        .color(rgba(1., 1., 1., 0.5));
    
    if model.draw_helpers{
        draw.line()
            .start(pt2(zero_x, -app.window_rect().h()/2.))
            .end(pt2(zero_x, app.window_rect().h()/2.))
            .weight(2.0)
            .color(rgba(1., 1., 1., 0.5));
        for i in 0..=20 {
            let x = (i as f32 - 10.)*vw/20.;
            let x_pos = (x-(vw/2.+model.a))*scale_x;
            draw.line()
                .start(pt2(x_pos, -5.))
                .end(pt2(x_pos, 5.))
                .weight(2.0)
                .color(rgba(1., 1., 1., 0.5));
            if i%2 == 0 {
                draw.text(&format!("{:.2}", x))
                    .x_y(x_pos, 10.)
                    .color(rgba(1., 1., 1., 0.5))
                    .font_size(20);
            }
        }
        // draw vertical tickmarks, do not draw zero
        // offset text to the right by 40px
        for i in 0..=20 {
            if i == 10 {continue;}
            let y = (i as f32 - 10.)*vh/20.;
            let y_pos = y*scale_y;
            draw.line()
                .start(pt2(zero_x-4., y_pos))
                .end(pt2(zero_x+4., y_pos))
                .weight(2.0)
                .color(rgba(1., 1., 1., 0.5));
            if i%2 == 0 {
                draw.text(&format!("{:.2}", y))
                    .x_y(zero_x-40., y_pos)
                    .color(rgba(1., 1., 1., 0.5))
                    .font_size(20);
            }
        }
    }
    draw.polyline()
        .weight(2.0)
        .color(INABAKUMORI)
        .points_colored(
            X_points.iter().zip(Y.iter()).map(|(x, y)| {
                let color = hsl(
                    (x/vw - app.time*0.1)%360.,
                    0.7,
                    0.5
                );
                (pt2(
                    (x-(vw/2.+model.a))*scale_x,
                    y.unwrap_or(0.)*scale_y
                ), color)
            }
        )
    );
        
    if model.show_area {
        let mut area = 0.;
        for i in 0..X_points.len()-1 {
            let x1 = X_points[i];
            let x2 = X_points[i+1];
            let y1 = (model.f)(&x1).unwrap_or(0.);
            let width = x2-x1;
            let height = y1;
            let this_area = width.abs()*height;
            area += this_area;
            draw.rect()
                .x_y(
                    (x1-(vw/2.+model.a))*scale_x + width*scale_x/2.,
                    y1*scale_y - height*scale_y/2.
                )
                .w_h(
                    width*scale_x,
                    height*scale_y
                )
                .color(if this_area > 0. {
                    hsla(
                        120./360.,
                        0.7,
                        (app.time - x1/vw).sin()/4. + 0.25,
                        0.5
                    )
                    } else {
                    hsla(
                        7./360.,
                        0.7,
                        (app.time - x1/vw).sin()/4. + 0.25,
                        0.5
                    )
                });
        }
        draw.rect()
            .x_y(
                0.,
                -app.window_rect().h()/2. + 20.
            )
            .w_h(
                100.,
                40.
            )
            .color(BLACK);
        draw.text(&format!("{:.3}", area))
            .x_y(
                0.,
                -app.window_rect().h()/2. + 20.
            )
            .font_size(30)
            .color(WHITE);
        
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size_pixels(1600, 950)
        .title("uhIntegrals")
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    fn f (x: &f32) -> Option<f32>{
        Some(-x.powi(3) + 2.0 * x)
    }
    Model { 
        f,
        n: 20,
        a: -1.,
        b: 2.,
        pick_mode: PickMode::LOW,
        custom_pick: 0.5,
        randomness_seed: String::from_str("integrali blin(").unwrap(),
        view_zoom: 1.1,
        show_area: true,
        draw_helpers: false,
        // area: 0.,
        egui,
    }
}

fn main() {
    println!("sanity: checked!");
    nannou::app(model)
        .view(view)
        .update(update)
        .run();    
}
