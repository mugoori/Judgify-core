#!/bin/bash

# ===================================
# Judgify-core Ver2.0 Final - Unified Installation Script (Mac/Linux)
# ===================================
# This script installs all dependencies needed for development
# Supports: macOS (Homebrew), Ubuntu/Debian (apt), Fedora/RHEL (dnf)
#
# Usage:
#   ./scripts/install-all.sh              # Interactive mode
#   ./scripts/install-all.sh --yes        # Auto-confirm all
#   ./scripts/install-all.sh --dry-run    # Show what would be installed
#   ./scripts/install-all.sh --skip-db    # Skip database installation
#
# ===================================

set -e  # Exit on error

# ===================================
# Color Codes
# ===================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# ===================================
# Configuration
# ===================================
AUTO_CONFIRM=false
DRY_RUN=false
SKIP_DB=false
SKIP_SYSTEM=false
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Parse arguments
for arg in "$@"; do
    case $arg in
        --yes|-y)
            AUTO_CONFIRM=true
            ;;
        --dry-run)
            DRY_RUN=true
            ;;
        --skip-db)
            SKIP_DB=true
            ;;
        --skip-system)
            SKIP_SYSTEM=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --yes, -y          Auto-confirm all prompts"
            echo "  --dry-run          Show what would be installed without installing"
            echo "  --skip-db          Skip database installation (PostgreSQL, Redis)"
            echo "  --skip-system      Skip system tools installation (Git, Node.js, etc.)"
            echo "  --help, -h         Show this help message"
            exit 0
            ;;
    esac
done

# ===================================
# Helper Functions
# ===================================
log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_step() {
    echo ""
    echo -e "${MAGENTA}===================================================${NC}"
    echo -e "${MAGENTA}$1${NC}"
    echo -e "${MAGENTA}===================================================${NC}"
}

confirm() {
    if [ "$AUTO_CONFIRM" = true ]; then
        return 0
    fi

    read -p "$1 (y/n): " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]]
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

run_command() {
    if [ "$DRY_RUN" = true ]; then
        echo -e "${BLUE}[DRY-RUN]${NC} Would run: $*"
        return 0
    fi
    "$@"
}

detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ -f /etc/os-release ]]; then
        . /etc/os-release
        echo "$ID"
    else
        echo "unknown"
    fi
}

detect_package_manager() {
    local os=$(detect_os)

    case $os in
        macos)
            echo "brew"
            ;;
        ubuntu|debian)
            echo "apt"
            ;;
        fedora|rhel|centos)
            echo "dnf"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# ===================================
# Main Installation Functions
# ===================================

install_homebrew() {
    log_step "📦 Checking Homebrew (macOS Package Manager)"

    if command_exists brew; then
        log_success "Homebrew already installed: $(brew --version | head -n 1)"
        return 0
    fi

    log_warning "Homebrew not found"

    if confirm "Install Homebrew?"; then
        log_info "Installing Homebrew..."
        run_command /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        log_success "Homebrew installed successfully"
    else
        log_warning "Skipping Homebrew installation"
        return 1
    fi
}

install_git() {
    log_step "📦 Checking Git"

    if command_exists git; then
        log_success "Git already installed: $(git --version)"
        return 0
    fi

    log_warning "Git not found"

    if confirm "Install Git?"; then
        local pkg_mgr=$(detect_package_manager)

        case $pkg_mgr in
            brew)
                run_command brew install git
                ;;
            apt)
                run_command sudo apt update
                run_command sudo apt install -y git
                ;;
            dnf)
                run_command sudo dnf install -y git
                ;;
            *)
                log_error "Unknown package manager. Please install Git manually."
                return 1
                ;;
        esac

        log_success "Git installed successfully"
    else
        log_warning "Skipping Git installation"
    fi
}

