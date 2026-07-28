impl DesktopApp {
    fn render_viewport(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND))
            .show(root, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                painter.rect_stroke(
                    response.rect,
                    0.0,
                    Stroke::new(1.0, BORDER),
                    StrokeKind::Inside,
                );
                self.handle_viewport_gestures(&ctx, &response);
                let Some(checkpoint) = self.maybe_display_checkpoint() else {
                    painter.text(
                        response.rect.left_top() + Vec2::new(20.0, 20.0),
                        Align2::LEFT_TOP,
                        "Step the scenario to render a semantic checkpoint",
                        FontId::proportional(18.0),
                        MUTED,
                    );
                    return;
                };
                let maybe_pair = self
                    .diagnostics
                    .maybe_comparison()
                    .zip(self.maybe_oracle.as_ref());
                match (self.comparison_mode, maybe_pair) {
                    (ComparisonMode::SideBySide, Some((comparison, oracle))) => {
                        let gap = 8.0;
                        let width = (response.rect.width() - gap) * 0.5;
                        let rust_rect = Rect::from_min_size(
                            response.rect.min,
                            Vec2::new(width, response.rect.height()),
                        );
                        let oracle_rect = Rect::from_min_size(
                            Pos2::new(rust_rect.max.x + gap, response.rect.min.y),
                            rust_rect.size(),
                        );
                        paint_checkpoint(
                            &painter,
                            checkpoint,
                            rust_rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Rust)),
                        );
                        paint_checkpoint(
                            &painter,
                            oracle,
                            oracle_rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Oracle)),
                        );
                    }
                    (ComparisonMode::Overlay, Some((comparison, oracle))) => {
                        paint_checkpoint(
                            &painter,
                            checkpoint,
                            response.rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Rust)),
                        );
                        paint_checkpoint(
                            &painter,
                            oracle,
                            response.rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Oracle)),
                        );
                    }
                    (ComparisonMode::SingleBackend, _)
                    | (ComparisonMode::Overlay | ComparisonMode::SideBySide, None) => {
                        paint_checkpoint(
                            &painter,
                            checkpoint,
                            response.rect,
                            self.camera(),
                            self.layers,
                            None,
                        );
                    }
                }
            });
    }

    fn handle_viewport_gestures(&mut self, ctx: &egui::Context, response: &egui::Response) {
        if !response.hovered() {
            return;
        }
        let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0
            && let Some(pointer) = response.hover_pos()
        {
            let old_scale = self.pixels_per_meter;
            let new_scale = (old_scale * 1.1_f32.powf(scroll / 40.0)).clamp(5.0, 400.0);
            let offset = pointer - response.rect.center();
            let world_x = self.center_x + offset.x / old_scale;
            let world_y = self.center_y - offset.y / old_scale;
            self.center_x = world_x - offset.x / new_scale;
            self.center_y = world_y + offset.y / new_scale;
            self.pixels_per_meter = new_scale;
        }
        let shift = ctx.input(|input| input.modifiers.shift);
        let panning = response.dragged_by(PointerButton::Middle)
            || (shift && response.dragged_by(PointerButton::Primary));
        if panning {
            let delta = ctx.input(|input| input.pointer.delta());
            self.center_x -= delta.x / self.pixels_per_meter;
            self.center_y += delta.y / self.pixels_per_meter;
        }
        if response.double_clicked() {
            self.center_x = 0.0;
            self.center_y = 0.0;
            self.pixels_per_meter = 42.0;
            self.maybe_selected_primitive = None;
        } else if response.clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            self.maybe_selected_primitive = self.hit_test(response.rect, pointer);
        }
    }

    fn hit_test(&self, rect: Rect, pointer: Pos2) -> Option<String> {
        let checkpoint = self.maybe_display_checkpoint()?;
        let viewport = protocol_viewport(rect, self.camera())?;
        let frame = project_checkpoint(checkpoint, viewport, self.layers).ok()?;
        hit_test_frame(
            &frame,
            ProtocolScreenPoint {
                x: pointer.x,
                y: pointer.y,
            },
            6.0,
        )
        .map(|key| format!("{key:?}"))
    }

    const fn camera(&self) -> (f32, f32, f32) {
        (self.center_x, self.center_y, self.pixels_per_meter)
    }

    fn render_settings(&mut self, ctx: &egui::Context) {
        if self.open_panel != OpenPanel::Settings {
            return;
        }
        let mut open = true;
        egui::Window::new("Run settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.colored_label(MUTED, "Validated values apply only through Apply & Restart");
                for field in SETTINGS_FIELDS {
                    ui.label(setting_label(field));
                    let index = setting_index(field);
                    let response = ui.text_edit_singleline(&mut self.settings_drafts[index]);
                    if response.lost_focus() || response.changed() {
                        self.settings
                            .edit(field, self.settings_drafts[index].clone());
                        self.settings.commit(field);
                    }
                    if let Some(error) = self.settings.maybe_error(field) {
                        ui.colored_label(ERROR, error);
                    }
                }
                if ui
                    .add_enabled(
                        self.settings.apply_enabled(),
                        egui::Button::new("Apply & Restart"),
                    )
                    .clicked()
                {
                    self.queue(PendingEffect::ApplySettings(self.settings.accepted()));
                    self.open_panel = OpenPanel::None;
                }
            });
        if !open {
            self.open_panel = OpenPanel::None;
        }
    }

    fn render_about(&mut self, ctx: &egui::Context) {
        if self.open_panel != OpenPanel::About {
            return;
        }
        let about = self.about_panel();
        let mut open = true;
        egui::Window::new("About & provenance")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading(about.project_name());
                ui.label(about.maintainer());
                ui.label(about.license_summary());
                ui.colored_label(MUTED, about.upstream_summary());
                ui.separator();
                for value in [
                    about.version_label(),
                    about.commit_label(),
                    about.profile(),
                    about.target(),
                    about.rust_toolchain(),
                    about.protocol_version(),
                    about.adapter_version(),
                    about.run_identity(),
                    about.oracle_identity(),
                    about.evidence_tier(),
                ] {
                    ui.label(value);
                }
                ui.separator();
                for link in about.links() {
                    ui.horizontal(|ui| {
                        if ui.link(link.label()).clicked() {
                            ctx.open_url(egui::OpenUrl::new_tab(link.url()));
                        }
                        if ui.small_button("Copy URL").clicked() {
                            ctx.copy_text(link.url().to_owned());
                        }
                        ui.colored_label(MUTED, link.url());
                    });
                }
            });
        if !open {
            self.open_panel = OpenPanel::None;
        }
    }

    fn about_panel(&self) -> AboutPanel {
        let target = format!("{}-{}", env::consts::ARCH, env::consts::OS);
        let maybe_run_identity = self
            .testbed
            .selected()
            .map(|selected| selected.identity().content_sha256().as_str());
        let maybe_oracle_identity = self
            .maybe_oracle
            .as_ref()
            .map(|checkpoint| checkpoint.resolved_sha256().as_str());
        build_about_panel(ProvenanceInput {
            version: Some(env!("CARGO_PKG_VERSION")),
            commit: option_env!("LIQUIDFUN_BUILD_COMMIT"),
            profile: option_env!("PROFILE"),
            target: Some(&target),
            rust_toolchain: Some("Rust 1.97.0"),
            protocol_version: Some("phase11-v1"),
            adapter_version: Some(env!("CARGO_PKG_VERSION")),
            run_identity: maybe_run_identity,
            oracle_revision: maybe_oracle_identity,
            oracle_compiler: None,
            oracle_preset: None,
            evidence_tier: self
                .maybe_oracle
                .as_ref()
                .map(|_| "diagnostic comparison; not compatibility authority"),
        })
    }

    fn render_shortcuts(&mut self, ctx: &egui::Context) {
        if self.open_panel != OpenPanel::ShortcutHelp {
            return;
        }
        let mut open = true;
        egui::Window::new("Keyboard shortcuts")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                for shortcut in [
                    "Space Run/Pause · Right Step · R Restart · C Capture",
                    "/ Search · 1–4 overlay groups · O Overlay/Side by side",
                    "F Focus difference · [ / ] Previous/Next difference",
                    "A Apply next typed scenario action",
                    "Home/double-click Reset camera · Wheel Zoom · Shift-drag Pan",
                ] {
                    ui.label(shortcut);
                }
                ui.colored_label(
                    MUTED,
                    "Presentation shortcuts never submit simulation commands.",
                );
                if let Some(shortcut) = self.scenario_shortcuts().first() {
                    ui.colored_label(
                        ACCENT,
                        format!(
                            "{} — {} ({})",
                            shortcut.key().to_ascii_uppercase(),
                            shortcut.label(),
                            shortcut.action_id().as_str()
                        ),
                    );
                }
            });
        if !open {
            self.open_panel = OpenPanel::None;
        }
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let editing = ctx.egui_wants_keyboard_input();
        let scenario_shortcuts = self.scenario_shortcuts();
        let maybe_key = ctx.input(|input| {
            [
                (egui::Key::Space, KeyboardKey::Space),
                (egui::Key::ArrowRight, KeyboardKey::Right),
                (egui::Key::R, KeyboardKey::R),
                (egui::Key::C, KeyboardKey::C),
                (egui::Key::Slash, KeyboardKey::Slash),
                (egui::Key::F, KeyboardKey::F),
                (egui::Key::OpenBracket, KeyboardKey::LeftBracket),
                (egui::Key::CloseBracket, KeyboardKey::RightBracket),
                (egui::Key::Num1, KeyboardKey::Digit1),
                (egui::Key::Num2, KeyboardKey::Digit2),
                (egui::Key::Num3, KeyboardKey::Digit3),
                (egui::Key::Num4, KeyboardKey::Digit4),
                (egui::Key::Home, KeyboardKey::Home),
                (egui::Key::Questionmark, KeyboardKey::QuestionMark),
                (egui::Key::Escape, KeyboardKey::Escape),
                (egui::Key::A, KeyboardKey::Scenario('a')),
            ]
            .into_iter()
            .find_map(|(egui_key, semantic_key)| {
                input.key_pressed(egui_key).then_some(semantic_key)
            })
        });
        let Some(key) = maybe_key else {
            return;
        };
        let checkpoint_id = self.testbed.reachable_checkpoint_id().cloned();
        let effect = resolve_key(
            key,
            InputContext {
                session_state: self.testbed.session_state(),
                editing_field: editing,
                maybe_checkpoint_id: checkpoint_id.as_ref(),
                scenario_shortcuts: &scenario_shortcuts,
            },
        );
        match effect {
            Some(InputEffect::Controller(action)) => {
                if matches!(action, ControllerAction::ApplyScenarioAction(_)) {
                    self.maybe_last_scenario_action_label = Some(PARTICLE_PAUSE_ACTION_LABEL);
                }
                self.queue(PendingEffect::Controller(action));
            }
            Some(InputEffect::Presentation(action)) => self.apply_presentation(action),
            None => {}
        }
    }

    fn apply_presentation(&mut self, action: PresentationAction) {
        match action {
            PresentationAction::FocusScenarioSearch => self.open_panel = OpenPanel::Scenario,
            PresentationAction::FocusDifference => {
                self.open_panel = OpenPanel::Inspector;
                self.focused_difference = 0;
            }
            PresentationAction::PreviousDifference => self.move_difference(-1),
            PresentationAction::NextDifference => self.move_difference(1),
            PresentationAction::ToggleOverlayGroup(group) => self.toggle_group(group),
            PresentationAction::ResetCamera => {
                self.center_x = 0.0;
                self.center_y = 0.0;
                self.pixels_per_meter = 42.0;
            }
            PresentationAction::OpenShortcutHelp => self.open_panel = OpenPanel::ShortcutHelp,
            PresentationAction::CloseTopmostOrClearFocus => self.open_panel = OpenPanel::None,
        }
    }

    fn move_difference(&mut self, direction: i8) {
        let count = self.diagnostics.maybe_comparison().map_or(0, |model| {
            DifferenceList::new(model, Camera::default(), BackendAvailability::Both)
                .entries()
                .len()
        });
        if count == 0 {
            self.focused_difference = 0;
        } else if direction < 0 {
            self.focused_difference = (self.focused_difference + count - 1) % count;
        } else {
            self.focused_difference = (self.focused_difference + 1) % count;
        }
    }

    fn toggle_group(&mut self, group: u8) {
        let layers: &[DebugLayerName] = match group {
            1 => &[DebugLayerName::Contacts, DebugLayerName::ContactNormals],
            2 => &[DebugLayerName::ParticleContacts],
            3 => &[DebugLayerName::BroadPhase],
            4 => &[DebugLayerName::Labels],
            _ => &[],
        };
        for layer in layers {
            let index = layer_index(*layer);
            self.layer_enabled[index] = !self.layer_enabled[index];
            self.layers.set(*layer, self.layer_enabled[index]);
        }
    }

    fn scenario_shortcuts(&self) -> Vec<ScenarioShortcut> {
        let next_ordinal = self.testbed.completed_logical_steps().saturating_add(1);
        self.testbed
            .selected()
            .and_then(|resolved| {
                resolved.actions().iter().find(|action| {
                    action.schedule()
                        == ActionSchedule::LogicalStep {
                            ordinal: next_ordinal,
                        }
                })
            })
            .and_then(|action| {
                let label = if matches!(
                    action.action(),
                    RigidWorldAction::Particle {
                        action: liquidfun_test_protocol::Phase9ParticleAction::SetPaused { .. }
                    }
                ) {
                    PARTICLE_PAUSE_ACTION_LABEL
                } else {
                    "Apply next typed scenario action"
                };
                ScenarioShortcut::new('a', action.action_id().clone(), label)
            })
            .into_iter()
            .collect()
    }

    fn handle_screenshot_result(&mut self, ctx: &egui::Context) {
        let maybe_image = ctx.input(|input| {
            input.events.iter().find_map(|event| {
                let egui::Event::Screenshot { image, .. } = event else {
                    return None;
                };
                Some(Arc::clone(image))
            })
        });
        let Some(image) = maybe_image else {
            return;
        };
        self.maybe_screenshot_status = Some(match save_screenshot(&image) {
            Ok(path) => format!(
                "Saved {} — diagnostic only, not compatibility evidence",
                path.display()
            ),
            Err(error) => error,
        });
    }
}
