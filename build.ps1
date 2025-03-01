cargo build --release
mkdir "build" -ErrorAction:SilentlyContinue
mkdir "build\Assets" -ErrorAction:SilentlyContinue
copy-item ".\target\release\lockdown-panel.exe" ".\build\lockdown-panel.exe"
copy-item ".\target\release\lockdown-service.exe" ".\build\Assets\lockdown-service.exe"
upx ".\build\lockdown-panel.exe"
