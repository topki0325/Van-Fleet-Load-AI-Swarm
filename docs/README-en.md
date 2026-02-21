# Vangriten AI Swarm - Documentation Library

Welcome to the detailed documentation for the Vangriten AI Swarm framework. This documentation library aims to provide a comprehensive understanding of the system architecture, implementation details, best practices, and usage guidelines.

## Documentation Index

- [Core Architecture](architecture-en.md): Overview of the system's core concepts, components, and three-layer approach.
- [Agent Roles and Capabilities](agents-en.md): Detailed introduction to different types of AI agents and their communication protocols.
- [Technical Implementation](technical-implementation-en.md): Implementation details including high-concurrency design, modular compilation scheduling, network discovery, and API key security.
- [Project Structure and Module Design](structure/file-tree-en.md): Detailed project directory layout and modular design philosophy.
  - [Frontend API Interface](structure/frontend-api-en.md): UI logic and backend command set in Tauri mode.
  - [Backend Core Services](structure/backend-services-en.md): Gatling scheduling, environment management, and distributed compute nodes.
  - [Shared Data Models](structure/data-models-en.md): Unified project, agent, and task entity specifications.
- [Getting Started](getting-started-en.md): Environment setup, installation steps, configuration guide, and running instructions.
- [Component Rules](component-rules-en.md): GUI sub-function group componentization specifications (for easier compilation splitting and reduced recompilation time).
- [Best Practices](best-practices-en.md): Recommendations for agent specialization, concurrency management, security, and performance optimization.
- [Future Enhancements](future-en.md): Plans for multimodal, federated learning, and CI/CD integration.
- [Project Progress](progress-en.md): Current development progress, verifiable features, and next steps.

## Feature Module Documentation

- [API Key Management](api-key-management-en.md): Manage API keys for multiple AI service providers with encrypted storage and usage statistics.
- [Resource Manager Agent](resource-manager-en.md): LAN AI model resource discovery, distributed GPU resource allocation, network load balancing, and more.
- [C Compiler Environment Management](c-compiler-en.md): Automatic GCC instance discovery, round-robin compilation, parallel compilation, and more.
- [Ollama Integration](ollama-en.md): Run various open-source AI models locally with support for chat, text generation, and vector embeddings.

## Other Important Files

- [README](../README.md): Project overview and quick start.
- [CONTRIBUTING](../CONTRIBUTING.md): How to contribute to project development.
- [CHANGELOG](../CHANGELOG.md): Version update history.
- [LICENSE](../LICENSE): MIT license information.
- [CODE_OF_CONDUCT](../CODE_OF_CONDUCT.md): Community code of conduct.

## Documentation Language

This documentation is available in both English and Chinese:

- **English**: Files ending with `-en.md`
- **Chinese**: Files without language suffix (default Chinese version)

For example:
- `ollama.md` - Chinese documentation
- `ollama-en.md` - English documentation

## Quick Links

- [Quick Start Guide](getting-started-en.md)
- [Architecture Overview](architecture-en.md)
- [API Reference](structure/frontend-api-en.md)
- [Best Practices](best-practices-en.md)

## Support

If you have questions or need help, please visit:
- [GitHub Issues](https://github.com/topki0325/Vangriten-AI-swarm/issues)
- [GitHub Discussions](https://github.com/topki0325/Vangriten-AI-swarm/discussions)
