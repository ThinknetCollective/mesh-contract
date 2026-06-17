# Build Issues on Windows

## Current Issue
When running `cargo build` or `cargo check` on Windows, the build fails with:
```
error: failed to remove C:\...\target\debug\deps\...: The process cannot access the file because it is being used by another process. (os error 32)
```

Docker builds also fail with permission denied errors when trying to write to the mounted volume.

## Root Cause
This is a common Windows issue where:
1. Processes (often antivirus software or Windows Defender) hold file locks on build artifacts
2. Docker volume mounts on Windows have permission issues preventing writes from containers

## Workarounds

### Option 1: Use WSL (Recommended)
The most reliable solution is to use Windows Subsystem for Linux (WSL):
```bash
# Install WSL if not already installed
wsl --install

# In WSL, navigate to your project
cd /mnt/c/Users/USER/OneDrive/Music/Gt\ 2

# Install Rust target
rustup target add wasm32-unknown-unknown

# Build contracts
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

### Option 2: Use GitHub Actions
Build and test using GitHub Actions CI/CD, which runs in a Linux environment. Create a `.github/workflows/build.yml` file.

### Option 3: Use a Linux VM
Build in a Linux virtual machine or cloud environment.

### Option 4: Fix Windows Permissions (Advanced)
Add the project directory to antivirus exclusions and adjust Docker permissions, but this is complex and unreliable.

## Deployment Scripts
Both bash (.sh) and PowerShell (.ps1) versions of deployment scripts are provided:
- `scripts/deploy.sh` - Bash script for Linux/Mac/WSL
- `scripts/deploy.ps1` - PowerShell script for Windows
- `scripts/seed_testnet.sh` - Bash seeding script for Linux/Mac/WSL
- `scripts/seed_testnet.ps1` - PowerShell seeding script for Windows

## Status
- ✅ Contract code is syntactically correct
- ✅ Deployment scripts are ready for use
- ✅ README updated with Stellar instructions
- ⚠️ Build environment needs WSL or Linux for reliable builds
- ⚠️ Windows Docker builds have permission issues

## Recommendation
Use WSL for development and deployment on Windows. This provides a Linux environment within Windows that avoids file locking and permission issues.
