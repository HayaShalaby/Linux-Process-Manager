use crate::process::Process;
use crate::process::tree::ProcessNode;
use crate::manager::Manager;
use crate::manager::operations;
use crate::user::{User, Privilege};
use egui::{Color32, RichText, ScrollArea, TextEdit};
use std::collections::{HashSet, HashMap};
use std::time::{Duration, Instant};

/// Resource thresholds for monitoring abnormal processes
#[derive(Clone)]
struct ResourceThresholds {
    cpu_percent: f32,
    memory_mb: u64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            cpu_percent: 80.0,
            memory_mb: 1000,
        }
    }
}

/// Main application state for the Process Manager GUI
pub struct ProcessManagerApp {
    manager: Manager,
    processes_vec: Vec<Process>, // Cached vector for display
    filtered_processes: Vec<usize>, // Indices into processes_vec
    search_filter: String,
    sort_column: SortColumn,
    sort_ascending: bool,
    last_refresh: Instant,
    refresh_interval: Duration,
    selected_pid: Option<u32>,
    selected_pids: HashSet<u32>, // For batch operations
    error_message: Option<String>,
    success_message: Option<String>,
    success_message_time: Option<Instant>, // Track when success message was set
    auto_refresh: bool,
    show_tree_view: bool,
    show_threshold_config: bool,
    thresholds: ResourceThresholds,
    priority_input: String,
}

#[derive(Clone, Copy, PartialEq)]
enum SortColumn {
    Pid,
    Name,
    Uid,
    State,
    Cpu,
    Memory,
    Priority,
}

impl Default for ProcessManagerApp {
    fn default() -> Self {
        // Create a default admin user for GUI
        let admin_user = User::new(0, "admin", Privilege::Admin);
        let manager = Manager::new(admin_user.clone()).unwrap_or_else(|e| {
            eprintln!("Failed to initialize manager: {}", e);
            // Create a minimal manager if initialization fails
            Manager {
                processes: HashMap::new(),
                active_user: admin_user,
                root_pid: 1,
            }
        });
        
        Self {
            manager,
            processes_vec: Vec::new(),
            filtered_processes: Vec::new(),
            search_filter: String::new(),
            sort_column: SortColumn::Pid,
            sort_ascending: true,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            selected_pid: None,
            selected_pids: HashSet::new(),
            error_message: None,
            success_message: None,
            success_message_time: None,
            auto_refresh: true,
            show_tree_view: false,
            show_threshold_config: false,
            thresholds: ResourceThresholds::default(),
            priority_input: String::new(),
        }
    }
}

