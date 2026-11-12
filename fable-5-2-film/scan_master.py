"""Blank-frame scan + title-safe check on out/master.mp4."""
import subprocess, json, os

OUT = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\master.mp4"
TMP = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\scan"

# Extract one frame every 10 frames (180 frames total) as JPEG
os.makedirs(TMP, exist_ok=True)
subprocess.run([
    "ffmpeg", "-y", "-loglevel", "error",
    "-i", OUT,
    "-vf", "fps=6",
    "-q:v", "3",
    f"{TMP}\\f%04d.jpg"
], check=True)

from PIL import Image
import glob

files = sorted(glob.glob(os.path.join(TMP, "*.jpg")))
print(f"extracted {len(files)} frames")

blank = []
dark = []
for f in files:
    img = Image.open(f).convert("L")
    # mean brightness
    hist = img.histogram()
    total = sum(hist)
    mean = sum(i * hist[i] for i in range(256)) / total
    # fraction of pixels above threshold 25
    lit = sum(hist[25:]) / total
    name = os.path.basename(f)
    if mean < 3:
        blank.append((name, round(mean, 2)))
    if mean < 20:
        dark.append((name, round(mean, 2), round(lit, 4)))

print(f"\nblank (mean<3): {len(blank)}", blank[:10])
print(f"dark  (mean<20): {len(dark)}", dark[:10])

# Title-safe: check that content reaches within 5% margins
# Sample the inner 90% region vs outer 10% border
print("\n=== title-safe margins ===")
for f in files[::30]:  # every 30th extracted frame
    img = Image.open(f).convert("L")
    w, h = img.size
    # outer 5% border ring
    border_vals = []
    for x in range(0, w, 7):
        border_vals.append(img.getpixel((x, 5)))
        border_vals.append(img.getpixel((x, h - 6)))
    for y in range(0, h, 7):
        border_vals.append(img.getpixel((5, y)))
        border_vals.append(img.getpixel((w - 6, y)))
    border_lit = sum(1 for v in border_vals if v > 40) / len(border_vals)
    name = os.path.basename(f)
    print(f"  {name}: border lit fraction = {border_lit:.3f}")