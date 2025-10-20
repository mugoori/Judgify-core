@echo off
REM ===================================
REM Judgify-core Ver2.0 Final - Unified Installation Script (Windows CMD)
REM ===================================
REM This script installs all dependencies needed for development on Windows
REM Requires: Windows 10/11
REM
REM Usage:
REM   scripts\install-all.bat              # Interactive mode
REM   scripts\install-all.bat --yes        # Auto-confirm all
REM   scripts\install-all.bat --dry-run    # Show what would be installed
REM   scripts\install-all.bat --skip-db    # Skip database installation
REM
REM ===================================

setlocal enabledelayedexpansion

REM Parse arguments
set AUTO_CONFIRM=0
set DRY_RUN=0
set SKIP_DB=0
set SKIP_SYSTEM=0

:parse_args
if "%~1"=="" goto :args_done
if /i "%~1"=="--yes" set AUTO_CONFIRM=1
if /i "%~1"=="-y" set AUTO_CONFIRM=1
if /i "%~1"=="--dry-run" set DRY_RUN=1
if /i "%~1"=="--skip-db" set SKIP_DB=1
if /i "%~1"=="--skip-system" set SKIP_SYSTEM=1
if /i "%~1"=="--help" goto :show_help
if /i "%~1"=="-h" goto :show_help
shift
goto :parse_args

:show_help
echo Usage: %~nx0 [OPTIONS]
echo.
echo Options:
echo   --yes, -y          Auto-confirm all prompts
echo   --dry-run          Show what would be installed without installing
echo   --skip-db          Skip database installation (PostgreSQL, Redis)
echo   --skip-system      Skip system tools installation (Git, Node.js, etc.)
echo   --help, -h         Show this help message
exit /b 0

:args_done

REM ===================================
REM Main Installation
REM ===================================

echo.
echo ===================================================
echo.
echo    Judgify-core Ver2.0 Final Installation
echo.
echo ===================================================
echo.

if %DRY_RUN%==1 (
    echo [WARNING] DRY-RUN MODE: No actual changes will be made
    echo.
)

REM Check for admin privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] Some installations may require Administrator privileges
    echo [INFO] If prompted, please run Command Prompt as Administrator
    echo.
)

REM ===================================
REM Recommend using PowerShell
REM ===================================
echo [INFO] For better installation experience, we recommend using PowerShell:
echo.
echo    PowerShell:
echo    .\scripts\install-all.ps1
echo.
echo    Or use Chocolatey package manager:
echo    https://chocolatey.org/install
echo.

if %AUTO_CONFIRM%==0 (
    choice /m "Continue with Command Prompt installation"
    if errorlevel 2 exit /b 0
)

REM ===================================
REM Check for Chocolatey
REM ===================================
echo.
echo ===================================================
echo Checking Chocolatey (Windows Package Manager)
echo ===================================================
echo.

where choco >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Chocolatey already installed
    for /f "tokens=*" %%a in ('choco --version') do echo Version: %%a
) else (
    echo [WARNING] Chocolatey not found
    echo.
    echo Please install Chocolatey first:
    echo   1. Open PowerShell as Administrator
    echo   2. Run: Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    echo   3. Restart Command Prompt
    echo.
    echo Or download from: https://chocolatey.org/install
    echo.
    pause
    exit /b 1
)

REM ===================================
REM Check Git
REM ===================================
if %SKIP_SYSTEM%==1 goto :skip_system

echo.
echo ===================================================
echo Checking Git
echo ===================================================
echo.

where git >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Git already installed
    git --version
) else (
    echo [WARNING] Git not found
    if %AUTO_CONFIRM%==0 (
        choice /m "Install Git"
        if errorlevel 2 goto :skip_git
    )

    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: choco install git -y
    ) else (
        echo Installing Git...
        choco install git -y
        echo [OK] Git installed
    )
)
:skip_git

REM ===================================
REM Check Node.js
REM ===================================
echo.
echo ===================================================
echo Checking Node.js
echo ===================================================
echo.

