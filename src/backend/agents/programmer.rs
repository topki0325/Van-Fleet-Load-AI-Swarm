use crate::shared::models::{AgentTrait, TaskOutput, VgaError, TaskSpec, ContextManager, PerfMetrics};

#[derive(Clone)]
pub struct ProgrammerAgent {
    context: ContextManager,
}

impl ProgrammerAgent {
    pub fn new() -> Self {
        Self {
            context: ContextManager {
                memory_slots: std::collections::HashMap::new(),
                docs: vec![],
            },
        }
    }

    /// Generate code based on specifications
    fn generate_code(&self, language: &str, spec: &str) -> Result<String, VgaError> {
        match language.to_lowercase().as_str() {
            "rust" => self.generate_rust_code(spec),
            "python" => self.generate_python_code(spec),
            "javascript" | "js" => self.generate_javascript_code(spec),
            "typescript" | "ts" => self.generate_typescript_code(spec),
            _ => Err(VgaError::CompileFailure(format!("Unsupported language: {}", language))),
        }
    }

    fn generate_rust_code(&self, spec: &str) -> Result<String, VgaError> {
        if spec.contains("struct") || spec.contains("data") {
            Ok(format!(
                "/// Generated Rust struct\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct {} {{\n    pub id: u64,\n    pub name: String,\n    pub created_at: DateTime<Utc>,\n}}\n\nimpl {} {{\n    pub fn new(name: String) -> Self {{\n        Self {{\n            id: 0, // TODO: Generate proper ID\n            name,\n            created_at: Utc::now(),\n        }}\n    }}\n}}",
                self.extract_entity_name(spec),
                self.extract_entity_name(spec)
            ))
        } else if spec.contains("function") || spec.contains("method") {
            Ok(format!(
                "/// Generated Rust function\npub fn {}(input: &str) -> Result<String, Box<dyn std::error::Error>> {{\n    // TODO: Implement {}\n    Ok(format!(\"Processed: {{}}\", input))\n}}",
                self.extract_function_name(spec),
                spec
            ))
        } else {
            Ok(format!("// Generated Rust code for: {}\n// TODO: Implement specific logic\npub fn process() -> Result<(), Box<dyn std::error::Error>> {{\n    println!(\"Processing: {}\");\n    Ok(())\n}}", spec, spec))
        }
    }

    fn generate_python_code(&self, spec: &str) -> Result<String, VgaError> {
        if spec.contains("class") || spec.contains("data") {
            Ok(format!(
                "# Generated Python class\nclass {}:\n    def __init__(self, name: str):\n        self.id = 0  # TODO: Generate proper ID\n        self.name = name\n        self.created_at = datetime.now()\n\n    def __str__(self):\n        return f\"{{self.name}} (ID: {{self.id}})\"\n\n    @classmethod\n    def from_dict(cls, data: dict):\n        return cls(data.get('name', ''))",
                self.extract_entity_name(spec)
            ))
        } else if spec.contains("function") {
            Ok(format!(
                "# Generated Python function\ndef {}(input_str: str) -> str:\n    \"\"\"\n    Process input string\n    \n    Args:\n        input_str: Input to process\n    \n    Returns:\n        Processed string\n    \"\"\"\n    # TODO: Implement {}\n    return f\"Processed: {{input_str}}\"",
                self.extract_function_name(spec),
                spec
            ))
        } else {
            Ok(format!("# Generated Python code for: {}\n# TODO: Implement specific logic\ndef process():\n    print(f\"Processing: {}\")\n    return True", spec, spec))
        }
    }

    fn generate_javascript_code(&self, spec: &str) -> Result<String, VgaError> {
        if spec.contains("class") || spec.contains("component") {
            Ok(format!(
                "// Generated JavaScript class\nclass {} {{\n    constructor(name) {{\n        this.id = 0; // TODO: Generate proper ID\n        this.name = name;\n        this.createdAt = new Date();\n    }}\n\n    toString() {{\n        return `${{this.name}} (ID: ${{this.id}})`;\n    }}\n\n    static fromObject(data) {{\n        return new {}(data.name || '');\n    }}\n}}",
                self.extract_entity_name(spec),
                self.extract_entity_name(spec)
            ))
        } else if spec.contains("function") {
            Ok(format!(
                "// Generated JavaScript function\nfunction {}(inputStr) {{\n    // TODO: Implement {}\n    return `Processed: ${{inputStr}}`;\n}}",
                self.extract_function_name(spec),
                spec
            ))
        } else {
            Ok(format!("// Generated JavaScript code for: {}\n// TODO: Implement specific logic\nfunction process() {{\n    console.log(`Processing: {}`);\n    return true;\n}}", spec, spec))
        }
    }

