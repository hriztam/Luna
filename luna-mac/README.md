# Luna macOS App

This directory contains the source code for the Luna macOS menu bar application.

## Setup Instructions

Since this project does not include a generated `.xcodeproj` file, you need to create one manually in Xcode.

1.  **Open Xcode** and select **Create a new Xcode project**.
2.  Choose **macOS** -> **App**.
3.  Set the Product Name to **Luna**.
4.  Ensure **Interface** is set to **SwiftUI** and **Language** is **Swift**.
5.  Save the project in the `luna-mac` directory (or anywhere you prefer, but you'll need to move the files).
6.  **Replace** the generated files in your new project with the files provided in `luna-mac/Luna/`:
    - `LunaApp.swift` (replaces the main app file)
    - `ContentView.swift` (replaces the default content view)
    - Add `AppDelegate.swift` to the project.
7.  **Build and Run** (Cmd+R).

## Features

- **Menu Bar Icon**: Shows a moon icon in the status bar.
- **Popover**: Click the icon to toggle a popover window.
- **Dark Mode**: The popover content uses a dark appearance.
