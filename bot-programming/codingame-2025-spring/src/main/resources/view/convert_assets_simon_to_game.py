import os
from PIL import Image

# Source and destination directories
SRC_DIR = "/home/jpn/Pictures/SC2025 -SuperSoaker/Assets/"
DST_DIR = os.path.join(os.path.dirname(__file__), "assets")

# List of image files to process
image_files = [
  "FX_WaterShot.png",
  'Bloc1.png',
  'Bloc1-2.png',
  'Bloc1-3.png',
  'Bloc1-4.png',
  'Bloc1-5.png',
  'Bloc2.png',
  'Bloc2-2.png',
  'Bloc2-3.png',
  'Bloc2-4.png',
  'Bloc2-5.png'
]

def resize_and_save(src_path, dst_path, scale=0.3):
  with Image.open(src_path) as img:
    new_size = (int(img.width * scale), int(img.height * scale))
    resized_img = img.resize(new_size, Image.LANCZOS)
    resized_img.save(dst_path)

if __name__ == "__main__":
  os.makedirs(DST_DIR, exist_ok=True)
  for filename in image_files:
    src_path = os.path.join(SRC_DIR, filename)
    dst_path = os.path.join(DST_DIR, filename)
    if os.path.isfile(src_path):
      resize_and_save(src_path, dst_path)
      print(f"Resized and saved {filename} to assets/")
    else:
      print(f"Source file not found: {src_path}")