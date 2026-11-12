import os
from PIL import Image

base = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills"

def scan_col(img, x, y0, y1, thresh=80):
    hits = []
    for y in range(y0, y1):
        p = img.getpixel((x, y))
        if sum(p) / 3 > thresh:
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

for f in [140, 620, 700]:
    img = Image.open(os.path.join(base, f"g{f:04d}.png")).convert("RGB")
    print(f"\n=== g{f} ===")
    clay = count_near(img, (201, 100, 66), 45)
    bone = count_near(img, (242, 239, 233), 35)
    ice = count_near(img, (191, 233, 245), 45)
    print(f"  clay: {clay}  bone: {bone}  ice: {ice}")
    if f == 140:
        h = scan_col(img, 560, 316, 412)
        print(f"  vertical leader x560 y316-412: {len(h)} hits", h[:4])
        print(f"  clay in wordmark left-half (560-960): {count_near(img,(201,100,66),45,(560,412,960,563))}")
        print(f"  clay in wordmark right-half (960-1360): {count_near(img,(201,100,66),45,(960,412,1360,563))}")
    if f == 700:
        print(f"  axis y800 ice pixels: {count_near(img,(191,233,245),45,(420,795,1580,805))}")
        print(f"  loss-line text area (150-1200, 880-920): {count_near(img,(242,239,233),35,(150,880,1200,920))}")