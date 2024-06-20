# Usage

```
project-soap [input file location] [options]
```

## Input file

project-soap uses ffmpeg to do all the heavy lifting with the input. As such, project-soap can clean almost any video or audio file.

## Options

### -m/--model [path]

Change the path to the model - default is ``C:\Program Files\project-soap\model\`` on Windows and ``~/.project-soap/model/`` on Linux.

### -o/--out [path]

Change the name and location of the output file - without this option, the input file's audio is overwritten.

### -t/--threads [int]

Change the number of threads to run on - default is your system's total number of threads.

## ``get-model``

```
project-soap get-model [options]
```
### --small, --medium, --large

Gets Vosk's small, 0.22-lgraph, and 0.22 models respectively. For a full list of Vosk's available models, please see https://alphacephei.com/vosk/models.

### -m/--model [path]

This option allows you to change where the model is downloaded.