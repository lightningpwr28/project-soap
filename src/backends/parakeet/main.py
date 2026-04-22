import json
import logging
import sys

logging.disable(logging.CRITICAL)

import nemo.collections.asr as nemo_asr
import torch


def main():
    asr_model = nemo_asr.models.ASRModel.from_pretrained(
        model_name="nvidia/parakeet-tdt-0.6b-v2"
    )

    output = asr_model.transcribe([sys.argv[1]], timestamps=True)

    print(output[0].timestamp["word"])

    out_json = json.dumps(output[0].timestamp["word"])
    # print(out_json)

    with open("out.json", "w") as f:
        f.write(out_json)


if __name__ == "__main__":
    main()
