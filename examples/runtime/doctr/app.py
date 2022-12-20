import os
import pathlib
import doctr

deploy_dir = os.environ.get("DEPLOY_DIR") or pathlib.Path(__file__).parent
os.environ["DOCTR_CACHE_DIR"] = str(deploy_dir)
os.environ["DOCTR_MULTIPROCESSING_DISABLE"] = "TRUE"
OCR_MODEL = doctr.models.ocr_predictor(
    pretrained=True, det_arch="db_resnet50", reco_arch="crnn_mobilenet_v3_small",
)

def cape_handler(pdf_bytes: bytes):
    doc = doctr.io.DocumentFile.from_pdf(pdf_bytes)
    result = OCR_MODEL(doc)
    transcript = result.render()
    return transcript.encode()


if __name__ == "__main__":
    result = cape_handler("../equifax_sample_100_pages.pdf")
    print(result)

