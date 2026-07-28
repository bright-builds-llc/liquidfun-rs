impl DesktopApp {
    fn new(maybe_oracle_path: Option<PathBuf>) -> Result<Self, String> {
        let maybe_oracle = maybe_oracle_path.map(load_oracle_checkpoint).transpose()?;
        let mut testbed = InteractiveTestbed::new().map_err(bounded_error)?;
        let first_visual = testbed
            .visible_rows()
            .iter()
            .position(|row| row.eligibility().visual())
            .ok_or_else(|| "reviewed catalog has no visual scenario".to_owned())?;
        testbed
            .select_visible(first_visual)
            .map_err(bounded_error)?;
        testbed.step_once().map_err(bounded_error)?;
        testbed
            .capture_reachable_checkpoint()
            .map_err(bounded_error)?;
        let settings = SettingsEditor::new(
            testbed
                .selected_settings()
                .ok_or_else(|| "selected scenario has no settings".to_owned())?,
        );
        let settings_drafts = settings_drafts(&settings);
        Ok(Self {
            shell: AppShell::default(),
            testbed,
            query: String::new(),
            layers: ProtocolLayerVisibility::all(),
            layer_enabled: [true; 9],
            pixels_per_meter: 42.0,
            center_x: 0.0,
            center_y: 0.0,
            maybe_oracle,
            diagnostics: DesktopDiagnostics::default(),
            open_panel: OpenPanel::None,
            settings,
            settings_drafts,
            comparison_mode: ComparisonMode::Overlay,
            focused_difference: 0,
            maybe_selected_primitive: None,
            maybe_last_scenario_action_label: None,
            maybe_pending_effect: None,
            maybe_driver_tick: None,
            maybe_screenshot_status: None,
        })
    }

    fn queue(&mut self, effect: PendingEffect) {
        if self.maybe_pending_effect.is_none() {
            self.maybe_pending_effect = Some(effect);
        }
    }

    fn dispatch_pending(&mut self) {
        let Some(effect) = self.maybe_pending_effect.take() else {
            return;
        };
        let clears_comparison = matches!(
            effect,
            PendingEffect::Select(_)
                | PendingEffect::ApplySettings(_)
                | PendingEffect::Controller(ControllerAction::Restart)
        );
        let command = match effect {
            PendingEffect::Controller(action) => self.testbed.begin_action(action),
            PendingEffect::Select(index) => self.testbed.begin_select_visible(index),
            PendingEffect::ApplySettings(settings) => self.testbed.begin_settings(settings),
        };
        let result = command.and_then(|command: SessionCommand| {
            let AppEffect::Submit(command) = self.shell.submit(command);
            self.testbed.submit_command(command)
        });
        match result {
            Ok(()) => {
                if clears_comparison {
                    self.diagnostics.reset_comparison();
                }
                if let Some(settings) = self.testbed.selected_settings() {
                    self.settings = SettingsEditor::new(settings);
                    self.settings_drafts = settings_drafts(&self.settings);
                }
            }
            Err(error) => self.diagnostics.set_error(error),
        }
    }

    fn refresh_comparison(&mut self) {
        let Some(native) = self.maybe_display_checkpoint() else {
            return;
        };
        let native_identity = (
            native.resolved_sha256().clone(),
            native.checkpoint_id().clone(),
        );
        if self.diagnostics.maybe_compared_identity() == Some(&native_identity) {
            return;
        }
        let Some(oracle) = self.maybe_oracle.as_ref() else {
            self.diagnostics.reset_comparison();
            return;
        };
        let policy = Phase4PolicyProfile::parse_toml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../protocol/tolerances/phase4-v1.toml"
        )));
        let comparison = policy
            .map_err(|error| error.to_string())
            .and_then(|policy| {
                compare_canonical_checkpoints(
                    native,
                    oracle,
                    &policy,
                    ComparisonLimits::phase11_default(),
                )
                .map_err(|error| error.to_string())
            });
        self.diagnostics
            .apply_comparison(native_identity, comparison);
        if self.diagnostics.maybe_comparison().is_some() {
            self.focused_difference = 0;
        }
    }

    fn maybe_display_checkpoint(&self) -> Option<&CanonicalCheckpoint> {
        if self.maybe_oracle.is_some() {
            return self.testbed.latest_checkpoint();
        }
        self.testbed.presentation_checkpoint()
    }

    fn render_app_bar(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("app_bar")
            .exact_size(48.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.heading("liquidfun-rs");
                    let comparison = self
                        .diagnostics
                        .maybe_comparison()
                        .map(ComparisonModel::state);
                    ui.colored_label(
                        state_color(comparison),
                        format!(
                            "{} {}",
                            status_marker(self.testbed.session_state(), comparison),
                            status_copy(self.testbed.session_state(), comparison)
                        ),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Private diagnostic UI — pixels and timing are not compatibility authority",
                        )
                        .color(MUTED),
                    );
                    if ui.button("About & provenance").clicked() {
                        self.open_panel = OpenPanel::About;
                    }
                });
            });
    }

    fn render_scenarios(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("scenario_rail")
            .resizable(true)
            .default_size(280.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(root, |ui| {
                ui.heading("Scenarios");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search scenarios (/)")
                        .desired_width(f32::INFINITY),
                );
                if response.changed()
                    && let Err(error) = self.testbed.set_query(&self.query)
                {
                    self.diagnostics.set_error(error);
                }
                let current = self.testbed.current_selection().cloned();
                let mut maybe_selected = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, row) in self.testbed.visible_rows().iter().enumerate() {
                        let identity = format!(
                            "{}@{}  R:{} O:{} V:{}",
                            row.selection().catalog_slug(),
                            row.selection().scenario_version(),
                            yes_no(row.eligibility().rust()),
                            yes_no(row.eligibility().oracle()),
                            yes_no(row.eligibility().visual())
                        );
                        let selected = current
                            .as_ref()
                            .is_some_and(|selection| selection == row.selection());
                        if ui
                            .selectable_label(
                                selected,
                                format!("{}\n{identity}", row.display_title()),
                            )
                            .clicked()
                        {
                            maybe_selected = Some(index);
                        }
                    }
                });
                if let Some(index) = maybe_selected {
                    self.queue(PendingEffect::Select(index));
                }
            });
    }

    fn render_inspector(&mut self, root: &mut egui::Ui) {
        egui::Panel::right("inspector")
            .resizable(true)
            .default_size(360.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(root, |ui| {
                ui.heading("Inspect");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let (heading, body) = comparison_copy(self.diagnostics.maybe_comparison());
                    ui.label(egui::RichText::new(heading).strong());
                    ui.colored_label(MUTED, body);
                    if let Some(selected) = self.testbed.selected() {
                        let identity = selected.identity();
                        ui.label(format!(
                            "Scenario: {}@{}",
                            identity.slug().as_str(),
                            identity.scenario_version().get()
                        ));
                        ui.colored_label(
                            MUTED,
                            format!(
                                "Resolved: {}",
                                shorten(identity.content_sha256().as_str(), 18)
                            ),
                        );
                    }
                    let state = if self.testbed.session_state() == SessionState::ReadyPaused {
                        SESSION_PAUSED_LABEL.to_owned()
                    } else {
                        format!("{:?}", self.testbed.session_state())
                    };
                    ui.label(format!("State: {state}"));
                    ui.label(format!(
                        "Logical steps: {}",
                        self.testbed.completed_logical_steps()
                    ));
                    if let Some(label) = self.maybe_last_scenario_action_label {
                        ui.colored_label(ACCENT, label);
                    }
                    if let Some(key) = self.maybe_selected_primitive.as_deref() {
                        ui.colored_label(
                            ACCENT,
                            format!("Selected primitive: {}", shorten(key, 42)),
                        );
                    }
                    self.render_checkpoint_diagnostics(ui);
                    self.render_difference(ui);
                    if let Some(error) = self.diagnostics.maybe_comparison_error() {
                        ui.colored_label(ERROR, format!("Comparison error: {error}"));
                    }
                    if let Some(error) = self.diagnostics.maybe_error() {
                        ui.colored_label(ERROR, format!("Last bounded error: {error}"));
                    }
                    if let Some(status) = self.maybe_screenshot_status.as_deref() {
                        ui.colored_label(ACCENT, status);
                    }
                });
            });
    }

    fn render_checkpoint_diagnostics(&self, ui: &mut egui::Ui) {
        let Some(displayed) = self.maybe_display_checkpoint() else {
            ui.colored_label(MUTED, "Captured checkpoints: 0");
            return;
        };
        let diagnostics = CheckpointDiagnostics::from_checkpoint(displayed);
        ui.label(format!(
            "Captured checkpoints: {}",
            self.testbed.captured_checkpoint_count()
        ));
        let showing_history = self
            .testbed
            .latest_checkpoint()
            .is_some_and(|latest| latest.checkpoint_id() != displayed.checkpoint_id());
        ui.label(format!(
            "Displayed: {}{}",
            displayed.checkpoint_id().as_str(),
            if showing_history {
                " (last drawable)"
            } else {
                ""
            }
        ));
        if showing_history {
            ui.colored_label(MUTED, "Latest capture is empty after teardown");
        }
        let boundary = match displayed.position() {
            CheckpointPosition::Action { ordinal, .. } => format!("action {ordinal}"),
            CheckpointPosition::LogicalStep { ordinal } => format!("logical step {ordinal}"),
        };
        ui.colored_label(
            MUTED,
            format!(
                "Boundary: {boundary} | sim {:.5}s",
                displayed.simulation_time_bits().to_f32()
            ),
        );
        ui.label(format!(
            "World B:{} F:{} J:{} C:{} P:{}",
            count_text(diagnostics.maybe_body_count()),
            count_text(diagnostics.maybe_fixture_count()),
            count_text(diagnostics.maybe_joint_count()),
            count_text(diagnostics.maybe_contact_count()),
            count_text(diagnostics.maybe_particle_count())
        ));
        ui.colored_label(
            MUTED,
            format!(
                "Draw shapes:{} joints:{} particles:{}",
                diagnostics.layer_count(DebugLayerName::Shapes),
                diagnostics.layer_count(DebugLayerName::Joints),
                diagnostics.layer_count(DebugLayerName::Particles)
            ),
        );
    }

    fn render_difference(&mut self, ui: &mut egui::Ui) {
        let Some(comparison) = self.diagnostics.maybe_comparison() else {
            return;
        };
        let differences =
            DifferenceList::new(comparison, Camera::default(), BackendAvailability::Both);
        let entries = differences.entries();
        ui.separator();
        ui.label(format!("Comparison: {:?}", comparison.state()));
        if entries.is_empty() {
            ui.colored_label(MUTED, "No differences at this checkpoint");
            return;
        }
        self.focused_difference = self.focused_difference.min(entries.len() - 1);
        ui.horizontal(|ui| {
            if ui.button("Previous").clicked() {
                self.focused_difference =
                    (self.focused_difference + entries.len() - 1) % entries.len();
            }
            if ui.button("Next").clicked() {
                self.focused_difference = (self.focused_difference + 1) % entries.len();
            }
        });
        let entry = entries[self.focused_difference];
        ui.label(format!(
            "Difference {} of {}",
            self.focused_difference + 1,
            entries.len()
        ));
        ui.label(entry.semantic_path());
        ui.colored_label(
            MUTED,
            format!("Rust: {}", entry.maybe_rust_value().unwrap_or("absent")),
        );
        ui.colored_label(
            MUTED,
            format!("Oracle: {}", entry.maybe_oracle_value().unwrap_or("absent")),
        );
    }

    fn render_controls(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        egui::Panel::bottom("controls")
            .frame(egui::Frame::new().fill(PANEL_ALT))
            .show(root, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Scenarios").clicked() {
                        self.open_panel = OpenPanel::Scenario;
                    }
                    if ui.button("Inspect").clicked() {
                        self.open_panel = OpenPanel::Inspector;
                    }
                    let projection = ControllerProjection::from_state(self.testbed.session_state());
                    if self.testbed.session_state() == SessionState::Running {
                        if ui
                            .add_enabled(
                                projection.enabled(ControlCapability::Pause),
                                egui::Button::new("Pause"),
                            )
                            .clicked()
                        {
                            self.queue(PendingEffect::Controller(ControllerAction::Pause));
                        }
                    } else if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::Run),
                            egui::Button::new("Run"),
                        )
                        .clicked()
                    {
                        self.queue(PendingEffect::Controller(ControllerAction::Run));
                    }
                    if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::StepOnce),
                            egui::Button::new("Step"),
                        )
                        .clicked()
                    {
                        self.queue(PendingEffect::Controller(ControllerAction::StepOnce));
                    }
                    if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::Restart),
                            egui::Button::new("Restart"),
                        )
                        .clicked()
                    {
                        self.queue(PendingEffect::Controller(ControllerAction::Restart));
                    }
                    let maybe_checkpoint = self.testbed.reachable_checkpoint_id().cloned();
                    if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::Capture)
                                && maybe_checkpoint.is_some(),
                            egui::Button::new("Capture"),
                        )
                        .clicked()
                        && let Some(checkpoint_id) = maybe_checkpoint
                    {
                        self.queue(PendingEffect::Controller(
                            ControllerAction::CaptureCheckpoint(checkpoint_id),
                        ));
                    }
                    if ui.button("Settings").clicked() {
                        self.open_panel = OpenPanel::Settings;
                    }
                    if ui
                        .add_enabled(
                            self.diagnostics.maybe_comparison().is_some(),
                            egui::Button::new(match self.comparison_mode {
                                ComparisonMode::Overlay => "Overlay",
                                ComparisonMode::SideBySide => "Side by side",
                                ComparisonMode::SingleBackend => "Rust view",
                            }),
                        )
                        .clicked()
                    {
                        self.comparison_mode = match self.comparison_mode {
                            ComparisonMode::Overlay => ComparisonMode::SideBySide,
                            ComparisonMode::SideBySide | ComparisonMode::SingleBackend => {
                                ComparisonMode::Overlay
                            }
                        };
                    }
                    if ui.button("Screenshot").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default(),
                        ));
                    }
                    if ui.button("Shortcuts").clicked() {
                        self.open_panel = OpenPanel::ShortcutHelp;
                    }
                });
            });
    }

}
