$ErrorActionPreference = 'Stop'

$tempDir = ".temp"
if (Test-Path $tempDir) {
    Remove-Item -Recurse -Force $tempDir
}
New-Item -ItemType Directory -Path $tempDir | Out-Null

Write-Host "Cloning Tier A Repositories..."
# ARES is already local, we will test it in place or copy it
Write-Host "Cloning Automyra..."
git clone https://github.com/Eswar-426/Automyra.git "$tempDir/automyra" --depth 1

Write-Host "Cloning Tier B Repositories..."
Write-Host "Cloning ripgrep..."
git clone https://github.com/BurntSushi/ripgrep.git "$tempDir/ripgrep" --depth 1

Write-Host "Cloning cargo-watch..."
git clone https://github.com/watchexec/cargo-watch.git "$tempDir/cargo-watch" --depth 1

Write-Host "Cloning nextjs starter..."
# We can just clone a small template or the main next.js repo. A huge repo takes a long time.
# Let's clone vercel's nextjs commerce or a simple starter.
git clone https://github.com/vercel/next.js.git "$tempDir/nextjs" --depth 1

Write-Host "Cloning nestjs starter..."
git clone https://github.com/nestjs/nest.git "$tempDir/nestjs" --depth 1

Write-Host "Cloning turborepo starter..."
git clone https://github.com/vercel/turborepo.git "$tempDir/turborepo" --depth 1

Write-Host "Cloning nx workspace..."
git clone https://github.com/nrwl/nx.git "$tempDir/nx" --depth 1

Write-Host "All benchmark repositories cloned successfully into .temp/"
