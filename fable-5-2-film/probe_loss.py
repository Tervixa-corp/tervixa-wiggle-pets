import os
from PIL import Image

img = Image.open(r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills\g0700.png").convert("RGB")
# loss line text area: left 150, top 892, spans ~1200px wide, 26px tall
print("=== g0700 loss line probe ===")
for y in [888, 892, 896, 900, 904, 908, 912]:
    row = [img.getpixel((x, y)) for x in [200, 300, 400, 500, 600, 700, 800]]
    print(f"  y{y}: {row}")
# brightest pixel in the loss-line band
best = []
for y in range(880, 920):
    for x in range(150, 1200):
        p = img.getpixel((x, y))
        best.append((sum(p)/3, x, y, p))
best.sort(reverse=True)
print("  brightest in band:", best[:5])
# also check footer source line at y866
print("=== g0700 footer probe ===")
for y in [862, 866, 870]:
    row = [img.getpixel((x, y)) for x in [200, 300, 400, 500]]
    print(f"  y{y}: {row}")
best2 = []
for y in range(855, 880):
    for x in range(150, 800):
        p = img.getpixel((x, y))
        best2.append((sum(p)/3, x, y, p))
best2.sort(reverse=True)
print("  brightest in footer band:", best2[:5])