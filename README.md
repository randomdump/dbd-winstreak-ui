## Credits

The UI design for this program is based on MomoSeventh's winstreak UI for his streams. You can find him on Twitch at [https://www.twitch.tv/momoseventh](https://www.twitch.tv/momoseventh).

## Quick Start

If you just want to run the application without installing Rust or compiling from source, you can download a pre-built version:

1.  **Visit the Releases page for this project:**
    [https://github.com/randomdump/dbd-winstreak-ui/releases](https://github.com/randomdump/dbd-winstreak-ui/releases)
2.  Locate and download the most recent release.
3.  Extract the contents of the downloaded archive. Inside you will find the executable and all necessary files.
4.  Run the executable directly.

To update the app, simply download the newest version and copy over your `killers.json` and `streaks.txt` files. Note: If you have added custom killers to the `media` folder, you will also need to copy those files to the new version.

## Using in OBS

To add this UI to your OBS scene, follow these steps:

1.  Add a new `Window Capture` source, naming it anything you want.
2.  If you want the transparent background without luma keying in OBS, set the window capture method to `Windows 10 (1903 and up)`.
3.  Right-click on your newly created source and click `Filters`.
4.  Add a `Crop/Pad` video filter, set it to relative, and set the right crop to 320.

If you can't capture the transparent background (or don't want it), check the "Black background" option in the UI and add the `Luma Key` video filter to your capture in OBS. Adjust the `Luma Min` value to your liking; I recommend 0.001.

## Streak Types

Once you run the application, a `streaks.txt` file will be created with instructions in it. If you want to add your own streak types, follow the instructions in that file.

## Adding Custom Killers

If you want to add a new killer (or anything else), add a new image to the `media` folder. The image must be in PNG format and should be 96x96 to look best in the UI. The name of the image will be automatically converted to be shown in the UI (though you can change it in `killers.json` afterwards).

- "TheNurse" or "The_Nurse" turns into "The Nurse"
- "the_nurse" turns into "the nurse"
- "THENURSE" stays "THENURSE"

## Building from Source (Requires Rust)

### Prerequisites

1.  **Install Rust:**
    If you don't already have Rust installed, download and install it from the official website:
    [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Running the Project

Once you have Rust installed:

1.  Open your terminal.
2.  Navigate to the root directory of this project.
3.  Run the following command:

    ```bash
    cargo run
    ```
    or
    ```bash
    cargo run --release
    ```