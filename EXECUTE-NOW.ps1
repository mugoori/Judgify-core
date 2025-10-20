# Judgify-core Execution Script
# Run with PowerShell Administrator

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "Judgify-core Ver2.0 Final" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check Environment
Write-Host "Step 1: Checking environment..." -ForegroundColor Yellow
Write-Host ""

# Check Node.js
Write-Host "Checking Node.js..." -ForegroundColor Green
node --version
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Node.js not installed!" -ForegroundColor Red
    Write-Host "Install from: https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}
Write-Host "[OK] Node.js installed" -ForegroundColor Green
Write-Host ""

# Check Rust
Write-Host "Checking Rust..." -ForegroundColor Green
$rustInstalled = $false
try {
    $null = cargo --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        $rustInstalled = $true
        Write-Host "[OK] Rust already installed" -ForegroundColor Green
        cargo --version
    }
}
catch {
    Write-Host "[WARN] Rust not installed" -ForegroundColor Yellow
}

if (-not $rustInstalled) {
    Write-Host ""
    Write-Host "Choose Rust installation method:" -ForegroundColor Cyan
    Write-Host "1. Auto install with winget (Recommended)" -ForegroundColor White
    Write-Host "2. Manual installation guide" -ForegroundColor White
    Write-Host "3. Skip (npm only)" -ForegroundColor White

    $choice = Read-Host "Select (1-3)"

    if ($choice -eq "1") {
        Write-Host "Installing Rust with winget..." -ForegroundColor Yellow
        winget install Rustlang.Rustup

        Write-Host ""
        Write-Host "[OK] Rust installation complete!" -ForegroundColor Green
        Write-Host "[IMPORTANT] Close PowerShell completely and restart!" -ForegroundColor Red
        Write-Host "[IMPORTANT] Then run this script again." -ForegroundColor Red
        Write-Host ""
        Read-Host "Press Enter to exit"
        exit 0
    }
    elseif ($choice -eq "2") {
        Write-Host ""
        Write-Host "Manual installation guide:" -ForegroundColor Cyan
        Write-Host "1. Visit https://rustup.rs/" -ForegroundColor White
        Write-Host "2. Download rustup-init.exe" -ForegroundColor White
        Write-Host "3. Run and complete installation" -ForegroundColor White
        Write-Host "4. Restart PowerShell" -ForegroundColor White
        Write-Host "5. Run this script again" -ForegroundColor White
        Write-Host ""
        Read-Host "Press Enter to exit"
        exit 0
    }
}
Write-Host ""

# Check Visual Studio Build Tools
Write-Host "Checking Visual Studio Build Tools..." -ForegroundColor Green
$vsBuildToolsPath = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"
if (Test-Path $vsBuildToolsPath) {
    Write-Host "[OK] Visual Studio Build Tools installed" -ForegroundColor Green
}
else {
    Write-Host "[WARN] Visual Studio Build Tools may not be installed" -ForegroundColor Yellow
    Write-Host "If Rust compilation fails, install with:" -ForegroundColor Yellow
    Write-Host "winget install Microsoft.VisualStudio.2022.BuildTools" -ForegroundColor White
    Write-Host "(Select 'Desktop development with C++' during installation)" -ForegroundColor White
}
Write-Host ""

# Check pnpm
Write-Host "Checking pnpm..." -ForegroundColor Green
try {
    $null = pnpm --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] pnpm installed" -ForegroundColor Green
        pnpm --version
    }
}
catch {
    Write-Host "[WARN] pnpm not installed - will use npm" -ForegroundColor Yellow
    Write-Host "Recommended: npm install -g pnpm" -ForegroundColor White
}
Write-Host ""

# Step 2: Check .env file
Write-Host "Step 2: Checking environment configuration..." -ForegroundColor Yellow
Write-Host ""

if (Test-Path ".env") {
    Write-Host "[OK] .env file exists" -ForegroundColor Green

    $envContent = Get-Content ".env" -Raw
    if ($envContent -match "sk-test-key") {
        Write-Host "[WARN] Please check OPENAI_API_KEY in .env file" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Edit .env file now? (Y/N)" -ForegroundColor Cyan
        $editEnv = Read-Host

        if ($editEnv -eq "Y" -or $editEnv -eq "y") {
            notepad .env
            Write-Host "[OK] .env file opened for editing" -ForegroundColor Green
            Write-Host "Please enter your real OpenAI API Key!" -ForegroundColor Yellow
            Read-Host "Press Enter after saving"
        }
    }
    else {
        Write-Host "[OK] OPENAI_API_KEY configured" -ForegroundColor Green
    }
}
else {
    Write-Host "[ERROR] .env file not found!" -ForegroundColor Red
    Write-Host "Creating .env from .env.example..." -ForegroundColor Yellow
    Copy-Item .env.example .env
    Write-Host "[OK] .env file created" -ForegroundColor Green
    Write-Host ""
    Write-Host "Please set OPENAI_API_KEY in .env file" -ForegroundColor Cyan
    notepad .env
    Read-Host "Press Enter after configuration"
}
Write-Host ""

# Step 3: Run
Write-Host "Step 3: Preparing to run development server..." -ForegroundColor Yellow
Write-Host ""

Write-Host "Select execution method:" -ForegroundColor Cyan
Write-Host "1. Run with pnpm (Recommended)" -ForegroundColor White
Write-Host "2. Run with npm" -ForegroundColor White

$runChoice = Read-Host "Select (1-2)"

Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "Starting development server..." -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "[INFO] First run takes 5-10 minutes (Rust compilation)" -ForegroundColor Yellow
Write-Host "[INFO] Frontend build: ~10 seconds" -ForegroundColor Yellow
Write-Host "[INFO] Rust compile: ~5-10 minutes (download + compile)" -ForegroundColor Yellow
Write-Host ""
Write-Host "[SUCCESS] Desktop app will open automatically!" -ForegroundColor Green
Write-Host ""

if ($runChoice -eq "1") {
    # Check pnpm installation
    try {
        $null = pnpm --version 2>&1
    }
    catch {
        Write-Host "Installing pnpm..." -ForegroundColor Yellow
        npm install -g pnpm
        Write-Host "[OK] pnpm installed" -ForegroundColor Green
        Write-Host ""
    }

    Write-Host "Running: pnpm tauri dev" -ForegroundColor Cyan
    pnpm tauri dev
}
else {
    Write-Host "Running: npm run tauri:dev" -ForegroundColor Cyan
    npm run tauri:dev
}

Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "Execution completed or terminated" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
