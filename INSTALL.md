# 📦 Judgify-core Ver2.0 Final - Installation Guide

Complete installation guide for setting up the development environment on a new PC.

## 📑 Table of Contents

1. [Quick Start](#-quick-start)
2. [System Requirements](#-system-requirements)
3. [Installation Methods](#-installation-methods)
4. [Platform-Specific Guides](#-platform-specific-guides)
5. [Manual Installation](#-manual-installation)
6. [Post-Installation Configuration](#-post-installation-configuration)
7. [Verification](#-verification)
8. [Troubleshooting](#-troubleshooting)
9. [Docker Alternative](#-docker-alternative)

---

## 🚀 Quick Start

### Automated Installation (Recommended)

The fastest way to get started is using our automated installation scripts:

#### macOS / Linux
```bash
# Clone the repository
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

# Run automated installer
chmod +x scripts/install-all.sh
./scripts/install-all.sh
```

#### Windows (PowerShell)
```powershell
# Clone the repository
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

# Run automated installer
.\scripts\install-all.ps1
```

#### Windows (Command Prompt)
```cmd
REM Clone the repository
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

REM Run automated installer
scripts\install-all.bat
```

---

## 💻 System Requirements

### Minimum Requirements

- **OS**: Windows 10/11, macOS 11+, Ubuntu 20.04+, or compatible Linux distribution
- **RAM**: 8GB (16GB recommended for development)
- **Disk Space**: 10GB free space
- **Internet**: Required for downloading dependencies

### Required Software Versions

| Tool | Minimum Version | Recommended Version |
|------|----------------|---------------------|
| **Git** | 2.30+ | Latest |
| **Node.js** | 18.x | 18.19.0 LTS |
| **Python** | 3.11+ | 3.11.x |
| **Rust** | 1.70+ | Latest stable |
| **PostgreSQL** | 15+ | 15.x |
| **Redis** | 7.0+ | 7.2+ |

---

## 🛠 Installation Methods

### Method 1: Automated Installation (Recommended)

**Advantages:**
- Fastest setup time (5-15 minutes)
- Automatic dependency detection
- Error handling and verification
- Platform-specific optimizations

**Use this if:**
- You want the quickest setup
- You're comfortable with automated scripts
- You have internet connection

**Scripts:**
- `scripts/install-all.sh` - macOS/Linux
- `scripts/install-all.ps1` - Windows PowerShell
- `scripts/install-all.bat` - Windows CMD

**Options:**
```bash
# Interactive mode (default)
./scripts/install-all.sh

# Auto-confirm all prompts
./scripts/install-all.sh --yes

# Preview without installing
./scripts/install-all.sh --dry-run

# Skip database installation (use Docker instead)
./scripts/install-all.sh --skip-db

# Skip system tools (already installed)
./scripts/install-all.sh --skip-system
```

### Method 2: Manual Installation

**Use this if:**
- You prefer full control over installations
- You want to use specific versions
- You're troubleshooting installation issues

See [Manual Installation](#-manual-installation) section below.

### Method 3: Docker (Partial)

**Use this if:**
- You want isolated environments
- You only need databases (PostgreSQL + Redis)
- You're experienced with Docker

See [Docker Alternative](#-docker-alternative) section below.

---

## 🖥 Platform-Specific Guides

### macOS

#### Prerequisites
```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Automated Installation
```bash
./scripts/install-all.sh
```

#### What gets installed:
- Git via Homebrew
- Node.js 18.x LTS via Homebrew
- Python 3.11 via Homebrew
- Rust via rustup
- PostgreSQL 15 + pgvector via Homebrew
- Redis 7+ via Homebrew

#### Post-Installation
```bash
# Verify installations
git --version
node --version
python3 --version
rustc --version
psql --version
redis-cli --version

# Activate Python virtual environment
source venv/bin/activate

# Start development
npm run dev
```

---

### Ubuntu / Debian Linux

#### Prerequisites
```bash
# Update package manager
sudo apt update
sudo apt upgrade -y

# Install curl (if needed)
sudo apt install -y curl
```

#### Automated Installation
```bash
chmod +x scripts/install-all.sh
./scripts/install-all.sh
```

#### What gets installed:
- Git via apt
- Node.js 18.x via NodeSource repository
- Python 3.11 via apt
- Rust via rustup
- PostgreSQL 15 + pgvector via apt
- Redis 7+ via apt

#### Post-Installation
```bash
# Verify installations
git --version
node --version
python3 --version
rustc --version
psql --version
redis-cli --version

# Activate Python virtual environment
source venv/bin/activate

# Start development
npm run dev
```

---

### Windows

#### Prerequisites

**Option A: PowerShell (Recommended)**
```powershell
# Install Chocolatey package manager
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Restart PowerShell
```

**Option B: Command Prompt**
- Install Chocolatey manually from: https://chocolatey.org/install

#### Automated Installation

**PowerShell:**
```powershell
.\scripts\install-all.ps1
```

**Command Prompt:**
```cmd
scripts\install-all.bat
```

#### What gets installed:
- Git via Chocolatey
- Node.js 18.x LTS via Chocolatey
- Python 3.11 via Chocolatey
- Rust via rustup-init.exe
- PostgreSQL 15 via Chocolatey (password: `postgres`)
- Memurai (Redis for Windows) via Chocolatey

#### Post-Installation
```powershell
# Verify installations
git --version
node --version
python --version
rustc --version
psql --version
memurai-cli --version  # or redis-cli --version

# Activate Python virtual environment
.\venv\Scripts\Activate.ps1

# Start development
npm run dev
```

#### Windows-Specific Notes

1. **Administrator Privileges**: Some installations require running PowerShell/CMD as Administrator
2. **PATH Updates**: Restart your terminal after installations to refresh PATH
3. **Redis Alternative**: Memurai is used instead of Redis (fully compatible)
4. **pgvector**: May need manual compilation from https://github.com/pgvector/pgvector

---

## 🔧 Manual Installation

### 1. Install Git

**macOS:**
```bash
brew install git
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y git
```

**Windows:**
- Download from: https://git-scm.com/download/win
- Or via Chocolatey: `choco install git -y`

### 2. Install Node.js 18.x LTS

**macOS:**
```bash
brew install node@18
```

**Ubuntu/Debian:**
```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
```

**Windows:**
- Download from: https://nodejs.org/
- Or via Chocolatey: `choco install nodejs-lts --version=18.19.0 -y`

### 3. Install Python 3.11

**macOS:**
```bash
brew install python@3.11
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y python3.11 python3.11-venv python3.11-dev python3-pip
```

**Windows:**
- Download from: https://www.python.org/downloads/
- Or via Chocolatey: `choco install python311 -y`

### 4. Install Rust

**All Platforms:**
```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
# Download and run: https://win.rustup.rs/x86_64
```

### 5. Install PostgreSQL 15

**macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15

# Install pgvector
brew install pgvector
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y postgresql-15 postgresql-contrib-15 postgresql-15-pgvector
sudo systemctl enable postgresql
sudo systemctl start postgresql
```

**Windows:**
- Download from: https://www.postgresql.org/download/windows/
- Or via Chocolatey: `choco install postgresql15 -y --params "/Password:postgres"`
- pgvector: https://github.com/pgvector/pgvector

### 6. Install Redis

**macOS:**
```bash
brew install redis
brew services start redis
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y redis-server
sudo systemctl enable redis-server
sudo systemctl start redis-server
```

**Windows:**
```powershell
# Install Memurai (Redis for Windows)
choco install memurai-developer -y

# Or use Docker
docker run -d -p 6379:6379 redis:7-alpine
```

### 7. Install Project Dependencies

```bash
# Navigate to project directory
cd Judgify-core

# Create Python virtual environment
python3 -m venv venv

# Activate virtual environment
source venv/bin/activate      # macOS/Linux
.\venv\Scripts\Activate.ps1   # Windows PowerShell
venv\Scripts\activate.bat     # Windows CMD

# Install Python dependencies
pip install --upgrade pip
pip install -r requirements.txt

# Install Node.js dependencies
npm install

# Build Tauri application
cd src-tauri
cargo build
cd ..
```

---

## ⚙️ Post-Installation Configuration

### 1. Create Environment Files

**Automated (Recommended):**
```bash
# macOS/Linux
bash scripts/setup-env.sh

# Windows PowerShell
.\scripts\setup-env.ps1

# Windows CMD
scripts\setup-env.bat
```

**Manual:**
```bash
# Copy template files
cp .env.example .env
cp .mcp.template.json .mcp.json
```

### 2. Configure Environment Variables

Edit `.env` file:

```bash
# Database Configuration
POSTGRES_URL=postgresql://user:password@localhost:5432/judgify_core
DATABASE_URL=${POSTGRES_URL}

# GitHub Integration
GITHUB_TOKEN=github_pat_xxxxxxxxxxxxxxxxxxxxx

# CI/CD
CIRCLECI_TOKEN=your-circleci-token-here

# AI/LLM Services
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
OPENAI_MODEL=gpt-4-turbo-preview

# Authentication
JWT_SECRET=your-jwt-secret-key-min-32-characters
JWT_EXPIRES_IN=7d

# Redis
REDIS_URL=redis://localhost:6379

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8006
```

### 3. Configure MCP Server

Edit `.mcp.json` file:

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxxxxxxxxxxxxxxxxxxx"
      }
    },
    "postgresql": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://user:password@localhost:5432/judgify_core"]
    }
  }
}
```

### 4. Initialize Database

```bash
# Create database
createdb judgify_core

# Or with psql
psql -U postgres -c "CREATE DATABASE judgify_core;"

# Enable pgvector extension
psql -U postgres -d judgify_core -c "CREATE EXTENSION IF NOT EXISTS vector;"

# Run migrations (when available)
# python manage.py migrate
```

---

## ✅ Verification

### Run Verification Script

The installation scripts automatically verify installations, or run manually:

```bash
# Check system tools
git --version
node --version
python3 --version
rustc --version

# Check databases
psql --version
redis-cli --version  # or memurai-cli --version on Windows

# Check environment files
ls -la .env .mcp.json  # macOS/Linux
dir .env .mcp.json     # Windows

# Test Python virtual environment
source venv/bin/activate  # macOS/Linux
python --version

# Test Node.js
npm --version

# Test Rust/Tauri
cargo --version
```

### Start Development Server

```bash
# Activate Python virtual environment
source venv/bin/activate  # macOS/Linux
.\venv\Scripts\Activate.ps1  # Windows

# Start frontend only
npm run dev

# Start Tauri desktop app
npm run tauri:dev
```

If the server starts successfully, you're all set! 🎉

---

## 🔍 Troubleshooting

### Common Issues

#### 1. Command Not Found After Installation

**Problem:** `git: command not found`, `node: command not found`, etc.

**Solution:**
```bash
# macOS/Linux
source ~/.bashrc
source ~/.zshrc

# Windows
# Restart PowerShell/CMD to refresh PATH
```

#### 2. Python Version Mismatch

**Problem:** `python --version` shows Python 2.x or wrong version

**Solution:**
```bash
# Use python3 explicitly
python3 --version
python3 -m venv venv

# Or create alias (macOS/Linux)
alias python=python3
```

#### 3. PostgreSQL Connection Error

**Problem:** `FATAL: role "user" does not exist`

**Solution:**
```bash
# Create PostgreSQL user
createuser -s -P judgify_user

# Or with psql
psql -U postgres -c "CREATE USER judgify_user WITH PASSWORD 'your_password' SUPERUSER;"

# Update .env
POSTGRES_URL=postgresql://judgify_user:your_password@localhost:5432/judgify_core
```

#### 4. pgvector Extension Not Found

**Problem:** `ERROR: extension "vector" does not exist`

**Solution:**

**macOS:**
```bash
brew install pgvector
```

**Ubuntu/Debian:**
```bash
sudo apt install postgresql-15-pgvector
```

**Windows:**
- Compile from source: https://github.com/pgvector/pgvector#installation-notes
- Or use Docker: `docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres pgvector/pgvector:pg15`

#### 5. Redis Not Starting on Windows

**Problem:** Redis not officially supported on Windows

**Solution:**

**Option A: Use Memurai**
```powershell
choco install memurai-developer -y
```

**Option B: Use Docker**
```bash
docker run -d -p 6379:6379 --name redis redis:7-alpine
```

**Option C: Use WSL2**
```bash
# In WSL2 Ubuntu
sudo apt install redis-server
sudo service redis-server start
```

#### 6. Rust/Tauri Build Errors

**Problem:** `error: linking with 'cc' failed`

**Solution:**

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Ubuntu/Debian:**
```bash
# Install build essentials
sudo apt install -y build-essential libssl-dev pkg-config
```

**Windows:**
- Install Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/
- Or install via Chocolatey: `choco install visualstudio2022buildtools -y`

#### 7. npm install Fails

**Problem:** `EACCES: permission denied`

**Solution:**

**macOS/Linux:**
```bash
# Fix npm permissions
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# Or use nvm (Node Version Manager)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
```

**Windows:**
- Run PowerShell/CMD as Administrator

#### 8. Virtual Environment Activation Fails

**Problem:** `Activate.ps1 cannot be loaded because running scripts is disabled`

**Solution:**
```powershell
# Windows PowerShell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

## 🐳 Docker Alternative

### Database Setup with Docker

If you prefer using Docker for databases instead of local installations:

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_USER: judgify_user
      POSTGRES_PASSWORD: your_password
      POSTGRES_DB: judgify_core
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

**Usage:**
```bash
# Start databases
docker-compose up -d

# Stop databases
docker-compose down

# View logs
docker-compose logs -f

# Update .env
POSTGRES_URL=postgresql://judgify_user:your_password@localhost:5432/judgify_core
REDIS_URL=redis://localhost:6379
```

**Install remaining tools normally:**
```bash
# Run installer with --skip-db flag
./scripts/install-all.sh --skip-db  # macOS/Linux
.\scripts\install-all.ps1 -SkipDb   # Windows
```

---

## 📚 Additional Resources

### Documentation
- [README.md](README.md) - Project overview
- [SETUP.md](SETUP.md) - Quick setup guide
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [SECURITY.md](SECURITY.md) - Security policies

### External Resources
- [Node.js Documentation](https://nodejs.org/docs)
- [Python Documentation](https://docs.python.org/3/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Redis Documentation](https://redis.io/documentation)
- [Tauri Documentation](https://tauri.app/v1/guides/)

### Support
- GitHub Issues: https://github.com/mugoori/Judgify-core/issues
- GitHub Discussions: https://github.com/mugoori/Judgify-core/discussions

---

## 🎯 Next Steps

After successful installation:

1. **Configure Environment**
   - Edit `.env` with your credentials
   - Edit `.mcp.json` with your GitHub token

2. **Initialize Database**
   - Create database: `createdb judgify_core`
   - Enable extensions: `CREATE EXTENSION vector;`

3. **Start Development**
   - Activate virtual environment: `source venv/bin/activate`
   - Start dev server: `npm run dev`
   - Or start Tauri app: `npm run tauri:dev`

4. **Read Documentation**
   - Review [CLAUDE.md](CLAUDE.md) for architecture details
   - Check [docs/](docs/) for service-specific guides

---

**Happy Coding! 🚀**

If you encounter any issues not covered in this guide, please:
1. Check [Troubleshooting](#-troubleshooting) section
2. Search [GitHub Issues](https://github.com/mugoori/Judgify-core/issues)
3. Create a new issue with detailed error messages and system information
