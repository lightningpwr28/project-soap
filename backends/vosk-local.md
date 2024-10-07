# Vosk - Local

This file lists installation and usage instructions for using the `vosk-local` backend.

## Installation

Getting everything ready to cleam your media is as easy as running `project-soap vosk-local get-model large`! This command downloads the `vosk-model-en-us-0.22` model from [Alpha Cephei's Website](https://alphacephei.com/vosk/models/) and places it in the default model directory (~/.project-soap/vosk/model/). You can have it placed wherever you want by using the `--model` option with the above command and specifying the folder you would like it placed in.

## Options

### -m/--model [path]

Change the path to the model - default is ``%USERPROFILE%\.project-soap\vosk\model\`` on Windows and ``~/.project-soap/vosk/model/`` on Linux.

## ``get-model``

```
project-soap get-model [options]
```
### --small, --medium, --large

Gets Vosk's small, 0.22-lgraph, and 0.22 models respectively. For a full list of Vosk's available models, please see https://alphacephei.com/vosk/models.

### -m/--model [path]

This option allows you to change where the model is downloaded.