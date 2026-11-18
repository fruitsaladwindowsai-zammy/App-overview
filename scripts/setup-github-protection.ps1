<#
.SYNOPSIS
    Configures GitHub repository settings for Winscroller 2d Appview:
      1. Renames default branch master -> main (if still on master)
      2. Configures branch protection on `main` (requires Windows CI `test-and-check`,
         blocks force-pushes & deletions, keeps admin pushes + auto-releases working)
      3. Enables GitHub Pages with GitHub Actions workflow deployment
      4. Triggers the mdBook GitHub Pages deployment workflow
.NOTES
    Requires GitHub CLI (`gh`) authenticated with `repo` and `admin:repo_hook` scopes:
      winget install --id GitHub.cli
      gh auth login
#>

param(
    [string]$Repo = "fruitsaladwindowsai-zammy/App-overview"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host "GitHub CLI (gh) not found. Install it first with:" -ForegroundColor Yellow
    Write-Host "  winget install --id GitHub.cli" -ForegroundColor Cyan
    Write-Host "Then restart PowerShell, run 'gh auth login', and re-run this script." -ForegroundColor Yellow
    exit 1
}

Write-Host "==> Checking GitHub authentication..." -ForegroundColor Cyan
gh auth status

Write-Host "==> Checking default branch on $Repo..." -ForegroundColor Cyan
$defaultBranch = gh api "repos/$Repo" --jq ".default_branch"
Write-Host "Current default branch: $defaultBranch" -ForegroundColor Gray

if ($defaultBranch -eq "master") {
    Write-Host "==> Renaming default branch master -> main on GitHub..." -ForegroundColor Cyan
    gh api --method POST "repos/$Repo/branches/master/rename" -f new_name=main | Out-Null
    Write-Host "Renamed GitHub default branch to 'main'!" -ForegroundColor Green

    # Update local git branch tracking if running inside repo
    $gitExe = (Get-Command git -ErrorAction SilentlyContinue)?.Source
    if (-not $gitExe) {
        $gitExe = (Get-Item "$env:LOCALAPPDATA\GitHubDesktop\app-*\resources\app\git\cmd\git.exe" -ErrorAction SilentlyContinue | Select-Object -First 1).FullName
    }
    if ($gitExe) {
        $repoRoot = Split-Path $PSScriptRoot -Parent
        & $gitExe -C $repoRoot branch -m master main 2>$null
        & $gitExe -C $repoRoot fetch origin
        & $gitExe -C $repoRoot branch -u origin/main main
        & $gitExe -C $repoRoot remote set-head origin -a
        Write-Host "Updated local git branch to track origin/main." -ForegroundColor Green
    }
}

Write-Host "==> Applying branch protection to 'main'..." -ForegroundColor Cyan
$protectionJson = @'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["test-and-check"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
'@

$protectionJson | gh api --method PUT "repos/$Repo/branches/main/protection" --input - | Out-Null
Write-Host "Branch protection applied to 'main' (requires Windows CI 'test-and-check', blocks force-push & deletion, allows admin push + auto-release)." -ForegroundColor Green

Write-Host "==> Enabling GitHub Pages (GitHub Actions workflow source)..." -ForegroundColor Cyan
try {
    '{"build_type":"workflow"}' | gh api --method POST "repos/$Repo/pages" --input - | Out-Null
    Write-Host "GitHub Pages enabled!" -ForegroundColor Green
} catch {
    '{"build_type":"workflow"}' | gh api --method PUT "repos/$Repo/pages" --input - | Out-Null
    Write-Host "GitHub Pages updated to workflow deployment!" -ForegroundColor Green
}

Write-Host "==> Triggering GitHub Pages documentation deployment..." -ForegroundColor Cyan
gh workflow run gh-pages.yml --repo $Repo --ref main
Write-Host ""
Write-Host "All GitHub protections and automations are configured!" -ForegroundColor Green
Write-Host "  Repo:    https://github.com/$Repo" -ForegroundColor Yellow
Write-Host "  Actions: https://github.com/$Repo/actions" -ForegroundColor Yellow
Write-Host "  Docs:    https://fruitsaladwindowsai-zammy.github.io/App-overview/" -ForegroundColor Yellow
