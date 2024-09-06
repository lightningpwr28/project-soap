This file contains backend-agnostic options. For backend-specific options, see their documentation.

# Usage

```
project-soap [backend] [input file location] [options]
```

## Backend

Currently, there are two backends implemented: [vosk-local](/backends/vosklocal.md) and [whisper-x-local](/backends/whisperxlocal.md).

## Input file

project-soap uses ffmpeg to do all the heavy lifting with the input. As such, project-soap can clean almost any video or audio file.

## Options

### -o/--out [path]

Change the name and location of the output file - without this option, the input file's audio is overwritten.

### -t/--threads [int]

Change the number of threads to run on - default is your system's total number of threads.