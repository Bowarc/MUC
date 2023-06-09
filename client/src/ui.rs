use std::fs::DirEntry;

pub struct Ui {
    server_handle: crate::server::handler::ServerHandle,

    username_text: String,
    password_text: String,
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
            Box::<Ui>::new(Ui::new())
        }),
    )
    .unwrap();
}

impl Ui {
    fn new() -> Self {
        Self {
            server_handle: crate::server::handler::ServerHandle::new(),
            username_text: String::new(),
            password_text: String::new(),
        }
    }

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

    fn render_waiting_screen(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("Waiting for the server to accept comunication");
    }

    fn render_account(&mut self, ctx: &eframe::egui::Context, ui: &mut eframe::egui::Ui) {
        let mut move_to_dir_opt = None;
        if let Some(scan) = &self.server_handle.account_state.as_ref().unwrap().fs {
            // debug!("Rendering {scan:?}");

            eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    if ui.button("..".to_string()).clicked() {
                        move_to_dir_opt = Some(String::from(".."))
                    }
                });

                for directory in &scan.directories {
                    ui.vertical_centered(|ui| {
                        if ui
                            .button(format!("Directory: {}", directory.name))
                            .clicked()
                        {
                            debug!("Clicked {}", directory.name);

                            move_to_dir_opt = Some(directory.name.clone())
                        }
                    });
                }

                for file in &scan.files {
                    ui.vertical_centered(|ui| {
                        if ui.button(format!("File: {}", file.name)).clicked() {
                            debug!("Clicked {}", file.name)
                        }
                    });
                }
            });
        } else {
            debug!("Did not received the file scan yet")
        }

        if let Some(move_to_dir) = move_to_dir_opt {
            self.server_handle.cd(move_to_dir);
        }
    }

    fn render_login(&mut self, _ctx: &eframe::egui::Context, ui: &mut eframe::egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label("Username");
                ui.text_edit_singleline(&mut self.username_text)
            });

            ui.horizontal(|ui| {
                ui.label("Password");
                ui.text_edit_singleline(&mut self.password_text)
            });

            if ui.button("Login").clicked() {
                debug!(
                    "Login with user: {}, pw: {}",
                    self.username_text, self.password_text
                );
                self.server_handle
                    .login(&self.username_text, &self.password_text);
            }
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

                if self.server_handle.account_state.is_some() {
                    self.render_account(ctx, ui);
                } else {
                    self.render_login(ctx, ui)
                }
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
