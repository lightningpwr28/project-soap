# project-Soap

project-soap is an open source command line program that removes expletives from media with audio. Powered by Vosk.

## Installation

### Requirements

FFmpeg is required for project-soap to function. Different backends may have other additional requirements.

### Steps

1. Download the latest release [here](https://github.com/lightningpwr28/project-soap/releases)
2. Configure your backend of choice. I recommend using [Vosk](/backends/vosklocal.md) for cpu inferencing and [WhisperX](/backends/whisperxlocal.md) if you have an Nvidia gpu.

## Usage
I recommend using this tool in conjunction with [Stacher](https://stacher.io/) and it's custom post-processing feature - simply add ``project-soap {}`` to a new line!

For more complete usage instructions, see [usage.md](https://github.com/lightningpwr28/project-soap/blob/master/usage.md)