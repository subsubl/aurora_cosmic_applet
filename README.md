# Aurora COSMIC Applet

A native Rust applet for the [COSMIC Desktop Environment](https://github.com/pop-os/cosmic-epoch) that monitors and displays the Aurora Borealis forecast (Kp index).

![Aurora Applet Screenshot](https://via.placeholder.com/600x100?text=Aurora+Applet+Preview)

## Features

- **Real-time Monitoring**: Fetches the latest 3-day Kp forecast from NOAA.
- **Visual Indicators**: Displays the Kp index with color-coded status (Green < 4, Orange < 7, Red >= 7).
- **Desktop Notifications**: Sends a system notification whenever new forecast data is retrieved.
- **Visualizations**: Downloads and combines the latest "viewline" and "ovation" images.
- **Interactive**: Clicking the applet launches `mpv` to display a loop of the latest Aurora images.

## Prerequisites

Ensure you have the following installed on your system:

- **Rust** (Stable)
- **mpv** (for viewing images)
- **System Dependencies** (for `libcosmic`):
  ```bash
  sudo apt install libssl-dev libwayland-dev libxkbcommon-dev pkg-config
  ```


## System Installation (for Panel Integration)

To make the applet appear in the COSMIC Panel "Add Applet" list:

1.  **Build the release binary**:
    ```bash
    cargo build --release
    ```

2.  **Install the binary**:
    ```bash
    sudo cp target/release/aurora_cosmic_applet /usr/local/bin/
    ```

3.  **Install the desktop entry**:
    ```bash
    mkdir -p ~/.local/share/applications
    cp com.user.AuroraApplet.desktop ~/.local/share/applications/
    ```

4.  **Add to Panel**:
    - Right-click the COSMIC Panel.
    - Select "Configure Panel".
    - Click "Add Applet".
    - Select "Aurora Forecast" from the list.

## How it Works

- **Fetching**: The applet polls NOAA services every 15 minutes.
- **Parsing**: It parses the text-based 3-day forecast to determine the maximum Kp index for the current time block (logic adapted from standard NOAA parsing scripts).
- **Caching**: Images and data are cached in `~/.cache/` (e.g., `~/.cache/aurora.txt`, `~/.cache/aurora.png`).

## License

[MIT](LICENSE)