install_nodejs() {
    log_step "📦 Checking Node.js"

    if command_exists node; then
        local version=$(node --version)
        log_success "Node.js already installed: $version"

        # Check if version is >= 18
        local major_version=$(echo $version | cut -d'.' -f1 | tr -d 'v')
        if [ "$major_version" -lt 18 ]; then
            log_warning "Node.js version $version is below required 18.x"
            if confirm "Upgrade Node.js to 18.x or higher?"; then
                # Continue to installation
                :
            else
                return 0
            fi
        else
            return 0
        fi
    fi

    log_warning "Node.js not found or needs upgrade"

    if confirm "Install Node.js 18.x LTS?"; then
        local pkg_mgr=$(detect_package_manager)

        case $pkg_mgr in
            brew)
                run_command brew install node@18
                ;;
            apt)
                log_info "Adding NodeSource repository..."
                run_command curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
                run_command sudo apt install -y nodejs
                ;;
            dnf)
                log_info "Adding NodeSource repository..."
                run_command curl -fsSL https://rpm.nodesource.com/setup_18.x | sudo bash -
                run_command sudo dnf install -y nodejs
                ;;
            *)
                log_error "Unknown package manager. Please install Node.js manually."
                return 1
                ;;
        esac

        log_success "Node.js installed successfully: $(node --version)"
    else
        log_warning "Skipping Node.js installation"
    fi
}

install_python() {
    log_step "📦 Checking Python"

    if command_exists python3; then
        local version=$(python3 --version)
        log_success "Python already installed: $version"

        # Check if version is >= 3.11
        local minor_version=$(python3 -c 'import sys; print(sys.version_info.minor)')
        if [ "$minor_version" -lt 11 ]; then
            log_warning "Python version is below required 3.11"
            if confirm "Install Python 3.11+?"; then
                # Continue to installation
                :
            else
                return 0
            fi
        else
            return 0
        fi
    fi

    log_warning "Python 3.11+ not found"

    if confirm "Install Python 3.11?"; then
        local pkg_mgr=$(detect_package_manager)

        case $pkg_mgr in
            brew)
                run_command brew install python@3.11
                ;;
            apt)
                run_command sudo apt update
                run_command sudo apt install -y python3.11 python3.11-venv python3.11-dev python3-pip
                ;;
            dnf)
                run_command sudo dnf install -y python3.11 python3.11-devel
                ;;
            *)
                log_error "Unknown package manager. Please install Python 3.11 manually."
                return 1
                ;;
        esac

        log_success "Python installed successfully: $(python3 --version)"
    else
        log_warning "Skipping Python installation"
    fi
}

install_rust() {
    log_step "📦 Checking Rust"

    if command_exists rustc; then
        log_success "Rust already installed: $(rustc --version)"
        return 0
    fi

    log_warning "Rust not found"

    if confirm "Install Rust (required for Tauri)?"; then
        log_info "Installing Rust via rustup..."
        run_command curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

        # Source cargo env
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        fi

        log_success "Rust installed successfully: $(rustc --version)"
    else
        log_warning "Skipping Rust installation"
    fi
}

install_postgresql() {
    if [ "$SKIP_DB" = true ]; then
        log_warning "Skipping database installation (--skip-db flag)"
        return 0
    fi

    log_step "📦 Checking PostgreSQL"

    if command_exists psql; then
        log_success "PostgreSQL already installed: $(psql --version)"
        return 0
    fi

    log_warning "PostgreSQL not found"

    if confirm "Install PostgreSQL 15+?"; then
        local pkg_mgr=$(detect_package_manager)

        case $pkg_mgr in
            brew)
                run_command brew install postgresql@15
                run_command brew services start postgresql@15
                ;;
            apt)
                run_command sudo apt update
                run_command sudo apt install -y postgresql-15 postgresql-contrib-15
                run_command sudo systemctl enable postgresql
                run_command sudo systemctl start postgresql
                ;;
            dnf)
                run_command sudo dnf install -y postgresql15-server postgresql15-contrib
                run_command sudo postgresql-setup --initdb
                run_command sudo systemctl enable postgresql
                run_command sudo systemctl start postgresql
                ;;
            *)
                log_error "Unknown package manager. Please install PostgreSQL manually."
                return 1
                ;;
        esac

        log_success "PostgreSQL installed successfully"
        log_info "Installing pgvector extension..."

        case $pkg_mgr in
            brew)
                run_command brew install pgvector
                ;;
            apt)
                run_command sudo apt install -y postgresql-15-pgvector
                ;;
            dnf)
                log_warning "pgvector may need to be compiled from source on Fedora/RHEL"
                log_info "See: https://github.com/pgvector/pgvector#installation-notes"
                ;;
        esac

        log_success "PostgreSQL with pgvector installed"
    else
        log_warning "Skipping PostgreSQL installation"
    fi
}

