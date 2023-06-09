pub struct Ui {
    server_handle: crate::server::handler::ServerHandle,
}

pub fn run() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(500.0, 350.0)), /*x800y450 is 16:9*/
        resizable: false,
        centered: true,
        vsync: true,
        decorated: false,
        transparent: true,
        // always_on_top: true,
        default_theme: eframe::Theme::Dark,

        ..Default::default()
    };
    eframe::run_native(
        "Muc client",
        options,
        Box::new(|cc| {
            use eframe::egui::{
                FontFamily::{Monospace, Proportional},
                FontId, TextStyle,
            };

            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles = [
                (TextStyle::Heading, FontId::new(25.0, Proportional)),
                (TextStyle::Body, FontId::new(16.0, Proportional)),
                (TextStyle::Monospace, FontId::new(16.0, Monospace)),
                (TextStyle::Button, FontId::new(16.0, Proportional)),
                (TextStyle::Small, FontId::new(8.0, Proportional)),
            ]
            .into();
            cc.egui_ctx.set_style(style);
            Box::<Ui>::new(Ui {
                server_handle: crate::server::handler::ServerHandle::new(),
            })
        }),
    )
    .unwrap();
}

impl Ui {
    fn render_title_bar(
        &mut self,
        ui: &mut eframe::egui::Ui,
        frame: &mut eframe::Frame,
        title_bar_rect: eframe::epaint::Rect,
        title: &str,
    ) {
        let painter = ui.painter();

        let title_bar_response = ui.interact(
            title_bar_rect,
            eframe::egui::Id::new("title_bar"),
            eframe::egui::Sense::click(),
        );

        // Paint the title:
        painter.text(
            title_bar_rect.center(),
            eframe::emath::Align2::CENTER_CENTER,
            title,
            eframe::epaint::FontId::proportional(20.0),
            ui.style().visuals.text_color(),
        );

        // Paint the line under the title:
        painter.line_segment(
            [
                title_bar_rect.left_bottom() + eframe::epaint::vec2(1.0, 0.0),
                title_bar_rect.right_bottom() + eframe::epaint::vec2(-1.0, 0.0),
            ],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        // Interact with the title bar (drag to move window):
        if title_bar_response.double_clicked() {
            // frame.set_maximized(!frame.info().window_info.maximized);
        } else if title_bar_response.is_pointer_button_down_on() {
            frame.drag_window();
        }

        // Show toggle button for light/dark mode
        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(
                eframe::egui::Layout::left_to_right(eframe::egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.visuals_mut().button_frame = false;
                    ui.add_space(8.0);
                    eframe::egui::widgets::global_dark_light_mode_switch(ui);
                },
            );
        });

        // Show some close/maximize/minimize buttons for the native window.
        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(
                eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.visuals_mut().button_frame = false;
                    ui.add_space(8.0);

                    let button_height = 12.0;

                    if ui
                        .add(eframe::egui::Button::new(
                            eframe::egui::RichText::new("❌").size(button_height),
                        ))
                        .on_hover_text("Close the window")
                        .clicked()
                    {
                        frame.close();
                    }

                    let (hover_text, clicked_state) = if frame.info().window_info.maximized {
                        ("Restore window", false)
                    } else {
                        ("Maximize window", true)
                    };

                    if ui
                        .add(eframe::egui::Button::new(
                            eframe::egui::RichText::new("🗗").size(button_height),
                        ))
                        .on_hover_text(hover_text)
                        .clicked()
                    {
                        frame.set_maximized(clicked_state);
                    }

                    if ui
                        .add(eframe::egui::Button::new(
                            eframe::egui::RichText::new("🗕").size(button_height),
                        ))
                        .on_hover_text("Minimize the window")
                        .clicked()
                    {
                        frame.set_minimized(true);
                    }
                },
            );
        });
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.server_handle.update();
        ctx.request_repaint();

        eframe::egui::CentralPanel::default()
            .frame(
                eframe::egui::Frame::none()
                    .fill(ctx.style().visuals.window_fill())
                    .rounding(10.0)
                    .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
                    .outer_margin(0.5),
            )
            .show(ctx, |ui| {
                let app_rect = ui.max_rect();

                // draw the title bar

                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + 32.0;
                    rect
                };
                self.render_title_bar(ui, frame, title_bar_rect, "Installer");

                // // rest of the window
                // let content_rect = {
                //     let mut rect = app_rect;
                //     rect.min.y = title_bar_rect.max.y;
                //     rect
                // }
                // .shrink(4.0);
                // let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            });
        eframe::egui::TopBottomPanel::bottom("Bottom panel")
            .frame(
                eframe::egui::Frame::none()
                    .fill(eframe::egui::Color32::TRANSPARENT)
                    .rounding(10.0)
                    .stroke(eframe::egui::Stroke::NONE)
                    .outer_margin(0.5)
                    .inner_margin(10.),
            )
            .show(ctx, |ui| ui.horizontal(|ui| ui.label("bottom pannel")));
    }
}
