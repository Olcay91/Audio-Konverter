## Downloads

| System | File |
|---|---|
| Windows (installer) | `Audio Konverter_…_x64-setup.exe` |
| Windows (portable, no installation) | `Audio-Konverter-…-windows-x64-portable.zip` |
| macOS with Apple silicon (M1 and later) | `Audio Konverter_…_aarch64.dmg` |
| macOS with Intel processor | `Audio Konverter_…_x64.dmg` |
| Linux (portable) | `Audio Konverter_…_amd64.AppImage` |
| Linux (Debian, Ubuntu, Mint) | `Audio Konverter_…_amd64.deb` |

ffmpeg is included, nothing else needs to be installed.

Already using Audio Konverter? The installed app (Windows installer, macOS, AppImage) updates itself: *Settings → About & updates → Update now*. The `.sig`, `.tar.gz` and `latest.json` files are used by this in-app updater and don't need to be downloaded manually.

## First launch

The app is not signed with a paid certificate yet, so your system will warn you the first time:

- **Windows:** "Windows protected your PC" → *More info* → *Run anyway*. The portable version needs the WebView2 runtime, which is usually preinstalled on Windows 10/11.
- **macOS:** Right-click the app → *Open* → *Open*. On macOS 15 and later: *System Settings → Privacy & Security → Open Anyway*.
- **Linux (AppImage):** Make the file executable (`chmod +x`) and run it.
