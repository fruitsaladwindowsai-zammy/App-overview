# Publishing This Wiki to GitHub

GitHub stores repository wikis in a separate `.wiki.git` repository. Once you enable **Wiki** in your GitHub repository settings (`Settings → General → Features → Wikis`) and create one initial placeholder page in the GitHub UI, you can publish this entire `wiki/` folder directly:

```powershell
git clone https://github.com/fruitsaladwindowsai-zammy/App-overview.wiki.git temp-wiki
Copy-Item -Recurse -Force .\wiki\* .\temp-wiki\
Remove-Item .\temp-wiki\README.md -ErrorAction SilentlyContinue
cd temp-wiki
git add -A
git commit -m "Publish Winscroller 2d Appview Wiki v0.1.0"
git push
cd ..
Remove-Item -Recurse -Force temp-wiki
```