install_redis() {
    if [ "$SKIP_DB" = true ]; then
        log_warning "Skipping database installation (--skip-db flag)"
        return 0
    fi

    log_step "📦 Checking Redis"

    if command_exists redis-server; then
        log_success "Redis already installed: $(redis-server --version)"
        return 0
    fi

    log_warning "Redis not found"

    if confirm "Install Redis 7.0+?"; then
        local pkg_mgr=$(detect_package_manager)

        case $pkg_mgr in
            brew)
                run_command brew install redis
                run_command brew services start redis
                ;;
            apt)
                run_command sudo apt update
                run_command sudo apt install -y redis-server
                run_command sudo systemctl enable redis-server
                run_command sudo systemctl start redis-server
                ;;
            dnf)
                run_command sudo dnf install -y redis
                run_command sudo systemctl enable redis
                run_command sudo systemctl start redis
                ;;
            *)
                log_error "Unknown package manager. Please install Redis manually."
                return 1
                ;;
        esac

        log_success "Redis installed successfully"
    else
        log_warning "Skipping Redis installation"
    fi
}

install_python_deps() {
    log_step "📦 Installing Python Dependencies"

    cd "$PROJECT_ROOT"

    # Create virtual environment
    if [ ! -d "venv" ]; then
        log_info "Creating Python virtual environment..."
        run_command python3 -m venv venv
        log_success "Virtual environment created"
    else
        log_success "Virtual environment already exists"
    fi

    # Activate virtual environment
    log_info "Activating virtual environment..."
    source venv/bin/activate

    # Upgrade pip
    log_info "Upgrading pip..."
    run_command pip install --upgrade pip

    # Install dependencies
    if [ -f "requirements.txt" ]; then
        log_info "Installing Python packages from requirements.txt..."
        run_command pip install -r requirements.txt
        log_success "Python dependencies installed successfully"
    else
        log_warning "requirements.txt not found. Skipping Python dependencies."
    fi
}

install_node_deps() {
    log_step "📦 Installing Node.js Dependencies"

    cd "$PROJECT_ROOT"

    if [ -f "package.json" ]; then
        log_info "Installing Node.js packages..."
        run_command npm install
        log_success "Node.js dependencies installed successfully"
    else
        log_warning "package.json not found. Skipping Node.js dependencies."
    fi
}

install_rust_deps() {
    log_step "📦 Building Rust/Tauri Application"

    cd "$PROJECT_ROOT/src-tauri"

    if [ -f "Cargo.toml" ]; then
        log_info "Building Tauri application (this may take several minutes)..."
        run_command cargo build
        log_success "Tauri application built successfully"
    else
        log_warning "Cargo.toml not found. Skipping Rust build."
    fi

    cd "$PROJECT_ROOT"
}

setup_environment_files() {
    log_step "📦 Setting Up Environment Files"

    cd "$PROJECT_ROOT"

    # Run setup script
    if [ -f "scripts/setup-env.sh" ]; then
        log_info "Running environment setup script..."
        bash scripts/setup-env.sh
    else
        log_warning "setup-env.sh not found. Creating environment files manually..."

        if [ ! -f ".env" ] && [ -f ".env.example" ]; then
            cp .env.example .env
            log_success ".env file created"
        fi

        if [ ! -f ".mcp.json" ] && [ -f ".mcp.template.json" ]; then
            cp .mcp.template.json .mcp.json
            log_success ".mcp.json file created"
        fi
    fi
}

