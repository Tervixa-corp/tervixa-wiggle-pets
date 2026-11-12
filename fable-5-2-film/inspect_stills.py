from PIL import Image
import os

base = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills"

def count_near(img, target, tol=40, region=None):
    """Count pixels within tol of target rgb, optionally in (x0,y0,x1,y1) region."""
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

def scan_column_light(img, x, y0, y1, tol_thresh=60):
    """Find if there's a light (bright) vertical line at column x between y0,y1."""
    hits = []
    for y in range(y0, y1):
        p = img.getpixel((x, y))
        bright = sum(p) / 3
        if bright > tol_thresh:
            hits.append((y, p))
    return hits

frames = [140, 620, 700]
for f in frames:
    path = os.path.join(base, f"g{f:04d}.png")
    img = Image.open(path).convert("RGB")
    print(f"\n=== g{f} ===")
    clay = count_near(img, (201, 100, 66), 45)
    bone = count_near(img, (242, 239, 233), 35)
    ice = count_near(img, (191, 233, 245), 45)
    print(f"  clay(hero ~201,100,66): {clay} px")
    print(f"  bone(value ~242,239,233): {bone} px")
    print(f"  ice(axis ~191,233,245): {ice} px")

    if f == 140:
        # vertical leader at x=560 y316-412 should be light
        hits = scan_column_light(img, 560, 316, 412)
        print(f"  vertical leader col x560 y316-412 light hits: {len(hits)}", hits[:5])
        # wordmark area x560-1360 y412-563: clay pixels for 5.2
        clay_in_wordmark = count_near(img, (201, 100, 66), 45, (560, 412, 1360, 563))
        bone_in_wordmark = count_near(img, (242, 239, 233), 35, (560, 412, 1360, 563))
        print(f"  clay in wordmark area: {clay_in_wordmark} px")
        print(f"  bone in wordmark area: {bone_in_wordmark} px")
