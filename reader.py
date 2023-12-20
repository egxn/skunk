import cv2
import os
from dotenv import load_dotenv
from paddleocr import PaddleOCR 
from roboflow import Roboflow

load_dotenv()

api_key = os.getenv("ROBOFLOW_API_KEY")
rf = Roboflow(api_key)
project = rf.workspace().project("english-text-bubble-detection")
model = project.version(3).model
ocr = PaddleOCR(use_angle_cls=True, lang='en', use_gpu=False)

predictions = model.predict("hunterx.jpeg")
predictions.save("prediction.jpg")

image = cv2.imread("hunterx.jpeg")
texts = []

for i, prediction in enumerate(predictions):
    min_x = min(point['x'] for point in prediction["points"])
    min_y = min(point['y'] for point in prediction["points"])
    w, h = prediction["width"], prediction["height"]
    x, y, w, h = int(min_x), int(min_y), int(w), int(h)
    roi = image[y:y+h, x:x+w]
    cv2.imwrite("roi_" + str(i) + ".jpg", roi)

    result = ocr.ocr(roi, cls=True)
    for idx in range(len(result)):
        res = result[idx]
        text = "" 
        for line in res:
            text = text + " " + line[1][0]
        texts.append([x, y, text])

print(texts)
