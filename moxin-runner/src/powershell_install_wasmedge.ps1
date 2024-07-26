$ProgressPreference = 'SilentlyContinue' ## makes downloads much faster
Invoke-WebRequest -Uri "https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-0.14.0-windows.zip" -OutFile "$env:TEMP\WasmEdge-0.14.0-windows.zip"
Expand-Archive -Force -Path "$env:TEMP\WasmEdge-0.14.0-windows.zip" -DestinationPath $home

Invoke-WebRequest -Uri "https://github.com/WasmEdge/WasmEdge/releases/download/0.14.0/WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip" -OutFile "$env:TEMP\WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip"
Expand-Archive -Force -Path "$env:TEMP\WasmEdge-plugin-wasi_nn-ggml-0.14.0-windows_x86_64.zip" -DestinationPath "$home\WasmEdge-0.14.0-Windows"
$ProgressPreference = 'Continue' ## restore default progress bars
