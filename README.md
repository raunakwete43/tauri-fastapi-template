# TAURI: React + FastAPI Template

A modern desktop application template leveraging [Tauri](https://tauri.app/), [React](https://react.dev/), and [FastAPI](https://fastapi.tiangolo.com/).

## Prerequisites

- **Python** ≥ 3.x
- **Rust** (for Tauri)
- **Node.js** and [pnpm](https://pnpm.io/)

## Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/raunakwete43/tauri-fastapi-template
cd tauri-fastapi-template
```

### 2. Install Node.js Dependencies

```bash
pnpm install
```

### 3. Set Up the Python Backend

```bash
cd fastapi_backend
uv sync
```

### 4. Configure FastAPI Path

Update the absolute path to your FastAPI backend and entry file in `src-tauri/fastapi-config.json`.

### 5. Start the Development Server

```bash
pnpm tauri dev
```

### 6. Build the Application

```bash
pnpm tauri build
```

---

For more details, refer to the [official documentation](https://tauri.app/) or open an issue for support.