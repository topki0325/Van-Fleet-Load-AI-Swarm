use eframe::egui;

#[derive(Debug, Default)]
pub struct ResourcesComponent;

impl ResourcesComponent {
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::app::VgaGuiApp) {
        ui.heading(app.tr("资源管理", "Resources"));
        ui.separator();

        egui::CollapsingHeader::new(app.tr("节点", "Nodes"))
            .id_source("resources_sub_nodes")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let label_allow_remote = app.tr("允许远程", "Allow remote");
                    if ui.button(app.tr("发现节点", "Discover Nodes")).clicked() {
                        app.discover_nodes();
                    }
                    if ui.button(app.tr("列出节点", "List Nodes")).clicked() {
                        app.list_discovered_nodes();
                    }
                    if ui.button(app.tr("读取远程开关", "Get Remote Status")).clicked() {
                        app.get_remote_access_status();
                    }
                    if ui.button(app.tr("应用远程开关", "Set Remote Status")).clicked() {
                        app.set_remote_access();
                    }
                    ui.checkbox(&mut app.allow_remote_access, label_allow_remote);
                });
            });

        ui.separator();

        egui::CollapsingHeader::new(app.tr("策略", "Strategy"))
            .id_source("resources_sub_strategy")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("策略", "Strategy"));
                    let label_least_loaded = app.tr("最小负载", "LeastLoaded");
                    let label_round_robin = app.tr("轮询", "RoundRobin");
                    let label_random = app.tr("随机", "Random");
                    ui.selectable_value(
                        &mut app.balancing_strategy,
                        vangriten_ai_swarm::shared::models::BalancingStrategy::LeastLoaded,
                        label_least_loaded,
                    );
                    ui.selectable_value(
                        &mut app.balancing_strategy,
                        vangriten_ai_swarm::shared::models::BalancingStrategy::RoundRobin,
                        label_round_robin,
                    );
                    ui.selectable_value(
                        &mut app.balancing_strategy,
                        vangriten_ai_swarm::shared::models::BalancingStrategy::Random,
                        label_random,
                    );

                    if ui.button(app.tr("设置", "Set")).clicked() {
                        app.set_balancing_strategy();
                    }
                    if ui.button(app.tr("读取", "Get")).clicked() {
                        app.get_balancing_strategy();
                    }
                });
            });

        ui.separator();

        egui::CollapsingHeader::new(app.tr("Swarm 组", "Swarm Groups"))
            .id_source("resources_sub_groups")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("Group 名称", "Group name"));
                    ui.text_edit_singleline(&mut app.group_name);
                    ui.label(app.tr("最大成员", "Max"));
                    ui.add(egui::DragValue::new(&mut app.group_max_members).clamp_range(1..=10_000));
                    if ui.button(app.tr("创建组", "Create")).clicked() {
                        app.create_swarm_group();
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("Group ID", "Group ID"));
                    ui.text_edit_singleline(&mut app.group_id);
                    if ui.button(app.tr("加入", "Join")).clicked() {
                        app.join_swarm_group();
                    }
                    if ui.button(app.tr("离开", "Leave")).clicked() {
                        app.leave_swarm_group();
                    }
                    if ui.button(app.tr("列出组", "List")).clicked() {
                        app.list_swarm_groups();
                    }
                    if ui.button(app.tr("成员", "Members")).clicked() {
                        app.get_group_members();
                    }
                });
            });

        ui.separator();

        egui::CollapsingHeader::new(app.tr("资源池", "Resource Pools"))
            .id_source("resources_sub_pools")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("Pool 名称", "Pool name"));
                    ui.text_edit_singleline(&mut app.pool_name);
                    ui.label(app.tr("节点IDs (逗号)", "Node IDs (csv)"));
                    ui.text_edit_singleline(&mut app.pool_node_ids_csv);
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button(app.tr("创建 Pool", "Create Pool")).clicked() {
                        app.create_resource_pool();
                    }
                    if ui.button(app.tr("列出 Pools", "List Pools")).clicked() {
                        app.list_resource_pools();
                    }
                });
            });

        ui.separator();

        egui::CollapsingHeader::new(app.tr("资源申请", "Allocations"))
            .id_source("resources_sub_allocations")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("任务类型", "Task type"));
                    ui.text_edit_singleline(&mut app.req_task_type);
                    ui.label(app.tr("优先级", "Priority"));
                    let label_priority_low = app.tr("低", "Low");
                    let label_priority_medium = app.tr("中", "Medium");
                    let label_priority_high = app.tr("高", "High");
                    let label_priority_critical = app.tr("紧急", "Critical");
                    ui.selectable_value(
                        &mut app.req_priority,
                        vangriten_ai_swarm::shared::models::Priority::Low,
                        label_priority_low,
                    );
                    ui.selectable_value(
                        &mut app.req_priority,
                        vangriten_ai_swarm::shared::models::Priority::Medium,
                        label_priority_medium,
                    );
                    ui.selectable_value(
                        &mut app.req_priority,
                        vangriten_ai_swarm::shared::models::Priority::High,
                        label_priority_high,
                    );
                    ui.selectable_value(
                        &mut app.req_priority,
                        vangriten_ai_swarm::shared::models::Priority::Critical,
                        label_priority_critical,
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    let label_gpu_required = app.tr("需要GPU", "GPU");
                    ui.label(app.tr("CPU cores", "CPU cores"));
                    ui.text_edit_singleline(&mut app.req_cpu_cores);
                    ui.label(app.tr("内存MB", "Memory MB"));
                    ui.text_edit_singleline(&mut app.req_memory_mb);
                    ui.checkbox(&mut app.req_gpu_required, label_gpu_required);
                    ui.label(app.tr("GPU MB", "GPU MB"));
                    ui.text_edit_singleline(&mut app.req_gpu_memory_mb);
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("偏好模型(csv)", "Models (csv)"));
                    ui.text_edit_singleline(&mut app.req_preferred_models_csv);
                    if ui.button(app.tr("申请资源", "Request")).clicked() {
                        app.request_resources();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("Allocation ID", "Allocation ID"));
                    ui.text_edit_singleline(&mut app.allocation_id);
                    if ui.button(app.tr("释放", "Release")).clicked() {
                        app.release_allocation();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(app.tr("Node ID", "Node ID"));
                    ui.text_edit_singleline(&mut app.node_id);
                    if ui.button(app.tr("健康检查", "Health Check")).clicked() {
                        app.perform_health_check();
                    }
                });

                egui::ScrollArea::vertical()
                    .id_source("resource_json_scroll")
                    .max_height(260.0)
                    .show(ui, |ui| {
                        ui.monospace(&app.resource_json);
                    });
            });
    }
}