where node >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Node.js already installed
    node --version
) else (
    echo [WARNING] Node.js not found
    if %AUTO_CONFIRM%==0 (
        choice /m "Install Node.js 18.x LTS"
        if errorlevel 2 goto :skip_node
    )

    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: choco install nodejs-lts --version=18.19.0 -y
    ) else (
        echo Installing Node.js...
        choco install nodejs-lts --version=18.19.0 -y
        echo [OK] Node.js installed
        echo [WARNING] Please restart Command Prompt to use Node.js
    )
)
:skip_node

REM ===================================
REM Check Python
REM ===================================
echo.
echo ===================================================
echo Checking Python
echo ===================================================
echo.

where python >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Python already installed
    python --version
) else (
    echo [WARNING] Python not found
    if %AUTO_CONFIRM%==0 (
        choice /m "Install Python 3.11"
        if errorlevel 2 goto :skip_python
    )

    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: choco install python311 -y
    ) else (
        echo Installing Python...
        choco install python311 -y
        echo [OK] Python installed
        echo [WARNING] Please restart Command Prompt to use Python
    )
)
:skip_python

REM ===================================
REM Check Rust
REM ===================================
echo.
echo ===================================================
echo Checking Rust
echo ===================================================
echo.

where rustc >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Rust already installed
    rustc --version
) else (
    echo [WARNING] Rust not found
    if %AUTO_CONFIRM%==0 (
        choice /m "Install Rust (required for Tauri)"
        if errorlevel 2 goto :skip_rust
    )

    echo [INFO] Please install Rust manually:
    echo   1. Visit: https://rustup.rs/
    echo   2. Download and run rustup-init.exe
    echo   3. Restart Command Prompt
    echo.
    pause
)
:skip_rust

:skip_system

REM ===================================
REM Check PostgreSQL
REM ===================================
if %SKIP_DB%==1 goto :skip_databases

echo.
echo ===================================================
echo Checking PostgreSQL
echo ===================================================
echo.

where psql >nul 2>&1
if %errorLevel%==0 (
    echo [OK] PostgreSQL already installed
    psql --version
) else (
    echo [WARNING] PostgreSQL not found
    if %AUTO_CONFIRM%==0 (
        choice /m "Install PostgreSQL 15"
        if errorlevel 2 goto :skip_postgresql
    )

    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: choco install postgresql15 -y --params "/Password:postgres"
    ) else (
        echo Installing PostgreSQL...
        choco install postgresql15 -y --params "/Password:postgres"
        echo [OK] PostgreSQL installed
        echo [WARNING] Default password: postgres
        echo [WARNING] Please change the default password!
        echo.
        echo [INFO] pgvector extension installation:
        echo   Visit: https://github.com/pgvector/pgvector
    )
)
:skip_postgresql

REM ===================================
REM Check Redis
REM ===================================
echo.
echo ===================================================
echo Checking Redis
echo ===================================================
echo.

echo [INFO] Redis is not officially supported on Windows
echo [INFO] Consider using Docker or WSL2 for Redis
echo.

where redis-server >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Redis already installed
    redis-server --version
) else (
    if %AUTO_CONFIRM%==0 (
        choice /m "Install Memurai (Redis for Windows)"
        if errorlevel 2 goto :skip_redis
    )

    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: choco install memurai-developer -y
    ) else (
        echo Installing Memurai...
        choco install memurai-developer -y
        echo [OK] Memurai installed
    )
)
:skip_redis

:skip_databases

REM ===================================
REM Install Python Dependencies
REM ===================================
echo.
echo ===================================================
echo Installing Python Dependencies
echo ===================================================
echo.

cd /d "%~dp0.."

if not exist "venv" (
    echo Creating Python virtual environment...
    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: python -m venv venv
    ) else (
        python -m venv venv
        echo [OK] Virtual environment created
    )
) else (
    echo [OK] Virtual environment already exists
)

echo Activating virtual environment...
if %DRY_RUN%==0 (
    call venv\Scripts\activate.bat
)

echo Upgrading pip...
if %DRY_RUN%==1 (
    echo [DRY-RUN] Would run: python -m pip install --upgrade pip
) else (
    python -m pip install --upgrade pip
)