    fn generate_typescript_code(&self, spec: &str) -> Result<String, VgaError> {
        if spec.contains("interface") || spec.contains("type") {
            Ok(format!(
                "// Generated TypeScript interface\nexport interface {} {{\n    id: number;\n    name: string;\n    createdAt: Date;\n}}\n\nexport class {} implements {} {{\n    constructor(\n        public id: number = 0,\n        public name: string = '',\n        public createdAt: Date = new Date()\n    ) {{}}\n\n    toString(): string {{\n        return `${{this.name}} (ID: ${{this.id}})`;\n    }}\n\n    static fromObject(data: Partial<{}>): {} {{\n        return new {}(\n            data.id || 0,\n            data.name || '',\n            data.createdAt || new Date()\n        );\n    }}\n}}",
                self.extract_entity_name(spec),
                self.extract_entity_name(spec),
                self.extract_entity_name(spec),
                self.extract_entity_name(spec),
                self.extract_entity_name(spec),
                self.extract_entity_name(spec)
            ))
        } else if spec.contains("function") {
            Ok(format!(
                "// Generated TypeScript function\nexport function {}(inputStr: string): string {{\n    // TODO: Implement {}\n    return `Processed: ${{inputStr}}`;\n}}",
                self.extract_function_name(spec),
                spec
            ))
        } else {
            Ok(format!("// Generated TypeScript code for: {}\n// TODO: Implement specific logic\nexport function process(): boolean {{\n    console.log(`Processing: {}`);\n    return true;\n}}", spec, spec))
        }
    }

    fn extract_entity_name(&self, spec: &str) -> String {
        // Simple entity name extraction - in real implementation, this would be more sophisticated
        if spec.contains("user") {
            "User".to_string()
        } else if spec.contains("product") {
            "Product".to_string()
        } else if spec.contains("order") {
            "Order".to_string()
        } else {
            "Entity".to_string()
        }
    }

    fn extract_function_name(&self, spec: &str) -> String {
        // Simple function name extraction
        if spec.contains("process") {
            "processData".to_string()
        } else if spec.contains("validate") {
            "validateInput".to_string()
        } else if spec.contains("calculate") {
            "calculateResult".to_string()
        } else {
            "executeTask".to_string()
        }
    }

    fn detect_language(&self, instruction: &str) -> String {
        let instr_lower = instruction.to_lowercase();
        if instr_lower.contains("rust") || instr_lower.contains("cargo") {
            "rust".to_string()
        } else if instr_lower.contains("python") || instr_lower.contains("pip") {
            "python".to_string()
        } else if instr_lower.contains("typescript") || instr_lower.contains("ts") {
            "typescript".to_string()
        } else if instr_lower.contains("javascript") || instr_lower.contains("js") {
            "javascript".to_string()
        } else {
            "rust".to_string() // Default to Rust
        }
    }

    /// Generate unit tests for the given code
    fn generate_tests(&self, language: &str, _code: &str) -> Result<String, VgaError> {
        match language.to_lowercase().as_str() {
            "rust" => Ok(format!("/// Generated Rust tests\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_basic_functionality() {{\n        // TODO: Add proper test cases\n        assert!(true);\n    }}\n}}")),
            "python" => Ok(format!("# Generated Python tests\nimport unittest\n\nclass TestGeneratedCode(unittest.TestCase):\n    def test_basic_functionality(self):\n        # TODO: Add proper test cases\n        self.assertTrue(True)\n\nif __name__ == '__main__':\n    unittest.main()")),
            "javascript" => Ok(format!("// Generated JavaScript tests\ndescribe('Generated Code Tests', () => {{\n    test('basic functionality', () => {{\n        // TODO: Add proper test cases\n        expect(true).toBe(true);\n    }});\n}});")),
            "typescript" => Ok(format!("// Generated TypeScript tests\nimport {{ describe, test, expect }} from '@jest/globals';\n\ndescribe('Generated Code Tests', () => {{\n    test('basic functionality', () => {{\n        // TODO: Add proper test cases\n        expect(true).toBe(true);\n    }});\n}});")),
            _ => Err(VgaError::CompileFailure(format!("Unsupported language for tests: {}", language))),
        }
    }
}

#[async_trait::async_trait]
impl AgentTrait for ProgrammerAgent {
    async fn execute_instruction(&self, instr: String) -> Result<TaskOutput, VgaError> {
        let _ = &self.context;
        // Generate code based on instruction
        let language = self.detect_language(&instr);
        let code = self.generate_code(&language, &instr)?;

        Ok(TaskOutput {
            content: format!("Generated {} code:\n\n{}", language, code),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("language".to_string(), language);
                meta.insert("code_length".to_string(), code.len().to_string());
                meta
            },
        })
    }

    async fn execute_block(&self, task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        let _ = &self.context;
        // Generate code for specific task
        let _ = self.detect_language(&task_spec.context_range);
        let code = self.generate_code(&task_spec.language, &task_spec.context_range)?;

        // Generate tests if requested
        let tests = if task_spec.target.contains("test") {
            Some(self.generate_tests(&task_spec.language, &code)?)
        } else {
            None
        };

        let has_tests = tests.is_some();

        let content = if let Some(test_code) = tests {
            format!("Generated {} code:\n\n{}\n\nTests:\n\n{}", task_spec.language, code, test_code)
        } else {
            format!("Generated {} code:\n\n{}", task_spec.language, code)
        };

        Ok(TaskOutput {
            content,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("language".to_string(), task_spec.language.clone());
                meta.insert("target".to_string(), task_spec.target.clone());
                meta.insert("has_tests".to_string(), has_tests.to_string());
                meta
            },
        })
    }

    fn update_context(&mut self, context: &ContextManager) {
        self.context = context.clone();
    }

    fn get_metrics(&self) -> PerfMetrics {
        PerfMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            avg_response_time: std::time::Duration::from_millis(150),
        }
    }
}