impl ProcessManagerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.refresh_processes();
        app
    }

    /// Refresh the process list from /proc filesystem using Manager
    fn refresh_processes(&mut self) {
        self.error_message = None;
        // Note: Don't clear success_message here - let it persist so user can see it
        
        // Use Manager's refresh method
        match self.manager.refresh() {
            Ok(_) => {
                // Update cached vector from manager
                self.processes_vec = self.manager.processes().into_iter().cloned().collect();
                self.apply_filters_and_sort();
                self.last_refresh = Instant::now();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to refresh processes: {}", e));
            }
        }
    }

    /// Apply search filter and sorting
    fn apply_filters_and_sort(&mut self) {
        // Filter processes
        self.filtered_processes = self
            .processes_vec
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if self.search_filter.is_empty() {
                    return true;
                }
                let filter_lower = self.search_filter.to_lowercase();
                p.name.to_lowercase().contains(&filter_lower)
                    || p.process_id.to_string().contains(&filter_lower)
                    || p.user_id.to_string().contains(&filter_lower)
            })
            .map(|(idx, _)| idx)
            .collect();

        // Sort filtered indices
        self.filtered_processes.sort_by(|&a, &b| {
            let cmp = match self.sort_column {
                SortColumn::Pid => self.processes_vec[a].process_id.cmp(&self.processes_vec[b].process_id),
                SortColumn::Name => self.processes_vec[a].name.cmp(&self.processes_vec[b].name),
                SortColumn::Uid => self.processes_vec[a].user_id.cmp(&self.processes_vec[b].user_id),
                SortColumn::State => self.processes_vec[a]
                    .pcb_data
                    .state
                    .cmp(&self.processes_vec[b].pcb_data.state),
                SortColumn::Cpu => self.processes_vec[a]
                    .pcb_data
                    .cpu_percent
                    .partial_cmp(&self.processes_vec[b].pcb_data.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => self.processes_vec[a]
                    .pcb_data
                    .memory_rss_mb
                    .cmp(&self.processes_vec[b].pcb_data.memory_rss_mb),
                SortColumn::Priority => self.processes_vec[a]
                    .pcb_data
                    .priority
                    .cmp(&self.processes_vec[b].pcb_data.priority),
            };

            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }

    /// Get selected process details
    fn get_selected_process(&self) -> Option<&Process> {
        self.selected_pid
            .and_then(|pid| self.processes_vec.iter().find(|p| p.process_id == pid))
    }

    /// Toggle selection of a process for batch operations
    fn toggle_selection(&mut self, pid: u32) {
        if self.selected_pids.contains(&pid) {
            self.selected_pids.remove(&pid);
        } else {
            self.selected_pids.insert(pid);
        }
    }

    /// Clear all selections
    fn clear_selections(&mut self) {
        self.selected_pids.clear();
    }

    /// Check if process is abnormal (zombie or exceeds thresholds)
    fn is_abnormal(&self, process: &Process) -> bool {
        process.pcb_data.state == 'Z' // Zombie
            || process.pcb_data.cpu_percent > self.thresholds.cpu_percent
            || process.pcb_data.memory_rss_mb > self.thresholds.memory_mb
    }

    /// Get abnormality reason for display
    fn get_abnormality_reason(&self, process: &Process) -> Option<String> {
        let mut reasons = Vec::new();
        if process.pcb_data.state == 'Z' {
            reasons.push("Zombie process".to_string());
        }
        if process.pcb_data.cpu_percent > self.thresholds.cpu_percent {
            reasons.push(format!(
                "CPU usage {:.1}% exceeds threshold {:.1}%",
                process.pcb_data.cpu_percent, self.thresholds.cpu_percent
            ));
        }
        if process.pcb_data.memory_rss_mb > self.thresholds.memory_mb {
            reasons.push(format!(
                "Memory usage {} MB exceeds threshold {} MB",
                process.pcb_data.memory_rss_mb, self.thresholds.memory_mb
            ));
        }
        if reasons.is_empty() {
            None
        } else {
            Some(reasons.join(", "))
        }
    }

    /// Build process tree structure using Manager
    fn build_process_tree(&self) -> Option<ProcessNode> {
        // Use Manager's build_process_tree method
        self.manager.build_process_tree()
    }

    /// Render process tree node recursively with beautiful tree visualization
    fn render_tree_node(&mut self, ui: &mut egui::Ui, node: &ProcessNode, depth: usize, is_last: bool, prefix: String) {
        let process = &node.process;
        let is_abnormal = self.is_abnormal(process);
        let is_selected = self.selected_pids.contains(&process.process_id);
        let has_children = !node.children.is_empty();

        // Build tree connector
        let connector = if depth == 0 {
            "🌳 ".to_string() // Root process
        } else if is_last {
            format!("{}└─ ", prefix)
        } else {
            format!("{}├─ ", prefix)
        };

        // Build continuation prefix for children
        let child_prefix = if depth == 0 {
            String::new()
        } else if is_last {
            format!("{}   ", prefix) // Empty space for last child's children
        } else {
            format!("{}│  ", prefix) // Vertical line for non-last children
        };

        ui.horizontal(|ui| {
            // Tree connector with styling
            ui.label(
                RichText::new(&connector)
                    .color(if depth == 0 { Color32::from_rgb(139, 69, 19) } else { Color32::GRAY })
                    .monospace()
            );

            // Checkbox for batch selection
            let mut checked = is_selected;
            if ui.checkbox(&mut checked, "").changed() {
                self.toggle_selection(process.process_id);
            }

            // Process info with better formatting
            let name_color = if is_abnormal {
                Color32::YELLOW
            } else if depth == 0 {
                Color32::from_rgb(100, 200, 100) // Light green for root
            } else {
                Color32::WHITE
            };

            // State color
            let state_color = match process.pcb_data.state {
                'R' => Color32::GREEN,
                'S' => Color32::BLUE,
                'D' => Color32::RED,
                'Z' => Color32::YELLOW,
                'T' => Color32::GRAY,
                _ => Color32::WHITE,
            };

            // Build process display text
            let pid_text = RichText::new(format!("PID:{}", process.process_id))
                .strong()
                .color(Color32::from_rgb(100, 150, 255));
            
            let name_text = RichText::new(&process.name)
                .color(name_color)
                .strong();
            
            let state_text = RichText::new(format!("[{}]", process.pcb_data.state))
                .color(state_color)
                .monospace();
            
            let mem_text = RichText::new(format!("{:.1}MB", process.pcb_data.memory_rss_mb))
                .color(Color32::from_rgb(255, 200, 100));

            // Display process info
            ui.horizontal(|ui| {
                ui.label(pid_text);
                ui.label(" • ");
                ui.label(name_text);
                ui.label(" • ");
                ui.label(state_text);
                ui.label(" • ");
                ui.label(mem_text);
                
                if has_children {
                    ui.label(
                        RichText::new(format!(" ({} children)", node.children.len()))
                            .color(Color32::from_rgb(150, 150, 150))
                            .small()
                    );
                }
            });

            // Make the whole row clickable
            if ui.interact(ui.available_rect(), egui::Id::new(process.process_id), egui::Sense::click()).clicked() {
                self.selected_pid = Some(process.process_id);
            }
        });

        // Render children with proper tree structure
        let child_count = node.children.len();
        for (idx, child) in node.children.iter().enumerate() {
            let is_last_child = idx == child_count - 1;
            self.render_tree_node(ui, child, depth + 1, is_last_child, child_prefix.clone());
        }
    }

    // Real backend function calls using Ismail's implementation
    fn kill_process(&mut self, pid: u32) -> Result<(), String> {
        operations::kill_process(&self.manager, pid)
    }

    fn terminate_process(&mut self, pid: u32) -> Result<(), String> {
        operations::terminate_process(&self.manager, pid)
    }

    fn pause_process(&mut self, pid: u32) -> Result<(), String> {
        operations::pause_process(&self.manager, pid)
    }

    fn resume_process(&mut self, pid: u32) -> Result<(), String> {
        operations::resume_process(&self.manager, pid)
    }

    fn set_priority(&mut self, pid: u32, nice: i32) -> Result<(), String> {
        operations::set_priority(&self.manager, pid, nice)
    }

    fn batch_kill(&mut self, pids: Vec<u32>, force: bool) {
        let mut successful = 0;
        let mut failed = 0;
        
        for pid in &pids {
            let result = if force {
                operations::kill_process(&self.manager, *pid)
            } else {
                operations::terminate_process(&self.manager, *pid)
            };
            
            match result {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    eprintln!("Failed to {} process {}: {}", if force { "kill" } else { "terminate" }, pid, e);
                }
            }
        }
        
        if failed == 0 {
            self.success_message = Some(format!(
                "Successfully {} {} process(es)",
                if force { "killed" } else { "terminated" },
                successful
            ));
            self.success_message_time = Some(Instant::now());
        } else {
            self.error_message = Some(format!(
                "{} {} process(es), {} failed",
                if force { "Killed" } else { "Terminated" },
                successful,
                failed
            ));
        }
        self.clear_selections();
    }

    fn batch_pause(&mut self, pids: Vec<u32>) {
        let mut successful = 0;
        let mut failed = 0;
        
        for pid in &pids {
            match operations::pause_process(&self.manager, *pid) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    eprintln!("Failed to pause process {}: {}", pid, e);
                }
            }
        }
        
        if failed == 0 {
            self.success_message = Some(format!("Successfully paused {} process(es)", successful));
            self.success_message_time = Some(Instant::now());
        } else {
            self.error_message = Some(format!("Paused {} process(es), {} failed", successful, failed));
        }
        self.clear_selections();
    }

    fn batch_resume(&mut self, pids: Vec<u32>) {
        let mut successful = 0;
        let mut failed = 0;
        
        for pid in &pids {
            match operations::resume_process(&self.manager, *pid) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    eprintln!("Failed to resume process {}: {}", pid, e);
                }
            }
        }
        
        if failed == 0 {
            self.success_message = Some(format!("Successfully resumed {} process(es)", successful));
            self.success_message_time = Some(Instant::now());
        } else {
            self.error_message = Some(format!("Resumed {} process(es), {} failed", successful, failed));
        }
        self.clear_selections();
    }
}

