# ===================================
# Judgify-core Ver2.0 Final - Unified Installation Script (Windows PowerShell)
# ===================================
# This script installs all dependencies needed for development on Windows
# Requires: Windows 10/11, PowerShell 5.1 or later
#
# Usage:
#   .\scripts\install-all.ps1              # Interactive mode
#   .\scripts\install-all.ps1 -AutoConfirm # Auto-confirm all
#   .\scripts\install-all.ps1 -DryRun      # Show what would be installed
#   .\scripts\install-all.ps1 -SkipDb      # Skip database installation
#
# ===================================

param(
    [switch]$AutoConfirm,
    [switch]$DryRun,
    [switch]$SkipDb,
    [switch]$SkipSystem,
    [switch]$Help
)

# Exit on error
$ErrorActionPreference = "Stop"

# ===================================
# Helper Functions
# ===================================
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Type = "Info"
    )

    $color = switch ($Type) {
        "Info"    { "Cyan" }
        "Success" { "Green" }
        "Warning" { "Yellow" }
        "Error"   { "Red" }
        "Step"    { "Magenta" }
        default   { "White" }
    }

    $prefix = switch ($Type) {
        "Info"    { "[INFO]" }
        "Success" { "[✓]" }
        "Warning" { "[⚠]" }
        "Error"   { "[✗]" }
        "Step"    { "" }
        default   { "" }
    }

    Write-Host "$prefix $Message" -ForegroundColor $color
}

function Write-Step {
    param([string]$Message)

    Write-Host ""
    Write-Host "===================================================" -ForegroundColor Magenta
    Write-Host $Message -ForegroundColor Magenta
    Write-Host "===================================================" -ForegroundColor Magenta
}

function Confirm-Action {
    param([string]$Message)

    if ($AutoConfirm) {
        return $true
    }

    $response = Read-Host "$Message (y/n)"
    return $response -match "^[Yy]$"
}

