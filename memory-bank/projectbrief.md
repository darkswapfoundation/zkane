# ZKane Frontend Project Brief

## Project Overview
ZKane is a privacy-preserving web application for alkanes assets using zero-knowledge proofs. The frontend is a full-stack Rust web application built with Leptos and WebAssembly that provides a user-friendly interface for interacting with ZKane privacy pools.

## Core Requirements
- **Privacy-First Design**: Zero-knowledge proofs for anonymous transactions
- **Multi-Asset Support**: Works with any alkanes-compatible asset
- **Local Storage**: Deposit notes stored securely in browser
- **No Server Dependencies**: Fully client-side application with WASM
- **Responsive Design**: Works on desktop and mobile devices

## Key Features
1. **Deposit alkanes assets** into privacy pools with configurable denominations
2. **Generate withdrawal proofs** using zero-knowledge cryptography
3. **Manage deposit notes** securely in local storage
4. **Browse privacy pools** and their anonymity sets
5. **Track transaction history** and manage user preferences

## Technology Stack
- **Frontend Framework**: Leptos (Reactive web framework for Rust)
- **WebAssembly**: Compiled Rust code running in the browser
- **Styling**: Custom CSS with CSS variables for theming
- **Build Tool**: Trunk for WASM compilation and development server
- **Testing**: wasm-bindgen-test for browser-based testing

## Current Status
The frontend has been previously developed and appears to have working components, styling, and build configuration. Previous work includes:
- Complete application structure with routing
- Enhanced CSS styling (28KB) with theme support
- Fixed loading screen and asset serving issues
- Comprehensive component library
- Service layer for ZKane and alkanes integration

## Goal
Build and deploy the zkane frontend, ensuring all components work correctly and the application provides a smooth user experience for privacy-preserving transactions.