verify_installation() {
    log_step "✅ Verifying Installation"

    local all_ok=true

    # Check system tools
    echo ""
    echo -e "${CYAN}System Tools:${NC}"

    if command_exists git; then
        log_success "Git: $(git --version)"
    else
        log_error "Git: Not installed"
        all_ok=false
    fi

    if command_exists node; then
        log_success "Node.js: $(node --version)"
    else
        log_error "Node.js: Not installed"
        all_ok=false
    fi

    if command_exists python3; then
        log_success "Python: $(python3 --version)"
    else
        log_error "Python: Not installed"
        all_ok=false
    fi

    if command_exists rustc; then
        log_success "Rust: $(rustc --version)"
    else
        log_error "Rust: Not installed"
        all_ok=false
    fi

    # Check databases
    echo ""
    echo -e "${CYAN}Databases:${NC}"

    if command_exists psql; then
        log_success "PostgreSQL: $(psql --version)"
    else
        log_warning "PostgreSQL: Not installed (optional if using Docker)"
    fi

    if command_exists redis-cli; then
        log_success "Redis: $(redis-cli --version)"
    else
        log_warning "Redis: Not installed (optional if using Docker)"
    fi

    # Check environment files
    echo ""
    echo -e "${CYAN}Environment Files:${NC}"

    if [ -f "$PROJECT_ROOT/.env" ]; then
        log_success ".env file exists"
    else
        log_warning ".env file not found"
    fi

    if [ -f "$PROJECT_ROOT/.mcp.json" ]; then
        log_success ".mcp.json file exists"
    else
        log_warning ".mcp.json file not found"
    fi

    # Overall status
    echo ""
    if [ "$all_ok" = true ]; then
        log_success "All required dependencies are installed!"
    else
        log_warning "Some dependencies are missing. Please review the output above."
    fi
}

# ===================================
# Main Execution
# ===================================
main() {
    echo ""
    echo -e "${MAGENTA}╔═══════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║                                                   ║${NC}"
    echo -e "${MAGENTA}║   🚀 Judgify-core Ver2.0 Final Installation      ║${NC}"
    echo -e "${MAGENTA}║                                                   ║${NC}"
    echo -e "${MAGENTA}╚═══════════════════════════════════════════════════╝${NC}"
    echo ""

    if [ "$DRY_RUN" = true ]; then
        log_warning "DRY-RUN MODE: No actual changes will be made"
    fi

    # Detect OS and package manager
    local os=$(detect_os)
    local pkg_mgr=$(detect_package_manager)

    log_info "Detected OS: $os"
    log_info "Package Manager: $pkg_mgr"
    echo ""

    # Install package manager (macOS only)
    if [ "$os" = "macos" ] && [ "$SKIP_SYSTEM" != true ]; then
        install_homebrew
    fi

    # Install system tools
    if [ "$SKIP_SYSTEM" != true ]; then
        install_git
        install_nodejs
        install_python
        install_rust
    fi

    # Install databases
    install_postgresql
    install_redis

    # Install project dependencies
    install_python_deps
    install_node_deps
    install_rust_deps

    # Setup environment files
    setup_environment_files

    # Verify installation
    verify_installation

    # Final instructions
    echo ""
    log_step "📝 Next Steps"
    echo ""
    echo "1. Configure environment variables:"
    echo "   - Edit .env file with your database credentials, API keys, etc."
    echo "   - Edit .mcp.json file with your GitHub Personal Access Token"
    echo ""
    echo "2. Activate Python virtual environment:"
    echo "   source venv/bin/activate"
    echo ""
    echo "3. Start development server:"
    echo "   npm run dev              # Frontend only"
    echo "   npm run tauri:dev        # Full desktop app"
    echo ""
    echo "4. Review detailed documentation:"
    echo "   - README.md: Project overview"
    echo "   - SETUP.md: Detailed setup guide"
    echo "   - INSTALL.md: Installation troubleshooting"
    echo ""
    log_success "Installation complete! 🎉"
    echo ""
}

# Run main function
main "$@"
