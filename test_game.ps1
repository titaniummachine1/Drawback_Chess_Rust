Write-Host "Building Drawback Chess..."
cargo build
if ($LASTEXITCODE -eq 0) {
    Write-Host "Running Drawback Chess with configuration file..."
    cargo run
} else {
    Write-Host "Build failed with exit code $LASTEXITCODE"
} 