impl eframe::App for ProcessManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-refresh logic
        if self.auto_refresh && self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh_processes();
        }
        
        // Clear success message after 3 seconds
        if let Some(msg_time) = self.success_message_time {
            if msg_time.elapsed().as_secs() >= 3 {
                self.success_message = None;
                self.success_message_time = None;
            }
        }

        // Request repaint for auto-refresh
        if self.auto_refresh {
            ctx.request_repaint_after(self.refresh_interval);
        }

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Refresh").clicked() {
                        self.refresh_processes();
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.auto_refresh, "Auto Refresh");
                    ui.checkbox(&mut self.show_tree_view, "Process Tree View");
                    ui.checkbox(&mut self.show_threshold_config, "Configure Thresholds");
                    ui.separator();
                    if ui.button("Sort by PID").clicked() {
                        self.sort_column = SortColumn::Pid;
                        self.apply_filters_and_sort();
                    }
                    if ui.button("Sort by CPU").clicked() {
                        self.sort_column = SortColumn::Cpu;
                        self.apply_filters_and_sort();
                    }
                    if ui.button("Sort by Name").clicked() {
                        self.sort_column = SortColumn::Name;
                        self.apply_filters_and_sort();
                    }
                    if ui.button("Sort by Memory").clicked() {
                        self.sort_column = SortColumn::Memory;
                        self.apply_filters_and_sort();
                    }
                });

                ui.menu_button("Operations", |ui| {
                    if ui.button("Kill Selected").clicked() {
                        if !self.selected_pids.is_empty() {
                            let pids: Vec<u32> = self.selected_pids.iter().copied().collect();
                            self.batch_kill(pids, false);
                            self.refresh_processes();
                        }
                    }
                    if ui.button("Force Kill Selected").clicked() {
                        if !self.selected_pids.is_empty() {
                            let pids: Vec<u32> = self.selected_pids.iter().copied().collect();
                            self.batch_kill(pids, true);
                            self.refresh_processes();
                        }
                    }
                    if ui.button("Pause Selected").clicked() {
                        if !self.selected_pids.is_empty() {
                            let pids: Vec<u32> = self.selected_pids.iter().copied().collect();
                            self.batch_pause(pids);
                            self.refresh_processes();
                        }
                    }
                    if ui.button("Resume Selected").clicked() {
                        if !self.selected_pids.is_empty() {
                            let pids: Vec<u32> = self.selected_pids.iter().copied().collect();
                            self.batch_resume(pids);
                            self.refresh_processes();
                        }
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "Processes: {} | Selected: {} | Last refresh: {:.1}s ago",
                        self.filtered_processes.len(),
                        self.selected_pids.len(),
                        self.last_refresh.elapsed().as_secs_f32()
                    ));
                });
            });
        });

        // Threshold configuration window
        if self.show_threshold_config {
            egui::Window::new("Resource Thresholds")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Configure resource thresholds for monitoring:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("CPU Threshold (%):");
                        ui.add(
                            egui::Slider::new(&mut self.thresholds.cpu_percent, 0.0..=100.0)
                                .text("CPU"),
                        );
                        ui.label(format!("{:.1}%", self.thresholds.cpu_percent));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Memory Threshold (MB):");
                        ui.add(
                            egui::Slider::new(&mut self.thresholds.memory_mb, 0..=10000)
                                .text("Memory"),
                        );
                        ui.label(format!("{} MB", self.thresholds.memory_mb));
                    });

                    if ui.button("Close").clicked() {
                        self.show_threshold_config = false;
                    }
                });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.colored_label(Color32::RED, format!("Error: {}", error));
            }
            if let Some(success) = &self.success_message {
                ui.colored_label(Color32::GREEN, format!("Success: {}", success));
            }

            ui.vertical(|ui| {
                // Search bar and controls
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let response = ui.text_edit_singleline(&mut self.search_filter);
                    if response.changed() {
                        self.apply_filters_and_sort();
                    }

                    if ui.button("Clear Selection").clicked() {
                        self.clear_selections();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄 Refresh").clicked() {
                            self.refresh_processes();
                        }
                    });
                });

                ui.separator();

                // Process tree view or table view
                if self.show_tree_view {
                    // Tree view with beautiful visualization
                    ui.label(
                        RichText::new("🌲 Process Tree View")
                            .strong()
                            .color(Color32::from_rgb(100, 200, 100))
                            .size(16.0)
                    );
                    ui.separator();
                    ScrollArea::vertical().show(ui, |ui| {
                        if let Some(root) = self.build_process_tree() {
                            self.render_tree_node(ui, &root, 0, true, String::new());
                        } else {
                            ui.label("Failed to build process tree");
                        }
                    });
                } else {
                    // Table view
                    ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("process_table")
                            .num_columns(8)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                // Header row
                                // Select column
                                ui.label(RichText::new("Select").strong());
                                
                                // PID column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::Pid,
                                        RichText::new("PID")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::Pid {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::Pid {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::Pid;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                // Name column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::Name,
                                        RichText::new("Name")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::Name {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::Name {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::Name;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                // UID column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::Uid,
                                        RichText::new("UID")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::Uid {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::Uid {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::Uid;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                // State column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::State,
                                        RichText::new("State")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::State {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::State {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::State;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                // CPU column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::Cpu,
                                        RichText::new("CPU %")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::Cpu {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::Cpu {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::Cpu;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                // Memory column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::Memory,
                                        RichText::new("Memory (MB)")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::Memory {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::Memory {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::Memory;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                // Priority column
                                if ui
                                    .selectable_label(
                                        self.sort_column == SortColumn::Priority,
                                        RichText::new("Priority")
                                            .strong()
                                            .color(if self.sort_column == SortColumn::Priority {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .clicked()
                                {
                                    if self.sort_column == SortColumn::Priority {
                                        self.sort_ascending = !self.sort_ascending;
                                    } else {
                                        self.sort_column = SortColumn::Priority;
                                        self.sort_ascending = true;
                                    }
                                    self.apply_filters_and_sort();
                                }

                                ui.end_row();

                                // Data rows
                                // Collect selection changes to avoid borrowing conflicts
                                let mut selection_changes: Vec<u32> = Vec::new();
                                
                                for &idx in &self.filtered_processes {
                                    let process = &self.processes_vec[idx];
                                    let is_selected = self.selected_pids.contains(&process.process_id);
                                    let is_abnormal = self.is_abnormal(process);

                                    // Selection checkbox
                                    let mut checked = is_selected;
                                    if ui.checkbox(&mut checked, "").changed() {
                                        selection_changes.push(process.process_id);
                                    }

                                    // PID column
                                    let pid_response = ui.selectable_label(
                                        self.selected_pid == Some(process.process_id),
                                        process.process_id.to_string(),
                                    );
                                    if pid_response.clicked() {
                                        self.selected_pid = Some(process.process_id);
                                    }

                                    // Name column (highlight if abnormal)
                                    let name_color = if is_abnormal {
                                        Color32::YELLOW
                                    } else {
                                        Color32::WHITE
                                    };
                                    let name_response = ui.selectable_label(
                                        self.selected_pid == Some(process.process_id),
                                        RichText::new(process.name.as_str()).color(name_color),
                                    );
                                    if name_response.clicked() {
                                        self.selected_pid = Some(process.process_id);
                                    }

                                    // UID column
                                    ui.label(process.user_id.to_string());

                                    // State column (color-coded)
                                    let state_color = match process.pcb_data.state {
                                        'R' => Color32::GREEN,  // Running
                                        'S' => Color32::BLUE,   // Sleeping
                                        'D' => Color32::RED,    // Disk sleep
                                        'Z' => Color32::YELLOW, // Zombie
                                        'T' => Color32::GRAY,   // Stopped
                                        _ => Color32::WHITE,
                                    };
                                    ui.colored_label(state_color, process.pcb_data.state.to_string());

                                    // CPU column (highlight if exceeds threshold)
                                    let cpu_color = if process.pcb_data.cpu_percent > self.thresholds.cpu_percent {
                                        Color32::RED
                                    } else {
                                        Color32::WHITE
                                    };
                                    ui.colored_label(cpu_color, format!("{:.1}", process.pcb_data.cpu_percent));

                                    // Memory column (highlight if exceeds threshold)
                                    let mem_color = if process.pcb_data.memory_rss_mb > self.thresholds.memory_mb {
                                        Color32::RED
                                    } else {
                                        Color32::WHITE
                                    };
                                    ui.colored_label(mem_color, format!("{:.1}", process.pcb_data.memory_rss_mb));

                                    // Priority column
                                    ui.label(process.pcb_data.priority.to_string());

                                    ui.end_row();
                                }
                                
                                // Apply selection changes after the loop
                                for pid in selection_changes {
                                    self.toggle_selection(pid);
                                }
                            });
                    });
                }

                ui.separator();
            });
        });

        // Bottom panel for process details - always visible
        egui::TopBottomPanel::bottom("details_panel")
            .resizable(true)
            .min_height(200.0)
            .default_height(300.0)
            .show(ctx, |ui| {
                // Process details and actions panel
                // Copy the selected PID and process data to avoid borrowing conflicts
                let selected_pid = self.selected_pid;
                let process_data = selected_pid.and_then(|pid| {
                    self.processes_vec.iter().find(|p| p.process_id == pid).map(|p| {
                        (
                            p.process_id,
                            p.name.clone(),
                            p.user_id,
                            p.parent_id,
                            p.pcb_data.state,
                            p.pcb_data.memory_rss_mb,
                            p.pcb_data.priority,
                            p.pcb_data.cpu_percent,
                            self.get_abnormality_reason(p),
                        )
                    })
                });
                
                if let Some((process_pid, process_name, user_id, parent_id, state, memory, priority, cpu, abnormality_reason)) = process_data {
                    egui::CollapsingHeader::new("Process Details & Actions")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Details
                                egui::Grid::new("process_details")
                                    .num_columns(2)
                                    .spacing([20.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Process ID:");
                                        ui.label(process_pid.to_string());
                                        ui.end_row();

                                        ui.label("Name:");
                                        ui.label(&process_name);
                                        ui.end_row();

                                        ui.label("User ID:");
                                        ui.label(user_id.to_string());
                                        ui.end_row();

                                        ui.label("Parent PID:");
                                        ui.label(
                                            parent_id
                                                .map(|p| p.to_string())
                                                .unwrap_or_else(|| "N/A".to_string()),
                                        );
                                        ui.end_row();

                                        ui.label("State:");
                                        let state_color = match state {
                                            'R' => Color32::GREEN,
                                            'S' => Color32::BLUE,
                                            'D' => Color32::RED,
                                            'Z' => Color32::YELLOW,
                                            'T' => Color32::GRAY,
                                            _ => Color32::WHITE,
                                        };
                                        ui.colored_label(state_color, state.to_string());
                                        ui.end_row();

                                        ui.label("Memory (RSS):");
                                        ui.label(format!("{:.2} MB", memory));
                                        ui.end_row();

                                        ui.label("Priority (Nice):");
                                        ui.label(priority.to_string());
                                        ui.end_row();

                                        ui.label("CPU %:");
                                        ui.label(format!("{:.2}%", cpu));
                                        ui.end_row();

                                        // Show abnormality reason if any
                                        if let Some(reason) = abnormality_reason {
                                            ui.label("⚠️ Warning:");
                                            ui.colored_label(Color32::YELLOW, reason);
                                            ui.end_row();
                                        }
                                    });

                                // Actions
                                ui.vertical(|ui| {
                                    ui.label("Actions:");
                                    ui.separator();

                                    if ui.button("Kill").clicked() {
                                        match self.kill_process(process_pid) {
                                            Ok(_) => {
                                                self.success_message = Some(format!("Killed process {}", process_pid));
                                                self.success_message_time = Some(Instant::now());
                                                self.refresh_processes();
                                            }
                                            Err(e) => self.error_message = Some(e),
                                        }
                                    }

                                    if ui.button("Force Kill").clicked() {
                                        match self.kill_process(process_pid) {
                                            Ok(_) => {
                                                self.success_message = Some(format!("Force killed process {}", process_pid));
                                                self.success_message_time = Some(Instant::now());
                                                self.refresh_processes();
                                            }
                                            Err(e) => self.error_message = Some(e),
                                        }
                                    }
                                    
                                    if ui.button("Terminate").clicked() {
                                        match self.terminate_process(process_pid) {
                                            Ok(_) => {
                                                self.success_message = Some(format!("Terminated process {}", process_pid));
                                                self.success_message_time = Some(Instant::now());
                                                self.refresh_processes();
                                            }
                                            Err(e) => self.error_message = Some(e),
                                        }
                                    }

                                    if ui.button("Pause").clicked() {
                                        match self.pause_process(process_pid) {
                                            Ok(_) => {
                                                self.success_message = Some(format!("Paused process {}", process_pid));
                                                self.success_message_time = Some(Instant::now());
                                                self.refresh_processes();
                                            }
                                            Err(e) => self.error_message = Some(e),
                                        }
                                    }

                                    if ui.button("Resume").clicked() {
                                        match self.resume_process(process_pid) {
                                            Ok(_) => {
                                                self.success_message = Some(format!("Resumed process {}", process_pid));
                                                self.success_message_time = Some(Instant::now());
                                                self.refresh_processes();
                                            }
                                            Err(e) => self.error_message = Some(e),
                                        }
                                    }

                                    ui.separator();
                                    ui.label("Set Priority (Nice):");
                                    ui.horizontal(|ui| {
                                        ui.add(TextEdit::singleline(&mut self.priority_input)
                                            .desired_width(60.0)
                                            .hint_text("-20 to 19"));
                                        if ui.button("Apply").clicked() {
                                            if let Ok(nice) = self.priority_input.parse::<i32>() {
                                                match self.set_priority(process_pid, nice) {
                                                    Ok(_) => {
                                                        self.success_message = Some(format!("Set priority {} for process {}", nice, process_pid));
                                                        self.success_message_time = Some(Instant::now());
                                                        self.priority_input.clear();
                                                        self.refresh_processes();
                                                    }
                                                    Err(e) => self.error_message = Some(e),
                                                }
                                            } else {
                                                self.error_message = Some("Invalid priority value".to_string());
                                            }
                                        }
                                    });
                                });
                            });
                        });
                } else {
                    ui.label("Select a process to view details and perform actions");
                }
            });
    }
}
