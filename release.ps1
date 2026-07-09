Write-Host "=== Git Status ==="
git status --short

Write-Host "`n=== Committing ==="
git add -A
git commit -m "fix: silent error swallowing, python imports, compare coupling, clean extension package"

Write-Host "`n=== Pushing to main ==="
git push origin main

Write-Host "`n=== Retagging v0.1.0 ==="
git push origin :refs/tags/v0.1.0
git tag -d v0.1.0
git tag v0.1.0
git push origin v0.1.0
