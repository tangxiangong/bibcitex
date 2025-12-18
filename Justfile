default:
    @just --list

# Install dependencies
install:
    deno install

# Start development server
dev:
    deno task dev

# Build the application
build:
    deno task build

# Run Vite dev server only (without Tauri)
vite-dev:
    deno task vite:dev

# Build frontend only
vite-build:
    deno task vite:build

# Preview built frontend
preview:
    deno task vite:preview

# Type check
check:
    deno task check

# Format code
fmt:
    deno fmt

# Lint code
lint:
    deno lint

# Clean build artifacts
clean:
    rm -rf build .svelte-kit node_modules
    cd src-tauri && cargo clean

# Generate icons
icon:
    cd icon-gen && python main.py

# Update dependencies
update:
    deno outdated --update
