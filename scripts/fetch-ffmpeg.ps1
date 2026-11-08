# Downloads a static FFmpeg build for Windows x64 and places it as the Tauri sidecar.
# Source: https://www.gyan.dev/ffmpeg/builds/ (GPL "essentials" build)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe"
if (Test-Path $dest) { Write-Host "Already present: $dest"; exit 0 }

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("ffmpeg-" + [Guid]::NewGuid())
New-Item -ItemType Directory $tmp | Out-Null
try {
  $zip = Join-Path $tmp "ffmpeg.zip"
  Write-Host "Downloading FFmpeg..."
  Invoke-WebRequest "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" -OutFile $zip
  Expand-Archive $zip -DestinationPath $tmp
  $exe = Get-ChildItem $tmp -Recurse -Filter ffmpeg.exe | Select-Object -First 1
  Copy-Item $exe.FullName $dest
  Write-Host "Installed: $dest"
} finally {
  Remove-Item $tmp -Recurse -Force
}
