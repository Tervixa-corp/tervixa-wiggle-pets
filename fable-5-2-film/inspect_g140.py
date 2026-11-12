import os
from PIL import Image

base = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills"

def scan_column_light(img, x, y0, y1, thresh=150):
    hits = []
    for y in range(y0, y1):
        p = img.getpixel((x, y))
        bright = sum(p) / 3
        if bright > thresh:
            hits.append((y, p))
    return hits

def count_near(img, target, tol=45, region=None):
    w, h = img.size
    if region:
        x0, y0, x1, y1 = region
    else:
        x0, y0, x1, y1 = 0, 0, w, h
    c = 0
    for y in range(y0, y1):
        for x in range(x0, x1):
            p = img.getpixel((x, y))
            if all(abs(p[i] - target[i]) <= tol for i in range(3)):
                c += 1
    return c

img = Image.open(os.path.join(base, "g0140.png")).convert("RGB")
print("=== g140 re-check ===")
hits = scan_column_light(img, 560, 316, 412)
print(f"vertical leader x560 y316-412 light hits: {len(hits)}", hits[:6])

# clay distribution in wordmark: left half (560-960) vs right half (960-1360)
clay_left = count_near(img, (201, 100, 66), 45, (560, 412, 960, 563))
clay_right = count_near(img, (201, 100, 66), 45, (960, 412, 1360, 563))
print(f"clay left-half of wordmark: {clay_left} px")
print(f"clay right-half of wordmark: {clay_right} px")

# light pixels on column x560 below y412 (would be leader touching wordmark - should be none after f156, but at f140 leader is y316-412 only)
below = scan_column_light(img, 560, 412, 563, thresh=150)
print(f"light on x560 y412-563 (leader into wordmark): {len(below)}")
