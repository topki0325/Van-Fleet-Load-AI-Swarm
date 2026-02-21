use crate::shared::models::{AgentTrait, TaskOutput, VgaError, TaskSpec, ContextManager, PerfMetrics};

#[derive(Clone)]
pub struct ArchitectAgent {
    context: ContextManager,
}

impl ArchitectAgent {
    pub fn new() -> Self {
        Self {
            context: ContextManager {
                memory_slots: std::collections::HashMap::new(),
                docs: vec![],
            },
        }
    }

    /// Analyze project requirements and generate architecture blueprint
    fn analyze_requirements(&self, requirements: &str) -> Result<ArchitectureBlueprint, VgaError> {
        // Parse requirements and extract key components
        let components = self.extract_components(requirements)?;
        let patterns = self.identify_patterns(&components)?;
        let tech_stack = self.recommend_tech_stack(&components)?;
        let interfaces = self.define_interfaces(&components)?;
        let data_flow = self.design_data_flow(&components)?;

        Ok(ArchitectureBlueprint {
            components,
            patterns,
            tech_stack,
            interfaces,
            data_flow,
        })
    }

    fn extract_components(&self, requirements: &str) -> Result<Vec<Component>, VgaError> {
        // Simple component extraction based on keywords
        let mut components = Vec::new();

        if requirements.contains("web") || requirements.contains("http") {
            components.push(Component {
                name: "WebServer".to_string(),
                component_type: ComponentType::Service,
                technologies: vec!["HTTP".to_string(), "REST".to_string()],
                dependencies: vec![],
            });
        }

        if requirements.contains("database") || requirements.contains("data") {
            components.push(Component {
                name: "Database".to_string(),
                component_type: ComponentType::DataStore,
                technologies: vec!["PostgreSQL".to_string()],
                dependencies: vec![],
            });
        }

        if requirements.contains("auth") || requirements.contains("login") {
            components.push(Component {
                name: "AuthService".to_string(),
                component_type: ComponentType::Service,
                technologies: vec!["JWT".to_string(), "OAuth".to_string()],
                dependencies: vec![],
            });
        }

        if requirements.contains("ui") || requirements.contains("frontend") {
            components.push(Component {
                name: "UserInterface".to_string(),
                component_type: ComponentType::UI,
                technologies: vec!["Tauri".to_string(), "HTML".to_string()],
                dependencies: vec![],
            });
        }

        if requirements.contains("worker") || requirements.contains("queue") {
            components.push(Component {
                name: "Worker".to_string(),
                component_type: ComponentType::Worker,
                technologies: vec!["Tokio".to_string()],
                dependencies: vec![],
            });
        }

        Ok(components)
    }

    fn identify_patterns(&self, components: &[Component]) -> Result<Vec<ArchitecturePattern>, VgaError> {
        let mut patterns = Vec::new();

        if components.len() > 3 {
            patterns.push(ArchitecturePattern::Microservices);
        } else {
            patterns.push(ArchitecturePattern::Monolithic);
        }

        if components.iter().any(|c| c.component_type == ComponentType::Service) {
            patterns.push(ArchitecturePattern::Layered);
        }

        if components.iter().any(|c| matches!(c.component_type, ComponentType::Worker)) {
            patterns.push(ArchitecturePattern::EventDriven);
        }

        Ok(patterns)
    }

    fn recommend_tech_stack(&self, components: &[Component]) -> Result<Vec<String>, VgaError> {
        let mut tech_stack = vec!["Rust".to_string()]; // Base technology

        for component in components {
            for tech in &component.technologies {
                if !tech_stack.contains(tech) {
                    tech_stack.push(tech.clone());
                }
            }
        }

        Ok(tech_stack)
    }

    fn define_interfaces(&self, components: &[Component]) -> Result<Vec<Interface>, VgaError> {
        let mut interfaces = Vec::new();

        for component in components {
            interfaces.push(Interface {
                name: format!("{}API", component.name),
                component: component.name.clone(),
                methods: vec!["create".to_string(), "read".to_string(), "update".to_string(), "delete".to_string()],
                protocol: "REST".to_string(),
            });
        }

        Ok(interfaces)
    }

    fn design_data_flow(&self, components: &[Component]) -> Result<Vec<DataFlow>, VgaError> {
        let mut flows = Vec::new();

        for i in 0..components.len() {
            for j in (i + 1)..components.len() {
                flows.push(DataFlow {
                    from: components[i].name.clone(),
                    to: components[j].name.clone(),
                    data_type: "RequestResponse".to_string(),
                    protocol: "HTTP".to_string(),
                });
            }
        }

        Ok(flows)
    }

