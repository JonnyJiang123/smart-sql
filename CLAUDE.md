# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is 智能SQLer (Smart SQLer), an AI-powered database management tool designed to simplify database operations and improve work efficiency. The project aims to provide intelligent SQL generation, performance optimization suggestions, and data analysis capabilities.

## Architecture

Based on the README.md requirements, this project is planned to have a hybrid architecture:

- **Frontend**: Servlet-based with Tailwind CSS and suitable UI components for responsive design
- **Backend**: Rust 1.91 for high-performance API interfaces
- **AI Integration**: OpenAI GPT models for natural language to SQL conversion
- **Cross-platform**: Future plans for web, Windows, and Mac support using Tauri

The UI follows a left-right split design:
- **Left panel**: Database connection list with hierarchical navigation (connections → databases → tables → fields/indices)
- **Right panel**: SQL editor (top) and results display (bottom)

## Key Features

1. **Automatic SQL Generation**: Natural language to SQL conversion
2. **Performance Optimization**: Database performance analysis and optimization suggestions
3. **Intelligent Data Analysis**: AI-powered data analysis with visualization
4. **Multi-Database Support**: MySQL, PostgreSQL, SQLite, plus JDBC connectivity
5. **Security**: OAuth 2.0 authentication for user and database connection security
6. **Data Export**: CSV, Excel export capabilities
7. **Visualization**: Charts (bar, line, pie) for query results

## Current Project State

This appears to be a new project with only documentation and design assets present. The current repository contains:
- README.md with detailed specifications and design requirements
- img/image.png with UI design mockup
- No implementation code yet

## Development Commands

Since there's no package.json or build system present yet, common commands will likely include:
- Build: TBD once build system is established
- Test: TBD when testing framework is implemented
- Lint: TBD when code style tools are configured
- Run: TBD for both Rust backend and servlet frontend

## Database Support

Planned database integrations:
- Primary support: MySQL, PostgreSQL, SQLite
- Extended support: Any JDBC-compatible database
- Connection management through a hierarchical interface

## Security Considerations

- OAuth 2.0 authentication implementation
- Secure storage of database credentials
- SQL injection prevention in AI-generated queries
- User data protection mechanisms

## Future Roadmap

- Multi-language support planned
- Cross-platform deployments (web, Windows, Mac)
- Tauri integration for performance and cross-platform compatibility