function Test-CommandExists {
    param([string]$Command)

    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

function Invoke-SafeCommand {
    param(
        [string]$Command,
        [string[]]$Arguments = @()
    )

    if ($DryRun) {
        Write-ColorOutput "[DRY-RUN] Would run: $Command $($Arguments -join ' ')" "Info"
        return $true
    }

    try {
        & $Command @Arguments
        return $true
    }
    catch {
        Write-ColorOutput "Command failed: $_" "Error"
        return $false
    }
}

function Test-AdminPrivileges {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# ===================================
# Installation Functions
# ===================================

function Install-Chocolatey {
    Write-Step "📦 Checking Chocolatey (Windows Package Manager)"

    if (Test-CommandExists "choco") {
        $version = & choco --version
        Write-ColorOutput "Chocolatey already installed: $version" "Success"
        return $true
    }

    Write-ColorOutput "Chocolatey not found" "Warning"

    if (Confirm-Action "Install Chocolatey (package manager)?") {
        if (-not (Test-AdminPrivileges)) {
            Write-ColorOutput "Administrator privileges required to install Chocolatey" "Error"
            Write-ColorOutput "Please run PowerShell as Administrator" "Warning"
            return $false
        }

        Write-ColorOutput "Installing Chocolatey..." "Info"

        if ($DryRun) {
            Write-ColorOutput "[DRY-RUN] Would install Chocolatey" "Info"
            return $true
        }

        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

        Write-ColorOutput "Chocolatey installed successfully" "Success"
        return $true
    }
    else {
        Write-ColorOutput "Skipping Chocolatey installation" "Warning"
        return $false
    }
}

function Install-Git {
    Write-Step "📦 Checking Git"

    if (Test-CommandExists "git") {
        $version = & git --version
        Write-ColorOutput "Git already installed: $version" "Success"
        return $true
    }

    Write-ColorOutput "Git not found" "Warning"

    if (Confirm-Action "Install Git?") {
        if (Test-CommandExists "choco") {
            Write-ColorOutput "Installing Git via Chocolatey..." "Info"
            Invoke-SafeCommand "choco" @("install", "git", "-y")
        }
        else {
            Write-ColorOutput "Please install Git manually from: https://git-scm.com/download/win" "Warning"
            return $false
        }

        Write-ColorOutput "Git installed successfully" "Success"
        Write-ColorOutput "Please restart PowerShell to use Git" "Warning"
        return $true
    }
    else {
        Write-ColorOutput "Skipping Git installation" "Warning"
        return $false
    }
}

function Install-NodeJs {
    Write-Step "📦 Checking Node.js"

    if (Test-CommandExists "node") {
        $version = & node --version
        Write-ColorOutput "Node.js already installed: $version" "Success"

        # Check if version is >= 18
        $majorVersion = [int]($version -replace 'v', '' -split '\.')[0]
        if ($majorVersion -lt 18) {
            Write-ColorOutput "Node.js version $version is below required 18.x" "Warning"
            if (Confirm-Action "Upgrade Node.js to 18.x or higher?") {
                # Continue to installation
            }
            else {
                return $true
            }
        }
        else {
            return $true
        }
    }

    Write-ColorOutput "Node.js not found or needs upgrade" "Warning"

    if (Confirm-Action "Install Node.js 18.x LTS?") {
        if (Test-CommandExists "choco") {
            Write-ColorOutput "Installing Node.js via Chocolatey..." "Info"
            Invoke-SafeCommand "choco" @("install", "nodejs-lts", "--version=18.19.0", "-y")
        }
        else {
            Write-ColorOutput "Please install Node.js manually from: https://nodejs.org/" "Warning"
            return $false
        }

        Write-ColorOutput "Node.js installed successfully" "Success"
        Write-ColorOutput "Please restart PowerShell to use Node.js" "Warning"
        return $true
    }
    else {
        Write-ColorOutput "Skipping Node.js installation" "Warning"
        return $false
    }
}

function Install-Python {
    Write-Step "📦 Checking Python"

    if (Test-CommandExists "python") {
        $version = & python --version
        Write-ColorOutput "Python already installed: $version" "Success"

        # Check if version is >= 3.11
        $versionMatch = $version -match "(\d+)\.(\d+)"
        if ($versionMatch) {
            $minorVersion = [int]$Matches[2]
            if ($minorVersion -lt 11) {
                Write-ColorOutput "Python version is below required 3.11" "Warning"
                if (Confirm-Action "Install Python 3.11+?") {
                    # Continue to installation
                }
                else {
                    return $true
                }
            }
            else {
                return $true
            }
        }
    }

    Write-ColorOutput "Python 3.11+ not found" "Warning"

    if (Confirm-Action "Install Python 3.11?") {
        if (Test-CommandExists "choco") {
            Write-ColorOutput "Installing Python via Chocolatey..." "Info"
            Invoke-SafeCommand "choco" @("install", "python311", "-y")
        }
        else {
            Write-ColorOutput "Please install Python manually from: https://www.python.org/downloads/" "Warning"
            return $false
        }

        Write-ColorOutput "Python installed successfully" "Success"
        Write-ColorOutput "Please restart PowerShell to use Python" "Warning"
        return $true
    }
    else {
        Write-ColorOutput "Skipping Python installation" "Warning"
        return $false
    }
}

function Install-Rust {
    Write-Step "📦 Checking Rust"

    if (Test-CommandExists "rustc") {
        $version = & rustc --version
        Write-ColorOutput "Rust already installed: $version" "Success"
        return $true
    }

    Write-ColorOutput "Rust not found" "Warning"

    if (Confirm-Action "Install Rust (required for Tauri)?") {
        Write-ColorOutput "Installing Rust via rustup..." "Info"

        if ($DryRun) {
            Write-ColorOutput "[DRY-RUN] Would install Rust via rustup" "Info"
            return $true
        }

        # Download and run rustup-init
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupPath = "$env:TEMP\rustup-init.exe"

        Write-ColorOutput "Downloading rustup-init..." "Info"
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath

        Write-ColorOutput "Running rustup-init..." "Info"
        & $rustupPath -y

        # Add cargo to PATH for current session
        $env:Path += ";$env:USERPROFILE\.cargo\bin"

        Write-ColorOutput "Rust installed successfully" "Success"
        Write-ColorOutput "Please restart PowerShell to use Rust" "Warning"
        return $true
    }
    else {
        Write-ColorOutput "Skipping Rust installation" "Warning"
        return $false
    }
}

function Install-PostgreSQL {
    if ($SkipDb) {
        Write-ColorOutput "Skipping database installation (--SkipDb flag)" "Warning"
        return $true
    }

    Write-Step "📦 Checking PostgreSQL"

    if (Test-CommandExists "psql") {
        $version = & psql --version
        Write-ColorOutput "PostgreSQL already installed: $version" "Success"
        return $true
    }

    Write-ColorOutput "PostgreSQL not found" "Warning"

    if (Confirm-Action "Install PostgreSQL 15+?") {
        if (Test-CommandExists "choco") {
            Write-ColorOutput "Installing PostgreSQL via Chocolatey..." "Info"
            Invoke-SafeCommand "choco" @("install", "postgresql15", "-y", "--params", "/Password:postgres")

            Write-ColorOutput "PostgreSQL installed successfully" "Success"
            Write-ColorOutput "Default password: postgres" "Warning"
            Write-ColorOutput "Please change the default password!" "Warning"

            # Install pgvector extension
            Write-ColorOutput "Installing pgvector extension..." "Info"
            Write-ColorOutput "pgvector needs to be installed manually from: https://github.com/pgvector/pgvector" "Warning"
        }
        else {
            Write-ColorOutput "Please install PostgreSQL manually from: https://www.postgresql.org/download/windows/" "Warning"
            return $false
        }

        return $true
    }
    else {
        Write-ColorOutput "Skipping PostgreSQL installation" "Warning"
        return $false
    }
}

function Install-Redis {
    if ($SkipDb) {
        Write-ColorOutput "Skipping database installation (--SkipDb flag)" "Warning"
        return $true
    }

    Write-Step "📦 Checking Redis"

    if (Test-CommandExists "redis-server") {
        $version = & redis-server --version
        Write-ColorOutput "Redis already installed: $version" "Success"
        return $true
    }

    Write-ColorOutput "Redis not found" "Warning"
    Write-ColorOutput "Note: Redis is not officially supported on Windows" "Warning"
    Write-ColorOutput "Consider using Docker or WSL2 for Redis" "Info"

    if (Confirm-Action "Install Redis (Memurai - Windows compatible)?") {
        if (Test-CommandExists "choco") {
            Write-ColorOutput "Installing Memurai (Redis for Windows) via Chocolatey..." "Info"
            Invoke-SafeCommand "choco" @("install", "memurai-developer", "-y")

            Write-ColorOutput "Memurai installed successfully" "Success"
        }
        else {
            Write-ColorOutput "Please install Memurai manually from: https://www.memurai.com/" "Warning"
            Write-ColorOutput "Or use Docker: docker run -d -p 6379:6379 redis:7-alpine" "Info"
            return $false
        }

        return $true
    }
    else {
        Write-ColorOutput "Skipping Redis installation" "Warning"
        Write-ColorOutput "You can use Docker: docker run -d -p 6379:6379 redis:7-alpine" "Info"
        return $false
    }
}

function Install-PythonDeps {
    Write-Step "📦 Installing Python Dependencies"

    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    Set-Location $projectRoot

    # Create virtual environment
    if (-not (Test-Path "venv")) {
        Write-ColorOutput "Creating Python virtual environment..." "Info"
        Invoke-SafeCommand "python" @("-m", "venv", "venv")
        Write-ColorOutput "Virtual environment created" "Success"
    }
    else {
        Write-ColorOutput "Virtual environment already exists" "Success"
    }

    # Activate virtual environment
    Write-ColorOutput "Activating virtual environment..." "Info"
    $activateScript = ".\venv\Scripts\Activate.ps1"

    if ($DryRun) {
        Write-ColorOutput "[DRY-RUN] Would activate virtual environment" "Info"
    }
    else {
        & $activateScript
    }

    # Upgrade pip
    Write-ColorOutput "Upgrading pip..." "Info"
    Invoke-SafeCommand "python" @("-m", "pip", "install", "--upgrade", "pip")

    # Install dependencies
    if (Test-Path "requirements.txt") {
        Write-ColorOutput "Installing Python packages from requirements.txt..." "Info"
        Invoke-SafeCommand "pip" @("install", "-r", "requirements.txt")
        Write-ColorOutput "Python dependencies installed successfully" "Success"
    }
    else {
        Write-ColorOutput "requirements.txt not found. Skipping Python dependencies." "Warning"
    }
}

function Install-NodeDeps {
    Write-Step "📦 Installing Node.js Dependencies"

    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    Set-Location $projectRoot

    if (Test-Path "package.json") {
        Write-ColorOutput "Installing Node.js packages..." "Info"
        Invoke-SafeCommand "npm" @("install")
        Write-ColorOutput "Node.js dependencies installed successfully" "Success"
    }
    else {
        Write-ColorOutput "package.json not found. Skipping Node.js dependencies." "Warning"
    }
}

function Install-RustDeps {
    Write-Step "📦 Building Rust/Tauri Application"

    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    Set-Location "$projectRoot\src-tauri"

    if (Test-Path "Cargo.toml") {
        Write-ColorOutput "Building Tauri application (this may take several minutes)..." "Info"
        Invoke-SafeCommand "cargo" @("build")
        Write-ColorOutput "Tauri application built successfully" "Success"
    }
    else {
        Write-ColorOutput "Cargo.toml not found. Skipping Rust build." "Warning"
    }

    Set-Location $projectRoot
}

function Setup-EnvironmentFiles {
    Write-Step "📦 Setting Up Environment Files"

    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    Set-Location $projectRoot

    # Run setup script
    $setupScript = ".\scripts\setup-env.ps1"

    if (Test-Path $setupScript) {
        Write-ColorOutput "Running environment setup script..." "Info"
        & $setupScript
    }
    else {
        Write-ColorOutput "setup-env.ps1 not found. Creating environment files manually..." "Warning"

        if ((-not (Test-Path ".env")) -and (Test-Path ".env.example")) {
            Copy-Item ".env.example" ".env"
            Write-ColorOutput ".env file created" "Success"
        }

        if ((-not (Test-Path ".mcp.json")) -and (Test-Path ".mcp.template.json")) {
            Copy-Item ".mcp.template.json" ".mcp.json"
            Write-ColorOutput ".mcp.json file created" "Success"
        }
    }
}

function Test-Installation {
    Write-Step "✅ Verifying Installation"

    $allOk = $true

    # Check system tools
    Write-Host ""
    Write-Host "System Tools:" -ForegroundColor Cyan

    if (Test-CommandExists "git") {
        $version = & git --version
        Write-ColorOutput "Git: $version" "Success"
    }
    else {
        Write-ColorOutput "Git: Not installed" "Error"
        $allOk = $false
    }

    if (Test-CommandExists "node") {
        $version = & node --version
        Write-ColorOutput "Node.js: $version" "Success"
    }
    else {
        Write-ColorOutput "Node.js: Not installed" "Error"
        $allOk = $false
    }

    if (Test-CommandExists "python") {
        $version = & python --version
        Write-ColorOutput "Python: $version" "Success"
    }
    else {
        Write-ColorOutput "Python: Not installed" "Error"
        $allOk = $false
    }

    if (Test-CommandExists "rustc") {
        $version = & rustc --version
        Write-ColorOutput "Rust: $version" "Success"
    }
    else {
        Write-ColorOutput "Rust: Not installed" "Error"
        $allOk = $false
    }

    # Check databases
    Write-Host ""
    Write-Host "Databases:" -ForegroundColor Cyan

    if (Test-CommandExists "psql") {
        $version = & psql --version
        Write-ColorOutput "PostgreSQL: $version" "Success"
    }
    else {
        Write-ColorOutput "PostgreSQL: Not installed (optional if using Docker)" "Warning"
    }

    if (Test-CommandExists "redis-cli") {
        $version = & redis-cli --version
        Write-ColorOutput "Redis: $version" "Success"
    }
    elseif (Test-CommandExists "memurai-cli") {
        $version = & memurai-cli --version
        Write-ColorOutput "Memurai (Redis): $version" "Success"
    }
    else {
        Write-ColorOutput "Redis: Not installed (optional if using Docker)" "Warning"
    }

    # Check environment files
    Write-Host ""
    Write-Host "Environment Files:" -ForegroundColor Cyan

    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

    if (Test-Path "$projectRoot\.env") {
        Write-ColorOutput ".env file exists" "Success"
    }
    else {
        Write-ColorOutput ".env file not found" "Warning"
    }

    if (Test-Path "$projectRoot\.mcp.json") {
        Write-ColorOutput ".mcp.json file exists" "Success"
    }
    else {
        Write-ColorOutput ".mcp.json file not found" "Warning"
    }

    # Overall status
    Write-Host ""
    if ($allOk) {
        Write-ColorOutput "All required dependencies are installed!" "Success"
    }
    else {
        Write-ColorOutput "Some dependencies are missing. Please review the output above." "Warning"
    }
}

# ===================================
# Main Execution
# ===================================
function Main {
    if ($Help) {
        Write-Host @"
Usage: .\scripts\install-all.ps1 [OPTIONS]

Options:
  -AutoConfirm      Auto-confirm all prompts
  -DryRun           Show what would be installed without installing
  -SkipDb           Skip database installation (PostgreSQL, Redis)
  -SkipSystem       Skip system tools installation (Git, Node.js, etc.)
  -Help             Show this help message
"@
        exit 0
    }

    Write-Host ""
    Write-Host "╔═══════════════════════════════════════════════════╗" -ForegroundColor Magenta
    Write-Host "║                                                   ║" -ForegroundColor Magenta
    Write-Host "║   🚀 Judgify-core Ver2.0 Final Installation      ║" -ForegroundColor Magenta
    Write-Host "║                                                   ║" -ForegroundColor Magenta
    Write-Host "╚═══════════════════════════════════════════════════╝" -ForegroundColor Magenta
    Write-Host ""

    if ($DryRun) {
        Write-ColorOutput "DRY-RUN MODE: No actual changes will be made" "Warning"
    }

    # Check for admin privileges
    if (-not (Test-AdminPrivileges)) {
        Write-ColorOutput "Note: Some installations may require Administrator privileges" "Warning"
        Write-ColorOutput "If prompted, please run PowerShell as Administrator" "Info"
        Write-Host ""
    }

    # Install Chocolatey
    if (-not $SkipSystem) {
        Install-Chocolatey
    }

    # Install system tools
    if (-not $SkipSystem) {
        Install-Git
        Install-NodeJs
        Install-Python
        Install-Rust
    }

    # Install databases
    Install-PostgreSQL
    Install-Redis

    # Install project dependencies
    Install-PythonDeps
    Install-NodeDeps
    Install-RustDeps

    # Setup environment files
    Setup-EnvironmentFiles

    # Verify installation
    Test-Installation

    # Final instructions
    Write-Host ""
    Write-Step "📝 Next Steps"
    Write-Host ""
    Write-Host "1. Configure environment variables:"
    Write-Host "   - Edit .env file with your database credentials, API keys, etc."
    Write-Host "   - Edit .mcp.json file with your GitHub Personal Access Token"
    Write-Host ""
    Write-Host "2. Activate Python virtual environment:"
    Write-Host "   .\venv\Scripts\Activate.ps1"
    Write-Host ""
    Write-Host "3. Start development server:"
    Write-Host "   npm run dev              # Frontend only"
    Write-Host "   npm run tauri:dev        # Full desktop app"
    Write-Host ""
    Write-Host "4. Review detailed documentation:"
    Write-Host "   - README.md: Project overview"
    Write-Host "   - SETUP.md: Detailed setup guide"
    Write-Host "   - INSTALL.md: Installation troubleshooting"
    Write-Host ""
    Write-ColorOutput "Installation complete! 🎉" "Success"
    Write-Host ""
}

# Run main function
Main