if exist "requirements.txt" (
    echo Installing Python packages from requirements.txt...
    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: pip install -r requirements.txt
    ) else (
        pip install -r requirements.txt
        echo [OK] Python dependencies installed successfully
    )
) else (
    echo [WARNING] requirements.txt not found
)

REM ===================================
REM Install Node.js Dependencies
REM ===================================
echo.
echo ===================================================
echo Installing Node.js Dependencies
echo ===================================================
echo.

if exist "package.json" (
    echo Installing Node.js packages...
    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: npm install
    ) else (
        npm install
        echo [OK] Node.js dependencies installed successfully
    )
) else (
    echo [WARNING] package.json not found
)

REM ===================================
REM Build Rust/Tauri Application
REM ===================================
echo.
echo ===================================================
echo Building Rust/Tauri Application
echo ===================================================
echo.

cd src-tauri

if exist "Cargo.toml" (
    echo Building Tauri application (this may take several minutes)...
    if %DRY_RUN%==1 (
        echo [DRY-RUN] Would run: cargo build
    ) else (
        cargo build
        echo [OK] Tauri application built successfully
    )
) else (
    echo [WARNING] Cargo.toml not found
)

cd ..

REM ===================================
REM Setup Environment Files
REM ===================================
echo.
echo ===================================================
echo Setting Up Environment Files
echo ===================================================
echo.

if exist "scripts\setup-env.bat" (
    echo Running environment setup script...
    call scripts\setup-env.bat
) else (
    echo [WARNING] setup-env.bat not found

    if not exist ".env" (
        if exist ".env.example" (
            copy .env.example .env
            echo [OK] .env file created
        )
    )

    if not exist ".mcp.json" (
        if exist ".mcp.template.json" (
            copy .mcp.template.json .mcp.json
            echo [OK] .mcp.json file created
        )
    )
)

REM ===================================
REM Verify Installation
REM ===================================
echo.
echo ===================================================
echo Verifying Installation
echo ===================================================
echo.

echo System Tools:
echo.

where git >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Git:
    git --version
) else (
    echo [ERROR] Git: Not installed
)

where node >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Node.js:
    node --version
) else (
    echo [ERROR] Node.js: Not installed
)

where python >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Python:
    python --version
) else (
    echo [ERROR] Python: Not installed
)

where rustc >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Rust:
    rustc --version
) else (
    echo [ERROR] Rust: Not installed
)

echo.
echo Databases:
echo.

where psql >nul 2>&1
if %errorLevel%==0 (
    echo [OK] PostgreSQL:
    psql --version
) else (
    echo [WARNING] PostgreSQL: Not installed (optional if using Docker)
)

where redis-cli >nul 2>&1
if %errorLevel%==0 (
    echo [OK] Redis:
    redis-cli --version
) else (
    where memurai-cli >nul 2>&1
    if %errorLevel%==0 (
        echo [OK] Memurai (Redis):
        memurai-cli --version
    ) else (
        echo [WARNING] Redis: Not installed (optional if using Docker)
    )
)

echo.
echo Environment Files:
echo.

if exist ".env" (
    echo [OK] .env file exists
) else (
    echo [WARNING] .env file not found
)

if exist ".mcp.json" (
    echo [OK] .mcp.json file exists
) else (
    echo [WARNING] .mcp.json file not found
)

REM ===================================
REM Final Instructions
REM ===================================
echo.
echo ===================================================
echo Next Steps
echo ===================================================
echo.
echo 1. Configure environment variables:
echo    - Edit .env file with your database credentials, API keys, etc.
echo    - Edit .mcp.json file with your GitHub Personal Access Token
echo.
echo 2. Activate Python virtual environment:
echo    venv\Scripts\activate.bat
echo.
echo 3. Start development server:
echo    npm run dev              # Frontend only
echo    npm run tauri:dev        # Full desktop app
echo.
echo 4. Review detailed documentation:
echo    - README.md: Project overview
echo    - SETUP.md: Detailed setup guide
echo    - INSTALL.md: Installation troubleshooting
echo.
echo [OK] Installation complete!
echo.
pause