    fn design_system_architecture(&self, context: &str) -> Result<TaskOutput, VgaError> {
        let blueprint = self.analyze_requirements(context)?;

        Ok(TaskOutput {
            content: format!(
                "System Architecture Design:\n\nHigh-Level Components:\n{}\n\nArchitecture Patterns:\n{}\n\nTechnology Stack:\n{}",
                blueprint.components.iter().map(|c| format!("- {} ({:?})", c.name, c.component_type)).collect::<Vec<_>>().join("\n"),
                blueprint.patterns.iter().map(|p| format!("- {:?}", p)).collect::<Vec<_>>().join("\n"),
                blueprint.tech_stack.join(", ")
            ),
            metadata: std::collections::HashMap::new(),
        })
    }

    fn generate_api_specification(&self, context: &str) -> Result<TaskOutput, VgaError> {
        let blueprint = self.analyze_requirements(context)?;

        let api_spec = blueprint.interfaces.iter()
            .map(|interface| {
                format!(
                    "API: {}\nComponent: {}\nMethods: {}\nProtocol: {}\n",
                    interface.name,
                    interface.component,
                    interface.methods.join(", "),
                    interface.protocol
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n");

        Ok(TaskOutput {
            content: format!("API Specification:\n\n{}", api_spec),
            metadata: std::collections::HashMap::new(),
        })
    }

    fn design_data_model(&self, context: &str) -> Result<TaskOutput, VgaError> {
        let blueprint = self.analyze_requirements(context)?;

        let data_model = blueprint.data_flow.iter()
            .map(|flow| {
                format!(
                    "Data Flow: {} -> {} ({} via {})",
                    flow.from, flow.to, flow.data_type, flow.protocol
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(TaskOutput {
            content: format!("Data Model Design:\n\n{}", data_model),
            metadata: std::collections::HashMap::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ArchitectureBlueprint {
    pub components: Vec<Component>,
    pub patterns: Vec<ArchitecturePattern>,
    pub tech_stack: Vec<String>,
    pub interfaces: Vec<Interface>,
    pub data_flow: Vec<DataFlow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub component_type: ComponentType,
    pub technologies: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Service,
    DataStore,
    UI,
    Worker,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchitecturePattern {
    Monolithic,
    Microservices,
    Layered,
    EventDriven,
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    pub component: String,
    pub methods: Vec<String>,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub struct DataFlow {
    pub from: String,
    pub to: String,
    pub data_type: String,
    pub protocol: String,
}

#[async_trait::async_trait]
impl AgentTrait for ArchitectAgent {
    async fn execute_instruction(&self, instr: String) -> Result<TaskOutput, VgaError> {
        let _ = &self.context;
        // Generate architecture blueprint based on requirements
        let blueprint = self.analyze_requirements(&instr)?;

        let content = format!(
            "Architecture Blueprint:\n\nComponents:\n{}\n\nPatterns:\n{}\n\nTech Stack:\n{}\n\nInterfaces:\n{}\n\nData Flow:\n{}",
            blueprint.components.iter().map(|c| format!("- {} ({:?})", c.name, c.component_type)).collect::<Vec<_>>().join("\n"),
            blueprint.patterns.iter().map(|p| format!("- {:?}", p)).collect::<Vec<_>>().join("\n"),
            blueprint.tech_stack.join(", "),
            blueprint.interfaces.iter().map(|i| format!("- {}: {} ({})", i.name, i.methods.join(", "), i.protocol)).collect::<Vec<_>>().join("\n"),
            blueprint.data_flow.iter().map(|f| format!("- {} -> {} ({})", f.from, f.to, f.protocol)).collect::<Vec<_>>().join("\n")
        );

        Ok(TaskOutput {
            content,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("component_count".to_string(), blueprint.components.len().to_string());
                meta.insert("pattern_count".to_string(), blueprint.patterns.len().to_string());
                meta.insert("tech_stack_size".to_string(), blueprint.tech_stack.len().to_string());
                meta
            },
        })
    }

    async fn execute_block(&self, task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        let _ = &self.context;
        // Handle specific architecture tasks
        match task_spec.target.as_str() {
            "design-system" => self.design_system_architecture(&task_spec.context_range),
            "api-spec" => self.generate_api_specification(&task_spec.context_range),
            "data-model" => self.design_data_model(&task_spec.context_range),
            _ => self.design_system_architecture(&task_spec.context_range),
        }
    }

    fn update_context(&mut self, context: &ContextManager) {
        self.context = context.clone();
    }

    fn get_metrics(&self) -> PerfMetrics {
        PerfMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            avg_response_time: std::time::Duration::from_millis(100),
        }
    }
}